//! Orchestrator Service
//!
//! Builds dynamic system prompts for the Orchestrator Agent
//! and manages delegation execution.

use crate::services::AgentsService;

/// Orchestrator service for dynamic prompt building
pub struct OrchestratorService;

impl OrchestratorService {
    /// Dynamically generate the Orchestrator's system prompt
    /// by iterating all non-Orchestrator agents and incorporating
    /// their name/description/when_to_use into the prompt.
    pub async fn build_orchestrator_prompt(agents_service: &AgentsService) -> String {
        let agents = agents_service.list_non_orchestrator_agents().await;

        let mut agent_descriptions = String::new();
        for agent in &agents {
            agent_descriptions.push_str(&format!(
                "- {} ({}): {}\n  适用场景: {}\n\n",
                agent.agent_id, agent.name, agent.description, agent.when_to_use
            ));
        }

        format!(
            r#"你是一个智能调度 Orchestrator Agent。你的职责是分析用户的自然语言请求，判断最合适的子 Agent 来处理任务，然后通过 delegate_to_agent 工具将任务委托给它。

## 可用 Agent 列表

{agent_list}

## 工作流程

1. **分析意图**：理解用户请求的核心需求
2. **选择 Agent**：根据各 Agent 的"适用场景"描述，选择最匹配的子 Agent
3. **委托执行**：使用 `delegate_to_agent` 工具，传入 agent_id 和 task 描述
4. **汇总结果**：收到子 Agent 的执行结果后，整理并以用户友好的方式返回

## 注意事项

- 如果用户请求不明确，可以先使用 `list_available_agents` 确认可用 Agent
- 如果没有合适的专业 Agent，委托给 `builtin-general-purpose`
- 你可以在一次回复中委托多个 Agent（串行执行）
- 保持回复简洁、专业、有帮助

## 输出格式

始终使用中文回复用户。使用 Markdown 格式让回复更清晰易读。"#,
            agent_list = agent_descriptions
        )
    }
}
