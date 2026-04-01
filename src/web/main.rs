//! Web Server Main Entry Point - Plugin Marketplace & Agent API

use claude_code_rs::web::{WebServer, AgentWebState};
use claude_code_rs::api::ApiClient;
use claude_code_rs::config::Settings;
use claude_code_rs::services::AgentsService;
use claude_code_rs::tools::ToolRegistry;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    info!("Starting Claude Code Agent Web Server");
    info!("Server will be available at http://127.0.0.1:{}", port);

    // Initialize core services
    let settings = Settings::default();
    let app_state = Arc::new(RwLock::new(claude_code_rs::state::AppState::new(settings.clone())));
    
    // Initialize AgentsService
    let agents_service = Arc::new(AgentsService::new(app_state.clone()));
    
    // Initialize ApiClient
    let api_client = Arc::new(ApiClient::new(settings));
    
    // Initialize ToolRegistry with orchestrator tools
    let mut tool_registry = ToolRegistry::new();
    tool_registry.register(Box::new(
        claude_code_rs::tools::DelegateToAgentTool::new(
            agents_service.clone(),
            api_client.clone(),
        )
    ));
    tool_registry.register(Box::new(
        claude_code_rs::tools::ListAvailableAgentsTool::new(
            agents_service.clone(),
        )
    ));
    let tool_registry = Arc::new(tool_registry);

    // Build AgentWebState
    let agent_state = AgentWebState::new(
        agents_service,
        tool_registry,
        api_client,
    );

    // Start server with Agent API
    let server = WebServer::new(port)
        .with_agent_state(agent_state);
    
    server.run().await
}
