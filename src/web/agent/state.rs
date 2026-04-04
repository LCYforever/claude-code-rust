//! Agent Web State
//!
//! Shared state for the Agent API endpoints.
//! Includes per-session ContextManager, HistorySnipManager, and CompactService
//! for comprehensive multi-turn conversation history management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::ApiClient;
use crate::memory::context::ContextManager;
use crate::memory::history_snip::{HistorySnipManager, HistorySnipConfig};
use crate::services::compact::CompactService;
use crate::services::AgentsService;
use crate::tools::ToolRegistry;

use super::ws::ConnectionManager;

/// Per-session context state: bundles ContextManager + HistorySnipManager
pub struct SessionContext {
    pub context_mgr: Arc<ContextManager>,
    pub snip_mgr: Arc<HistorySnipManager>,
}

/// Shared state for Agent API handlers
#[derive(Clone)]
pub struct AgentWebState {
    pub agents_service: Arc<AgentsService>,
    pub tool_registry: Arc<ToolRegistry>,
    pub api_client: Arc<ApiClient>,
    pub connection_manager: Option<Arc<ConnectionManager>>,
    /// Per-session context + snip managers for multi-turn conversation history.
    pub session_contexts: Arc<RwLock<HashMap<String, Arc<SessionContext>>>>,
    /// Shared compact service for three-tier compression (stateless, can be shared)
    pub compact_service: Arc<CompactService>,
}

impl AgentWebState {
    pub fn new(
        agents_service: Arc<AgentsService>,
        tool_registry: Arc<ToolRegistry>,
        api_client: Arc<ApiClient>,
    ) -> Self {
        Self {
            agents_service,
            tool_registry,
            api_client,
            connection_manager: Some(Arc::new(ConnectionManager::new())),
            session_contexts: Arc::new(RwLock::new(HashMap::new())),
            compact_service: Arc::new(CompactService::with_defaults()),
        }
    }

    /// Get or create a SessionContext (ContextManager + HistorySnipManager) for a session.
    pub async fn get_or_create_session_context(&self, session_id: &str) -> Arc<SessionContext> {
        // Fast path: read lock
        {
            let contexts = self.session_contexts.read().await;
            if let Some(ctx) = contexts.get(session_id) {
                return ctx.clone();
            }
        }
        // Slow path: write lock to create new context
        let mut contexts = self.session_contexts.write().await;
        // Double-check after acquiring write lock
        if let Some(ctx) = contexts.get(session_id) {
            return ctx.clone();
        }
        let session_ctx = Arc::new(SessionContext {
            context_mgr: Arc::new(ContextManager::with_max_tokens(128000)),
            snip_mgr: Arc::new(HistorySnipManager::new(HistorySnipConfig {
                enabled: true,
                ..Default::default()
            })),
        });
        contexts.insert(session_id.to_string(), session_ctx.clone());
        println!("📝 Created new context window + snip manager for session: {}", session_id);
        session_ctx
    }

    /// Backward-compatible: get or create just a ContextManager
    pub async fn get_or_create_context(&self, session_id: &str) -> Arc<ContextManager> {
        let session_ctx = self.get_or_create_session_context(session_id).await;
        session_ctx.context_mgr.clone()
    }
}
