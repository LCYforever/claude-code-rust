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
use super::web_search::WebSearchService;

use crate::services::{AgentDefinition, AgentType};

// ===== Chat Handler =====

pub async fn chat_handler(
    State(state): State<AgentWebState>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>, StatusCode> {
    let agent_id = req.agent_id.clone();
    let message = req.message.clone();

    // Use session_id for multi-turn context; auto-generate if not provided
    let session_id = req.session_id.clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Get target agent
    let agent = state.agents_service.get_agent_by_id(&agent_id).await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Try to get active API key from user-configured keys
    let active_key = {
        let keys = state.agents_service.list_api_keys().await;
        keys.into_iter().find(|k| k.is_active)
    };

    // Get or create full session context (ContextManager + HistorySnipManager)
    let session_ctx = state.get_or_create_session_context(&session_id).await;
    let context_mgr = session_ctx.context_mgr.clone();
    let snip_mgr = session_ctx.snip_mgr.clone();
    let compact_service = state.compact_service.clone();

    // Add the current user message to the context window
    context_mgr.add_user(&message).await;

    let stats = context_mgr.stats().await;
    println!(
        "📊 Context[{}]: {} entries, {}/{} tokens ({:.1}% used)",
        &session_id[..8.min(session_id.len())],
        stats.total_entries, stats.total_tokens, stats.max_tokens,
        stats.utilization * 100.0
    );

    // Clone what we need for the async stream
    let api_client = state.api_client.clone();
    let agents_service = state.agents_service.clone();
    let context_mgr_clone = context_mgr.clone();
    let snip_mgr_clone = snip_mgr.clone();
    let compact_service_clone = compact_service.clone();

    if agent.is_orchestrator {
        // === Orchestrator two-step routing ===
        // Step 1: Intent recognition (non-streaming)
        // Step 2: Delegate to the selected agent (streaming)
        let stream = async_stream::stream! {
            // Send orchestrator agent tag first
            let tag_data = serde_json::json!({
                "agent_id": "builtin-orchestrator",
                "agent_name": "Orchestrator"
            });
            yield Ok(Event::default()
                .event("agent_tag")
                .data(serde_json::to_string(&tag_data).unwrap_or_default()));

            // Step 1: Intent recognition
            let intent_prompt = OrchestratorService::build_intent_prompt(&agents_service).await;
            let intent_messages = vec![
                crate::api::ChatMessage::system(intent_prompt),
                crate::api::ChatMessage::user(message.clone()),
            ];

            println!("🧠 Orchestrator: analyzing intent for message: {}", &message);

            let intent_result = if let Some(ref key_config) = active_key {
                api_client.chat_with_key(
                    intent_messages,
                    &key_config.api_key,
                    &key_config.base_url,
                    &key_config.default_model,
                ).await
            } else {
                api_client.chat(intent_messages).await
            };

            let (target_agent_id, _reason) = match intent_result {
                Ok(response) => {
                    let response_text = response.choices.first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default();

                    println!("🧠 Orchestrator raw intent response: {}", response_text);

                    match OrchestratorService::parse_intent_response(&response_text) {
                        Some(intent) => {
                            println!("🎯 Orchestrator routed to: {} (reason: {})", intent.agent_id, intent.reason);
                            (intent.agent_id, intent.reason)
                        }
                        None => {
                            println!("⚠️ Orchestrator failed to parse intent, falling back to general-purpose");
                            ("builtin-general-purpose".to_string(), "无法解析意图，使用通用 Agent".to_string())
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Orchestrator intent recognition failed: {}, falling back to general-purpose", e);
                    ("builtin-general-purpose".to_string(), format!("意图识别失败: {}", e))
                }
            };

            // Get the target agent (with fallback to general-purpose)
            let target_agent = match agents_service.get_agent_by_id(&target_agent_id).await {
                Some(agent) => agent,
                None => {
                    println!("⚠️ Target agent {} not found, falling back to general-purpose", target_agent_id);
                    agents_service.get_agent_by_id("builtin-general-purpose").await
                        .expect("builtin-general-purpose must exist")
                }
            };

            // Send delegation event to frontend
            let delegation_data = serde_json::json!({
                "target_agent_id": target_agent.agent_id,
                "target_agent_name": target_agent.name,
                "task": message,
                "status": "running"
            });
            yield Ok(Event::default()
                .event("delegation")
                .data(serde_json::to_string(&delegation_data).unwrap_or_default()));

            // Update agent tag to show the actual responding agent
            let new_tag_data = serde_json::json!({
                "agent_id": target_agent.agent_id,
                "agent_name": target_agent.name
            });
            yield Ok(Event::default()
                .event("agent_tag")
                .data(serde_json::to_string(&new_tag_data).unwrap_or_default()));

            // Step 2: If delegating to general-purpose, check if web search is needed
            let search_context = if target_agent.agent_id == "builtin-general-purpose" {
                // Check search intent
                let search_intent_prompt = WebSearchService::build_search_intent_prompt();
                let search_intent_messages = vec![
                    crate::api::ChatMessage::system(search_intent_prompt),
                    crate::api::ChatMessage::user(message.clone()),
                ];

                println!("🔍 Orchestrator→GP: checking if web search is needed");

                let search_intent_result = if let Some(ref key_config) = active_key {
                    api_client.chat_with_key(
                        search_intent_messages,
                        &key_config.api_key,
                        &key_config.base_url,
                        &key_config.default_model,
                    ).await
                } else {
                    api_client.chat(search_intent_messages).await
                };

                let search_intent = match search_intent_result {
                    Ok(response) => {
                        let text = response.choices.first()
                            .map(|c| c.message.content.clone())
                            .unwrap_or_default();
                        WebSearchService::parse_search_intent(&text)
                    }
                    Err(_) => None,
                };

                if let Some(ref intent) = search_intent {
                    if intent.needs_search && !intent.queries.is_empty() {
                        println!("🌐 Orchestrator→GP: executing web search: {:?}", intent.queries);

                        let tool_start_data = serde_json::json!({
                            "tool_name": "web_search",
                            "input": { "queries": intent.queries }
                        });
                        yield Ok(Event::default()
                            .event("tool_call_start")
                            .data(serde_json::to_string(&tool_start_data).unwrap_or_default()));

                        let search_service = WebSearchService::new();
                        let search_results = search_service.multi_search(&intent.queries).await;
                        let context = WebSearchService::format_search_results_as_context(&search_results);
                        let result_count: usize = search_results.iter().map(|r| r.results.len()).sum();

                        let tool_end_data = serde_json::json!({
                            "tool_name": "web_search",
                            "output": format!("找到 {} 条搜索结果", result_count)
                        });
                        yield Ok(Event::default()
                            .event("tool_call_end")
                            .data(serde_json::to_string(&tool_end_data).unwrap_or_default()));

                        Some(context)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Step 3: Stream the response using the target agent's system prompt (with search context if available)
            // Build messages with full conversation history from ContextManager
            let final_system_prompt = if let Some(ref ctx) = search_context {
                format!(
                    "{}\n\n{}\n\n注意：你拥有联网搜索能力。以上搜索结果来自实时互联网搜索。请结合搜索结果为用户提供最新、最准确的回答。",
                    target_agent.system_prompt,
                    ctx
                )
            } else {
                target_agent.system_prompt.clone()
            };

            // Build messages: system prompt + conversation history from ContextManager
            let mut target_messages = vec![
                crate::api::ChatMessage::system(final_system_prompt),
            ];
            // Get history entries, apply HistorySnip + CompactService, then convert to messages
            let raw_entries = context_mgr_clone.get_entries().await;
            let snipped_entries = snip_mgr_clone.snip_if_needed(&raw_entries).await;
            let (compacted_entries, compact_result) = compact_service_clone.compact(&snipped_entries);
            if compact_result.tokens_before != compact_result.tokens_after {
                println!(
                    "🗜️  Compact[Orchestrator]: {:?} applied, {} -> {} tokens ({:.0}% ratio)",
                    compact_result.level_applied,
                    compact_result.tokens_before, compact_result.tokens_after,
                    compact_result.compression_ratio * 100.0
                );
            }
            let history_messages: Vec<crate::api::ChatMessage> = compacted_entries.iter()
                .map(|e| crate::api::ChatMessage {
                    role: e.role.clone(),
                    content: e.content.clone(),
                    tool_calls: None,
                })
                .collect();
            target_messages.extend(history_messages);

            let stream_result = if let Some(ref key_config) = active_key {
                api_client.chat_stream_with_key(
                    target_messages,
                    &key_config.api_key,
                    &key_config.base_url,
                    &key_config.default_model,
                ).await
            } else {
                api_client.chat_stream(target_messages).await
            };

            match stream_result {
                Ok(response) => {
                    use futures::StreamExt;
                    let mut byte_stream = response.bytes_stream();
                    let mut buffer = String::new();
                    let mut full_response = String::new();

                    while let Some(chunk_result) = byte_stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));

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
                                        full_response.push_str(&content);
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

                    // Save assistant response to context window for multi-turn memory
                    if !full_response.is_empty() {
                        context_mgr_clone.add_assistant(&full_response).await;
                        println!("💾 Saved assistant response to context ({} chars)", full_response.len());
                    }

                    // Send delegation result event
                    let result_data = serde_json::json!({
                        "target_agent_id": target_agent.agent_id,
                        "target_agent_name": target_agent.name,
                        "task": message,
                        "status": "completed"
                    });
                    yield Ok(Event::default()
                        .event("delegation")
                        .data(serde_json::to_string(&result_data).unwrap_or_default()));

                    yield Ok(Event::default()
                        .event("done")
                        .data("{}"));
                }
                Err(e) => {
                    let err_data = serde_json::json!({
                        "message": format!("Failed to start delegation stream: {}", e)
                    });
                    yield Ok(Event::default()
                        .event("error")
                        .data(serde_json::to_string(&err_data).unwrap_or_default()));
                }
            }
        };

        Ok(Sse::new(Box::pin(stream)))
    } else if agent.agent_id == "builtin-general-purpose" {
        // === General Purpose Agent with Web Search ===
        // Three-step flow:
        // 1. Determine if web search is needed (non-streaming LLM call)
        // 2. Execute web search if needed
        // 3. Stream the final response with search context
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

            // Step 1: Determine if web search is needed
            let search_intent_prompt = WebSearchService::build_search_intent_prompt();
            let intent_messages = vec![
                crate::api::ChatMessage::system(search_intent_prompt),
                crate::api::ChatMessage::user(message.clone()),
            ];

            println!("🔍 General Purpose: checking if web search is needed for: {}", &message);

            let intent_result = if let Some(ref key_config) = active_key {
                api_client.chat_with_key(
                    intent_messages,
                    &key_config.api_key,
                    &key_config.base_url,
                    &key_config.default_model,
                ).await
            } else {
                api_client.chat(intent_messages).await
            };

            let search_intent = match intent_result {
                Ok(response) => {
                    let response_text = response.choices.first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default();
                    println!("🔍 Search intent response: {}", response_text);
                    WebSearchService::parse_search_intent(&response_text)
                }
                Err(e) => {
                    println!("⚠️ Search intent check failed: {}", e);
                    None
                }
            };

            // Step 2: Execute search if needed
            let search_context = if let Some(ref intent) = search_intent {
                if intent.needs_search && !intent.queries.is_empty() {
                    println!("🌐 Web search needed! Queries: {:?}", intent.queries);

                    // Send a tool_call_start event to frontend
                    let tool_start_data = serde_json::json!({
                        "tool_name": "web_search",
                        "input": { "queries": intent.queries }
                    });
                    yield Ok(Event::default()
                        .event("tool_call_start")
                        .data(serde_json::to_string(&tool_start_data).unwrap_or_default()));

                    // Execute searches
                    let search_service = WebSearchService::new();
                    let search_results = search_service.multi_search(&intent.queries).await;
                    let context = WebSearchService::format_search_results_as_context(&search_results);

                    let result_count: usize = search_results.iter()
                        .map(|r| r.results.len())
                        .sum();

                    // Send tool_call_end event
                    let tool_end_data = serde_json::json!({
                        "tool_name": "web_search",
                        "output": format!("找到 {} 条搜索结果", result_count)
                    });
                    yield Ok(Event::default()
                        .event("tool_call_end")
                        .data(serde_json::to_string(&tool_end_data).unwrap_or_default()));

                    println!("✅ Web search complete: {} results", result_count);
                    Some(context)
                } else {
                    println!("ℹ️ No web search needed: {}", intent.reason);
                    None
                }
            } else {
                println!("ℹ️ Could not determine search intent, proceeding without search");
                None
            };

            // Step 3: Build messages with search context and stream response
            let system_prompt = if let Some(ref ctx) = search_context {
                format!(
                    "{}\n\n{}\n\n注意：你拥有联网搜索能力。以上搜索结果来自实时互联网搜索。请结合搜索结果为用户提供最新、最准确的回答。",
                    agent.system_prompt,
                    ctx
                )
            } else {
                agent.system_prompt.clone()
            };

            // Build messages: system prompt + conversation history from ContextManager
            let mut messages = vec![
                crate::api::ChatMessage::system(system_prompt),
            ];
            // Get history entries, apply HistorySnip + CompactService, then convert to messages
            let raw_entries = context_mgr_clone.get_entries().await;
            let snipped_entries = snip_mgr_clone.snip_if_needed(&raw_entries).await;
            let (compacted_entries, compact_result) = compact_service_clone.compact(&snipped_entries);
            if compact_result.tokens_before != compact_result.tokens_after {
                println!(
                    "🗜️  Compact[GP]: {:?} applied, {} -> {} tokens ({:.0}% ratio)",
                    compact_result.level_applied,
                    compact_result.tokens_before, compact_result.tokens_after,
                    compact_result.compression_ratio * 100.0
                );
            }
            let history_messages: Vec<crate::api::ChatMessage> = compacted_entries.iter()
                .map(|e| crate::api::ChatMessage {
                    role: e.role.clone(),
                    content: e.content.clone(),
                    tool_calls: None,
                })
                .collect();
            messages.extend(history_messages);

            // Call LLM stream (General Purpose Agent)
            let stream_result = if let Some(ref key_config) = active_key {
                api_client.chat_stream_with_key(
                    messages,
                    &key_config.api_key,
                    &key_config.base_url,
                    &key_config.default_model,
                ).await
            } else {
                api_client.chat_stream(messages).await
            };

            match stream_result {
                Ok(response) => {
                    use futures::StreamExt;
                    let mut byte_stream = response.bytes_stream();
                    let mut buffer = String::new();
                    let mut full_response = String::new();

                    while let Some(chunk_result) = byte_stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));

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
                                        full_response.push_str(&content);
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

                    // Save assistant response to context window for multi-turn memory
                    if !full_response.is_empty() {
                        context_mgr_clone.add_assistant(&full_response).await;
                        println!("💾 Saved GP assistant response to context ({} chars)", full_response.len());
                    }

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
    } else {
        // === Direct agent chat (non-Orchestrator, non-GeneralPurpose) ===
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

            // Build messages: system prompt + conversation history from ContextManager
            let mut messages = vec![
                crate::api::ChatMessage::system(agent.system_prompt.clone()),
            ];
            // Get history entries, apply HistorySnip + CompactService, then convert to messages
            let raw_entries = context_mgr_clone.get_entries().await;
            let snipped_entries = snip_mgr_clone.snip_if_needed(&raw_entries).await;
            let (compacted_entries, compact_result) = compact_service_clone.compact(&snipped_entries);
            if compact_result.tokens_before != compact_result.tokens_after {
                println!(
                    "🗜️  Compact[Direct]: {:?} applied, {} -> {} tokens ({:.0}% ratio)",
                    compact_result.level_applied,
                    compact_result.tokens_before, compact_result.tokens_after,
                    compact_result.compression_ratio * 100.0
                );
            }
            let history_messages: Vec<crate::api::ChatMessage> = compacted_entries.iter()
                .map(|e| crate::api::ChatMessage {
                    role: e.role.clone(),
                    content: e.content.clone(),
                    tool_calls: None,
                })
                .collect();
            messages.extend(history_messages);

            // Call LLM stream (Direct Agent)
            let stream_result = if let Some(ref key_config) = active_key {
                api_client.chat_stream_with_key(
                    messages,
                    &key_config.api_key,
                    &key_config.base_url,
                    &key_config.default_model,
                ).await
            } else {
                api_client.chat_stream(messages).await
            };

            match stream_result {
                Ok(response) => {
                    use futures::StreamExt;
                    let mut byte_stream = response.bytes_stream();
                    let mut buffer = String::new();
                    let mut full_response = String::new();

                    while let Some(chunk_result) = byte_stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));

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
                                        full_response.push_str(&content);
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

                    // Save assistant response to context window for multi-turn memory
                    if !full_response.is_empty() {
                        context_mgr_clone.add_assistant(&full_response).await;
                        println!("💾 Saved direct agent response to context ({} chars)", full_response.len());
                    }

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

// ===== API Key Management =====

pub async fn list_api_keys(
    State(state): State<AgentWebState>,
) -> Json<ApiResult<Vec<ApiKeyConfig>>> {
    let keys = state.agents_service.list_api_keys().await;
    Json(ApiResult::ok(keys))
}

pub async fn get_api_key(
    State(state): State<AgentWebState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResult<ApiKeyConfig>>, StatusCode> {
    let key = state.agents_service.get_api_key(&id).await
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(ApiResult::ok(key)))
}

pub async fn create_api_key(
    State(state): State<AgentWebState>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Json<ApiResult<ApiKeyConfig>> {
    let now = chrono::Utc::now().to_rfc3339();
    let key_config = ApiKeyConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name,
        api_key: req.api_key,
        base_url: req.base_url,
        default_model: req.default_model,
        is_active: true,
        created_at: now.clone(),
        updated_at: now,
    };

    state.agents_service.save_api_key(key_config.clone()).await;
    Json(ApiResult::ok(key_config))
}

pub async fn update_api_key(
    State(state): State<AgentWebState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateApiKeyRequest>,
) -> Result<Json<ApiResult<ApiKeyConfig>>, StatusCode> {
    let mut existing = state.agents_service.get_api_key(&id).await
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = req.name { existing.name = name; }
    if let Some(api_key) = req.api_key { existing.api_key = api_key; }
    if let Some(base_url) = req.base_url { existing.base_url = base_url; }
    if let Some(default_model) = req.default_model { existing.default_model = default_model; }
    if let Some(is_active) = req.is_active { existing.is_active = is_active; }
    existing.updated_at = chrono::Utc::now().to_rfc3339();

    state.agents_service.save_api_key(existing.clone()).await;
    Ok(Json(ApiResult::ok(existing)))
}

pub async fn delete_api_key(
    State(state): State<AgentWebState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResult<String>>, StatusCode> {
    state.agents_service.delete_api_key(&id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(ApiResult::ok(format!("API Key {} deleted", id))))
}

// ===== WebSocket Handler (Skeleton) =====

pub async fn ws_handler() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "WebSocket endpoint - coming soon")
}
