//! SSE Stream Processing
//!
//! Parses LLM response streams and pushes structured AgentEvent events.

use super::models::*;

/// Parse an SSE line from LLM stream response into text content
pub fn parse_sse_line(line: &str) -> Option<String> {
    if line.starts_with("data: ") {
        let data = &line[6..];
        if data == "[DONE]" {
            return None;
        }
        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
            if let Some(content) = chunk
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("delta"))
                .and_then(|d| d.get("content"))
                .and_then(|c| c.as_str())
            {
                return Some(content.to_string());
            }
            // Check for tool_call in delta
            if let Some(_tool_calls) = chunk
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("delta"))
                .and_then(|d| d.get("tool_calls"))
            {
                // Tool call detected — will be handled by the caller
                return None;
            }
        }
    }
    None
}

/// Check if an SSE line indicates the stream is done
pub fn is_stream_done(line: &str) -> bool {
    line.starts_with("data: [DONE]")
}

/// Check if an SSE line contains a tool call
pub fn extract_tool_call(line: &str) -> Option<serde_json::Value> {
    if line.starts_with("data: ") {
        let data = &line[6..];
        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
            if let Some(tool_calls) = chunk
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("delta"))
                .and_then(|d| d.get("tool_calls"))
            {
                return Some(tool_calls.clone());
            }
        }
    }
    None
}

/// Create a text SSE event string
pub fn make_sse_event(event_type: &str, data: &serde_json::Value) -> String {
    format!("event: {}\ndata: {}\n\n", event_type, serde_json::to_string(data).unwrap_or_default())
}

/// Create a done SSE event
pub fn make_done_event() -> String {
    "event: done\ndata: {}\n\n".to_string()
}

/// Create an error SSE event
pub fn make_error_event(message: &str) -> String {
    let data = serde_json::json!({
        "message": message
    });
    format!("event: error\ndata: {}\n\n", serde_json::to_string(&data).unwrap_or_default())
}

/// Create a text chunk SSE event
pub fn make_text_event(content: &str) -> String {
    let data = serde_json::json!({
        "content": content
    });
    make_sse_event("text", &data)
}

/// Create a delegation SSE event
pub fn make_delegation_event(
    target_agent_id: &str,
    target_agent_name: &str,
    task: &str,
    status: &str,
) -> String {
    let data = serde_json::json!({
        "target_agent_id": target_agent_id,
        "target_agent_name": target_agent_name,
        "task": task,
        "status": status,
    });
    make_sse_event("delegation", &data)
}

/// Create a delegation_result SSE event
pub fn make_delegation_result_event(
    agent_id: &str,
    agent_name: &str,
    task: &str,
    result: &str,
    success: bool,
) -> String {
    let data = serde_json::json!({
        "agent_id": agent_id,
        "agent_name": agent_name,
        "task": task,
        "result": result,
        "success": success,
    });
    make_sse_event("delegation_result", &data)
}

/// Create a native_call SSE event
pub fn make_native_call_event(
    action: &str,
    method: &str,
    params: Option<&serde_json::Value>,
    callback_id: Option<&str>,
    auto_execute: Option<bool>,
    label: Option<&str>,
) -> String {
    let mut data = serde_json::json!({
        "action": action,
        "method": method,
    });
    if let Some(p) = params {
        data["params"] = p.clone();
    }
    if let Some(cb) = callback_id {
        data["callback_id"] = serde_json::json!(cb);
    }
    if let Some(ae) = auto_execute {
        data["auto_execute"] = serde_json::json!(ae);
    }
    if let Some(l) = label {
        data["label"] = serde_json::json!(l);
    }
    make_sse_event("native_call", &data)
}

/// Create an agent_tag SSE event
pub fn make_agent_tag_event(agent_id: &str, agent_name: &str) -> String {
    let data = serde_json::json!({
        "agent_id": agent_id,
        "agent_name": agent_name,
    });
    make_sse_event("agent_tag", &data)
}
