//! Agent API Sub-module
//!
//! Provides the Agent Web API for the intelligent assistant system.
//! Includes routes, handlers, models, SSE streaming, WebSocket skeleton,
//! and Orchestrator service.

pub mod state;
pub mod models;
pub mod handlers;
pub mod router;
pub mod sse;
pub mod ws;
pub mod orchestrator;

pub use state::AgentWebState;
pub use router::agent_router;
