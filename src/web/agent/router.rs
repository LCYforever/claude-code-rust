//! Agent API Router
//!
//! Defines all routes for the Agent API sub-module.

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use super::handlers;
use super::state::AgentWebState;

/// Create the Agent API router
/// All routes are relative and should be nested under /api/agent
pub fn agent_router() -> Router<AgentWebState> {
    Router::new()
        // Chat
        .route("/chat", post(handlers::chat_handler))
        // Sessions
        .route("/sessions", post(handlers::create_session))
        .route("/sessions", get(handlers::list_sessions))
        .route("/sessions/:id", get(handlers::get_session))
        .route("/sessions/:id", delete(handlers::delete_session))
        // Agents
        .route("/agents", get(handlers::list_agents))
        .route("/agents/:id", get(handlers::get_agent))
        .route("/agents", post(handlers::create_agent))
        .route("/agents/:id", put(handlers::update_agent))
        .route("/agents/:id", delete(handlers::delete_agent))
        // Tools
        .route("/tools", get(handlers::list_tools))
        // Health
        .route("/health", get(handlers::health_check))
        // Native callback
        .route("/native-callback", post(handlers::native_callback))
        // WebSocket (skeleton)
        .route("/ws", get(handlers::ws_handler))
}
