//! Agent Web State
//!
//! Shared state for the Agent API endpoints.

use std::sync::Arc;

use crate::api::ApiClient;
use crate::services::AgentsService;
use crate::tools::ToolRegistry;

use super::ws::ConnectionManager;

/// Shared state for Agent API handlers
#[derive(Clone)]
pub struct AgentWebState {
    pub agents_service: Arc<AgentsService>,
    pub tool_registry: Arc<ToolRegistry>,
    pub api_client: Arc<ApiClient>,
    pub connection_manager: Option<Arc<ConnectionManager>>,
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
        }
    }
}
