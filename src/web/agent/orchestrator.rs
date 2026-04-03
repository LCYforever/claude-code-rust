//! Orchestrator Service
//!
//! Builds dynamic system prompts for the Orchestrator Agent
//! and manages delegation execution.
//!
//! The Orchestrator uses a two-step approach:
//! 1. Intent Recognition: A non-streaming LLM call to determine which agent to route to
//! 2. Delegation: Stream the user's request using the selected agent's system prompt

use crate::services::AgentsService;

/// Orchestrator service for dynamic prompt building and intent routing
pub struct OrchestratorService;

/// The result of Orchestrator intent recognition
#[derive(Debug, Clone)]
pub struct IntentResult {
    /// The selected agent_id to delegate to
    pub agent_id: String,
    /// Brief reason for the selection
    pub reason: String,
}

impl OrchestratorService {
    /// Build the intent-recognition prompt for the Orchestrator.
    /// This prompt instructs the LLM to return ONLY a JSON object with agent_id and reason.
    pub async fn build_intent_prompt(agents_service: &AgentsService) -> String {
        let agents = agents_service.list_non_orchestrator_agents().await;

        let mut agent_descriptions = String::new();
        for agent in &agents {
            agent_descriptions.push_str(&format!(
                "- agent_id: \"{}\"\n  名称: {}\n  描述: {}\n  适用场景: {}\n\n",
                agent.agent_id, agent.name, agent.description, agent.when_to_use
            ));
        }

        format!(
            r#"你是一个智能意图分析引擎。你的唯一任务是分析用户的请求，从下方的 Agent 列表中选择最合适的一个来处理。

## 可用 Agent 列表

{agent_list}

## 规则

1. 分析用户消息的核心意图
2. 根据各 Agent 的"适用场景"，选择最匹配的 agent_id
3. 如果用户的请求是一般性的聊天、问答、闲聊、写作、翻译、数学计算、知识问答、需要联网搜索最新信息、或者不明确适合哪个专业 Agent，就选择 "builtin-general-purpose"（该 Agent 具有联网搜索能力）
4. 你必须且只能输出一个 JSON 对象，不要输出其他任何内容

## 输出格式（严格遵守，不要添加任何其他文字）

{{"agent_id": "选中的agent_id", "reason": "简短的选择理由"}}"#,
            agent_list = agent_descriptions
        )
    }

    /// Parse the LLM's intent recognition response into an IntentResult.
    /// Attempts to extract JSON from the response text.
    pub fn parse_intent_response(response_text: &str) -> Option<IntentResult> {
        let text = response_text.trim();

        // Try to parse the entire response as JSON
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
            if let (Some(agent_id), reason) = (
                v.get("agent_id").and_then(|a| a.as_str()),
                v.get("reason").and_then(|r| r.as_str()).unwrap_or(""),
            ) {
                return Some(IntentResult {
                    agent_id: agent_id.to_string(),
                    reason: reason.to_string(),
                });
            }
        }

        // Try to find JSON in the response (LLM might add extra text around it)
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                let json_str = &text[start..=end];
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let (Some(agent_id), reason) = (
                        v.get("agent_id").and_then(|a| a.as_str()),
                        v.get("reason").and_then(|r| r.as_str()).unwrap_or(""),
                    ) {
                        return Some(IntentResult {
                            agent_id: agent_id.to_string(),
                            reason: reason.to_string(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Build the full Orchestrator system prompt (legacy, for direct chat without delegation)
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
