//! Agent API Handlers
//!
//! HTTP request handlers for all Agent API endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    Json,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::pin::Pin;

use super::models::*;
use super::orchestrator::OrchestratorService;
use super::sse as sse_utils;
use super::state::AgentWebState;

use crate::services::{AgentDefinition, AgentType};

// ===== Chat Handler =====

pub async fn chat_handler(
    State(state): State<AgentWebState>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>, StatusCode> {
    let agent_id = req.agent_id.clone();
    let message = req.message.clone();

    // Get target agent
    let agent = state.agents_service.get_agent_by_id(&agent_id).await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Build messages for the LLM
    let system_prompt = if agent.is_orchestrator {
        OrchestratorService::build_orchestrator_prompt(&state.agents_service).await
    } else {
        agent.system_prompt.clone()
    };

    let messages = vec![
        crate::api::ChatMessage::system(system_prompt),
        crate::api::ChatMessage::user(message.clone()),
    ];

    // Clone what we need for the async stream
    let api_client = state.api_client.clone();
    let agent_name = agent.name.clone();
    let agent_id_clone = agent_id.clone();

    let stream = async_stream::stream! {
        // Send agent tag
        let tag_data = serde_json::json!({
            "agent_id": agent_id_clone,
            "agent_name": agent_name
        });
        yield Ok(Event::default()
            .event("agent_tag")
            .data(serde_json::to_string(&tag_data).unwrap_or_default()));

        // Call LLM stream
        match api_client.chat_stream(messages).await {
            Ok(response) => {
                use futures::StreamExt;
                let mut byte_stream = response.bytes_stream();
                let mut buffer = String::new();

                while let Some(chunk_result) = byte_stream.next().await {
                    match chunk_result {
                        Ok(bytes) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));

                            // Process complete SSE lines
                            while let Some(pos) = buffer.find('\n') {
                                let line = buffer[..pos].trim().to_string();
                                buffer = buffer[pos + 1..].to_string();

                                if line.is_empty() {
                                    continue;
                                }

                                if sse_utils::is_stream_done(&line) {
                                    break;
                                }

                                if let Some(content) = sse_utils::parse_sse_line(&line) {
                                    let text_data = serde_json::json!({
                                        "content": content
                                    });
                                    yield Ok(Event::default()
                                        .event("text")
                                        .data(serde_json::to_string(&text_data).unwrap_or_default()));
                                }
                            }
                        }
                        Err(e) => {
                            let err_data = serde_json::json!({
                                "message": format!("Stream error: {}", e)
                            });
                            yield Ok(Event::default()
                                .event("error")
                                .data(serde_json::to_string(&err_data).unwrap_or_default()));
                            break;
                        }
                    }
                }

                // Send done event
                yield Ok(Event::default()
                    .event("done")
                    .data("{}"));
            }
            Err(e) => {
                let err_data = serde_json::json!({
                    "message": format!("Failed to start stream: {}", e)
                });
                yield Ok(Event::default()
                    .event("error")
                    .data(serde_json::to_string(&err_data).unwrap_or_default()));
            }
        }
    };

    Ok(Sse::new(Box::pin(stream)))
}

// ===== Session Handlers =====

pub async fn create_session(
    State(state): State<AgentWebState>,
    Json(req): Json<CreateSessionRequest>,
) -> Json<ApiResult<SessionResponse>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let title = req.title.unwrap_or_else(|| "New Chat".to_string());

    Json(ApiResult::ok(SessionResponse {
        id: session_id,
        agent_id: req.agent_id,
        title,
        created_at: chrono::Utc::now().to_rfc3339(),
        messages: vec![],
    }))
}

pub async fn list_sessions(
    State(state): State<AgentWebState>,
) -> Json<ApiResult<Vec<SessionResponse>>> {
    let sessions = state.agents_service.list_sessions().await;
    let responses: Vec<SessionResponse> = sessions
        .iter()
        .map(|s| SessionResponse {
            id: s.id.clone(),
            agent_id: s.agent_id.clone(),
            title: format!("Session {}", &s.id[..8]),
            created_at: s.created_at.to_rfc3339(),
            messages: s.messages.iter().map(|m| MessageResponse {
                role: m.role.clone(),
                content: m.content.clone(),
                blocks: vec![MessageBlock::Text { content: m.content.clone() }],
                timestamp: m.timestamp.to_rfc3339(),
                agent_tag: None,
            }).collect(),
        })
        .collect();

    Json(ApiResult::ok(responses))
}

pub async fn get_session(
    State(state): State<AgentWebState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResult<SessionResponse>>, StatusCode> {
    let session = state.agents_service.get_session(&id).await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResult::ok(SessionResponse {
        id: session.id.clone(),
        agent_id: session.agent_id.clone(),
        title: format!("Session {}", &session.id[..8]),
        created_at: session.created_at.to_rfc3339(),
        messages: session.messages.iter().map(|m| MessageResponse {
            role: m.role.clone(),
            content: m.content.clone(),
            blocks: vec![MessageBlock::Text { content: m.content.clone() }],
            timestamp: m.timestamp.to_rfc3339(),
            agent_tag: None,
        }).collect(),
    })))
}

