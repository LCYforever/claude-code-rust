//! Delegate To Agent Tool
//!
//! Allows the Orchestrator to delegate tasks to sub-agents.
//! Implements the Tool trait for integration with ToolRegistry.

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::ApiClient;
use crate::services::AgentsService;
use crate::tools::{Tool, ToolError, ToolOutput};

pub struct DelegateToAgentTool {
    agents_service: Arc<AgentsService>,
    api_client: Arc<ApiClient>,
}

impl DelegateToAgentTool {
    pub fn new(agents_service: Arc<AgentsService>, api_client: Arc<ApiClient>) -> Self {
        Self {
            agents_service,
            api_client,
        }
    }
}

#[async_trait]
impl Tool for DelegateToAgentTool {
    fn name(&self) -> &str {
        "delegate_to_agent"
    }

    fn description(&self) -> &str {
        "委托指定的子Agent执行任务。传入目标agent_id和任务描述，子Agent将执行并返回结果。"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "目标Agent的ID，如 builtin-explore, builtin-plan 等"
                },
                "task": {
                    "type": "string",
                    "description": "需要委托执行的任务描述"
                }
            },
            "required": ["agent_id", "task"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let agent_id = input.get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing required field: agent_id".to_string(),
                code: Some("invalid_input".to_string()),
            })?;

        let task = input.get("task")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing required field: task".to_string(),
                code: Some("invalid_input".to_string()),
            })?;

        // Get the target agent definition
        let agent = self.agents_service.get_agent_by_id(agent_id).await
            .ok_or_else(|| ToolError {
                message: format!("Agent not found: {}", agent_id),
                code: Some("agent_not_found".to_string()),
            })?;

        // Prevent delegating to orchestrator itself
        if agent.is_orchestrator {
            return Err(ToolError {
                message: "Cannot delegate to orchestrator itself".to_string(),
                code: Some("invalid_delegation".to_string()),
            });
        }

        println!("📤 Delegating to agent: {} ({})", agent.name, agent_id);

        // Build messages for the sub-agent
        let messages = vec![
            crate::api::ChatMessage {
                role: "system".to_string(),
                content: agent.system_prompt.clone(),
                tool_calls: None,
            },
            crate::api::ChatMessage {
                role: "user".to_string(),
                content: task.to_string(),
                tool_calls: None,
            },
        ];

        // Execute via ApiClient (single-round call for now)
        let response = self.api_client.chat(messages).await
            .map_err(|e| ToolError {
                message: format!("Failed to execute agent: {}", e),
                code: Some("execution_error".to_string()),
            })?;

        let result = response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        println!("📥 Agent {} completed delegation", agent.name);

        let mut metadata = HashMap::new();
        metadata.insert("agent_id".to_string(), json!(agent_id));
        metadata.insert("agent_name".to_string(), json!(agent.name));
        metadata.insert("task".to_string(), json!(task));

        Ok(ToolOutput {
            output_type: "delegation_result".to_string(),
            content: result,
            metadata,
        })
    }
}
