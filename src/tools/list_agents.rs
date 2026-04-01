//! List Available Agents Tool
//!
//! Allows the Orchestrator to dynamically query all available agents.
//! Returns agent_id, name, description, and when_to_use for each agent.

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use crate::services::AgentsService;
use crate::tools::{Tool, ToolError, ToolOutput};

pub struct ListAvailableAgentsTool {
    agents_service: Arc<AgentsService>,
}

impl ListAvailableAgentsTool {
    pub fn new(agents_service: Arc<AgentsService>) -> Self {
        Self { agents_service }
    }
}

#[async_trait]
impl Tool for ListAvailableAgentsTool {
    fn name(&self) -> &str {
        "list_available_agents"
    }

    fn description(&self) -> &str {
        "获取所有可用的Agent列表，包含每个Agent的ID、名称、描述和适用场景。"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(&self, _input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let agents = self.agents_service.list_non_orchestrator_agents().await;

        let agent_list: Vec<serde_json::Value> = agents
            .iter()
            .map(|a| {
                json!({
                    "agent_id": a.agent_id,
                    "name": a.name,
                    "description": a.description,
                    "when_to_use": a.when_to_use,
                    "tools": a.tools,
                })
            })
            .collect();

        let result = json!({
            "agents": agent_list,
            "count": agent_list.len()
        });

        Ok(ToolOutput {
            output_type: "agent_list".to_string(),
            content: serde_json::to_string_pretty(&result).unwrap_or_default(),
            metadata: HashMap::new(),
        })
    }
}