pub async fn delete_session(
    State(state): State<AgentWebState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResult<String>>, StatusCode> {
    state.agents_service.cancel_session(&id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResult::ok(format!("Session {} deleted", id))))
}

// ===== Agent CRUD Handlers =====

pub async fn list_agents(
    State(state): State<AgentWebState>,
) -> Json<ApiResult<Vec<AgentResponse>>> {
    let agents = state.agents_service.list_agents().await;
    let responses: Vec<AgentResponse> = agents
        .iter()
        .map(|a| AgentResponse {
            agent_id: a.agent_id.clone(),
            name: a.name.clone(),
            description: a.description.clone(),
            when_to_use: a.when_to_use.clone(),
            tools: a.tools.clone(),
            model: a.model.clone(),
            source: a.source.clone(),
            is_orchestrator: a.is_orchestrator,
        })
        .collect();

    Json(ApiResult::ok(responses))
}

pub async fn get_agent(
    State(state): State<AgentWebState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResult<AgentResponse>>, StatusCode> {
    let agent = state.agents_service.get_agent_by_id(&id).await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResult::ok(AgentResponse {
        agent_id: agent.agent_id,
        name: agent.name,
        description: agent.description,
        when_to_use: agent.when_to_use,
        tools: agent.tools,
        model: agent.model,
        source: agent.source,
        is_orchestrator: agent.is_orchestrator,
    })))
}

pub async fn create_agent(
    State(state): State<AgentWebState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<ApiResult<AgentResponse>>, StatusCode> {
    let definition = AgentDefinition {
        agent_id: String::new(), // Will be generated
        agent_type: AgentType::Custom,
        name: req.name.clone(),
        description: req.description.clone(),
        when_to_use: req.when_to_use.clone(),
        tools: req.tools.clone(),
        model: req.model.clone(),
        system_prompt: req.system_prompt.clone(),
        source: "custom".to_string(),
        base_dir: "custom".to_string(),
        is_orchestrator: false,
    };

    state.agents_service.register_custom_agent(definition).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return the created agent info
    Ok(Json(ApiResult::ok(AgentResponse {
        agent_id: format!("custom-{}", "pending"), // ID generated internally
        name: req.name,
        description: req.description,
        when_to_use: req.when_to_use,
        tools: req.tools,
        model: req.model,
        source: "custom".to_string(),
        is_orchestrator: false,
    })))
}

pub async fn update_agent(
    State(state): State<AgentWebState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAgentRequest>,
) -> Result<Json<ApiResult<String>>, StatusCode> {
    // Get existing agent
    let existing = state.agents_service.get_agent_by_id(&id).await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if built-in
    if existing.source == "built-in" {
        return Ok(Json(ApiResult::err("Cannot modify built-in agent")));
    }

    let updated = AgentDefinition {
        agent_id: id.clone(),
        agent_type: AgentType::Custom,
        name: req.name.unwrap_or(existing.name),
        description: req.description.unwrap_or(existing.description),
        when_to_use: req.when_to_use.unwrap_or(existing.when_to_use),
        tools: req.tools.unwrap_or(existing.tools),
        model: req.model.unwrap_or(existing.model),
        system_prompt: req.system_prompt.unwrap_or(existing.system_prompt),
        source: "custom".to_string(),
        base_dir: existing.base_dir,
        is_orchestrator: false,
    };

    state.agents_service.update_custom_agent(&id, updated).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResult::ok(format!("Agent {} updated", id))))
}

pub async fn delete_agent(
    State(state): State<AgentWebState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResult<String>>, StatusCode> {
    // Check if built-in
    if let Some(agent) = state.agents_service.get_agent_by_id(&id).await {
        if agent.source == "built-in" {
            return Ok(Json(ApiResult::err("Cannot delete built-in agent")));
        }
    }

    state.agents_service.delete_custom_agent(&id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResult::ok(format!("Agent {} deleted", id))))
}

// ===== Tools Handler =====

pub async fn list_tools(
    State(state): State<AgentWebState>,
) -> Json<ApiResult<Vec<ToolResponse>>> {
    let tools = state.tool_registry.list();
    let responses: Vec<ToolResponse> = tools
        .iter()
        .map(|t| ToolResponse {
            name: t.name().to_string(),
            description: t.description().to_string(),
        })
        .collect();

    Json(ApiResult::ok(responses))
}

// ===== Health Check =====

pub async fn health_check() -> Json<ApiResult<serde_json::Value>> {
    Json(ApiResult::ok(serde_json::json!({
        "status": "healthy",
        "service": "agent-api",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

// ===== Native Callback =====

pub async fn native_callback(
    State(_state): State<AgentWebState>,
    Json(req): Json<NativeCallbackRequest>,
) -> Json<ApiResult<NativeCallbackResponse>> {
    // Log the callback for now; in production, inject result back into Agent context
    println!("📲 Native callback received: {} (success={})", req.callback_id, req.success);

    Json(ApiResult::ok(NativeCallbackResponse {
        received: true,
        callback_id: req.callback_id,
    }))
}

// ===== WebSocket Handler (Skeleton) =====

pub async fn ws_handler() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "WebSocket endpoint - coming soon")
}
