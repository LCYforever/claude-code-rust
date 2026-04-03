//! Agents Service - Built-in agent system
//!
//! Built-in agents for various tasks including:
//! - builtin-orchestrator: Orchestrator agent for automatic routing
//! - claudeCodeGuideAgent: Claude Code guidance
//! - exploreAgent: Codebase exploration
//! - generalPurposeAgent: General purpose tasks
//! - planAgent: Planning and task breakdown
//! - verificationAgent: Verification and testing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;
use crate::web::agent::models::ApiKeyConfig;

/// Get the default API keys storage file path: ~/.claude-code/api_keys.json
fn default_api_keys_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".claude-code").join("api_keys.json")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    Orchestrator,
    ClaudeCodeGuide,
    Explore,
    GeneralPurpose,
    Plan,
    Verification,
    Custom,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Orchestrator => write!(f, "orchestrator"),
            AgentType::ClaudeCodeGuide => write!(f, "claude-code-guide"),
            AgentType::Explore => write!(f, "explore"),
            AgentType::GeneralPurpose => write!(f, "general-purpose"),
            AgentType::Plan => write!(f, "plan"),
            AgentType::Verification => write!(f, "verification"),
            AgentType::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    #[serde(default)]
    pub agent_id: String,
    pub agent_type: AgentType,
    pub name: String,
    pub description: String,
    pub when_to_use: String,
    pub tools: Vec<String>,
    pub model: String,
    pub system_prompt: String,
    pub source: String,
    pub base_dir: String,
    #[serde(default)]
    pub is_orchestrator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub agent_type: AgentType,
    pub agent_id: String,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<AgentMessage>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatusReport {
    pub available_agents: Vec<AgentDefinition>,
    pub active_sessions: usize,
    pub sessions: Vec<AgentSession>,
}

pub struct AgentsService {
    state: Arc<RwLock<AppState>>,
    agents: Arc<RwLock<HashMap<String, AgentDefinition>>>,
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
    api_keys: Arc<RwLock<HashMap<String, ApiKeyConfig>>>,
    api_keys_path: PathBuf,
}

impl AgentsService {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        let agents = Self::load_builtin_agents();
        let api_keys_path = default_api_keys_path();

        // Load persisted API keys from file at startup
        let api_keys = Self::load_api_keys_from_file(&api_keys_path);

        Self {
            state,
            agents: Arc::new(RwLock::new(agents)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            api_keys: Arc::new(RwLock::new(api_keys)),
            api_keys_path,
        }
    }

    fn load_builtin_agents() -> HashMap<String, AgentDefinition> {
        let mut agents = HashMap::new();

        // Orchestrator Agent - the main routing agent
        agents.insert(
            "builtin-orchestrator".to_string(),
            AgentDefinition {
                agent_id: "builtin-orchestrator".to_string(),
                agent_type: AgentType::Orchestrator,
                name: "Orchestrator".to_string(),
                description: "智能调度中心，自动分析用户意图并委托给最合适的子 Agent 执行任务".to_string(),
                when_to_use: "Default entry point. Automatically analyzes user intent and delegates to the best sub-agent.".to_string(),
                tools: vec!["delegate_to_agent".to_string(), "list_available_agents".to_string()],
                model: "sonnet".to_string(),
                system_prompt: String::new(), // Dynamic prompt built at runtime by OrchestratorService
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
                is_orchestrator: true,
            },
        );

        agents.insert(
            "builtin-claude-code-guide".to_string(),
            AgentDefinition {
                agent_id: "builtin-claude-code-guide".to_string(),
                agent_type: AgentType::ClaudeCodeGuide,
                name: "Claude Code Guide".to_string(),
                description: "Guides users through Claude Code features and best practices".to_string(),
                when_to_use: "When you need help understanding Claude Code features, commands, or workflows".to_string(),
                tools: vec!["file_read".to_string(), "search".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are a Claude Code Guide agent. Your role is to help users understand and effectively use Claude Code.

Key responsibilities:
1. Explain Claude Code features and capabilities
2. Guide users through common workflows
3. Provide best practices and tips
4. Help troubleshoot issues

Be concise, helpful, and focus on practical guidance."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
                is_orchestrator: false,
            },
        );

        agents.insert(
            "builtin-explore".to_string(),
            AgentDefinition {
                agent_id: "builtin-explore".to_string(),
                agent_type: AgentType::Explore,
                name: "Explore Agent".to_string(),
                description: "Explores and analyzes codebases to understand structure and patterns".to_string(),
                when_to_use: "When you need to understand a codebase, find specific code, or analyze project structure".to_string(),
                tools: vec!["file_read".to_string(), "search".to_string(), "list_files".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are an Explore agent. Your role is to analyze and understand codebases.

Key responsibilities:
1. Explore project structure
2. Identify key files and patterns
3. Understand code organization
4. Find relevant code for tasks

Be thorough but efficient. Focus on providing useful insights about the codebase."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
                is_orchestrator: false,
            },
        );

        agents.insert(
            "builtin-general-purpose".to_string(),
            AgentDefinition {
                agent_id: "builtin-general-purpose".to_string(),
                agent_type: AgentType::GeneralPurpose,
                name: "General Purpose Agent".to_string(),
                description: "通用智能助手，支持联网搜索，处理各类问答、写作、翻译和知识查询".to_string(),
                when_to_use: "For general tasks that don't fit other specialized agents, including questions requiring web search for latest information".to_string(),
                tools: vec!["web_search".to_string(), "file_read".to_string(), "file_write".to_string(), "file_edit".to_string(), "search".to_string(), "execute_command".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"你是一个通用智能助手，具有联网搜索能力。你可以帮助用户处理各种任务。

## 核心能力
1. **联网搜索**：当需要最新信息时，系统会自动搜索互联网并将结果提供给你
2. **知识问答**：回答各领域的问题，结合已有知识和搜索结果
3. **写作创作**：撰写文章、翻译、润色文字
4. **数据分析**：分析数据、计算、总结

## 回答规范
- 始终使用中文回复
- 使用 Markdown 格式让回复更清晰
- 如果回答基于搜索结果，请自然地引用信息来源
- 如果信息可能不够准确或时效性有限，请主动说明
- 保持回复简洁、专业、有帮助

## 搜索结果使用
- 当系统提供了搜索结果时，优先使用搜索结果中的信息
- 将搜索结果与你的知识结合，给出全面的回答
- 适当标注信息来源（如提供了 URL）"#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
                is_orchestrator: false,
            },
        );

        agents.insert(
            "builtin-plan".to_string(),
            AgentDefinition {
                agent_id: "builtin-plan".to_string(),
                agent_type: AgentType::Plan,
                name: "Plan Agent".to_string(),
                description: "Creates detailed plans and breaks down complex tasks".to_string(),
                when_to_use: "When you need to plan a complex task or break down work into steps".to_string(),
                tools: vec!["file_read".to_string(), "search".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are a Plan agent. Your role is to create detailed plans for complex tasks.

Key responsibilities:
1. Analyze task requirements
2. Break down complex tasks into steps
3. Identify dependencies and risks
4. Create actionable plans

Be thorough and structured. Focus on creating clear, executable plans."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
                is_orchestrator: false,
            },
        );

        agents.insert(
            "builtin-verification".to_string(),
            AgentDefinition {
                agent_id: "builtin-verification".to_string(),
                agent_type: AgentType::Verification,
                name: "Verification Agent".to_string(),
                description: "Verifies implementations and runs tests".to_string(),
                when_to_use: "When you need to verify code works correctly or run tests".to_string(),
                tools: vec!["file_read".to_string(), "execute_command".to_string(), "search".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are a Verification agent. Your role is to verify implementations and ensure quality.

Key responsibilities:
1. Run tests and analyze results
2. Verify code correctness
3. Check for edge cases
4. Report issues clearly

Be thorough and systematic. Focus on finding and reporting issues."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
                is_orchestrator: false,
            },
        );

        agents
    }

    // ===== Query Methods =====

    pub async fn list_agents(&self) -> Vec<AgentDefinition> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get agent by agent_id (new primary lookup)
    pub async fn get_agent_by_id(&self, agent_id: &str) -> Option<AgentDefinition> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Legacy: get agent by AgentType (maps to first matching agent_id)
    pub async fn get_agent(&self, agent_type: &AgentType) -> Option<AgentDefinition> {
        let agents = self.agents.read().await;
        agents.values().find(|a| &a.agent_type == agent_type).cloned()
    }

    /// List all agents except Orchestrator (for Orchestrator prompt building)
    pub async fn list_non_orchestrator_agents(&self) -> Vec<AgentDefinition> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.agent_type != AgentType::Orchestrator)
            .cloned()
            .collect()
    }

    // ===== Agent Execution =====

    pub async fn run_agent(&self, agent_type: &AgentType, prompt: &str) -> anyhow::Result<AgentSession> {
        let agent = self.get_agent(agent_type).await
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {:?}", agent_type))?;

        self.run_agent_by_id(&agent.agent_id, prompt).await
    }

    pub async fn run_agent_by_id(&self, agent_id: &str, prompt: &str) -> anyhow::Result<AgentSession> {
        let agent = self.get_agent_by_id(agent_id).await
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let session = AgentSession {
            id: session_id.clone(),
            agent_type: agent.agent_type.clone(),
            agent_id: agent.agent_id.clone(),
            status: AgentStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: vec![AgentMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                timestamp: Utc::now(),
            }],
            result: None,
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }

        println!("🤖 Running agent: {} ({})", agent.name, session_id);

        let result = self.execute_agent(&agent, prompt).await?;

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = AgentStatus::Completed;
            session.result = Some(result.clone());
            session.updated_at = Utc::now();
            session.messages.push(AgentMessage {
                role: "assistant".to_string(),
                content: result,
                timestamp: Utc::now(),
            });
            
            return Ok(session.clone());
        }

        Err(anyhow::anyhow!("Session not found after execution"))
    }

    async fn execute_agent(&self, agent: &AgentDefinition, prompt: &str) -> anyhow::Result<String> {
        let state = self.state.read().await;
        let api_client = crate::api::ApiClient::new(state.settings.clone());

        let messages = vec![
            crate::api::ChatMessage {
                role: "system".to_string(),
                content: agent.system_prompt.clone(),
                tool_calls: None,
            },
            crate::api::ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                tool_calls: None,
            },
        ];

        let response = api_client.chat(messages).await?;
        
        if let Some(choice) = response.choices.first() {
            return Ok(choice.message.content.clone());
        }

        Ok(String::new())
    }

    // ===== Session Management =====

    pub async fn get_session(&self, session_id: &str) -> Option<AgentSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn list_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn cancel_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = AgentStatus::Failed;
            session.updated_at = Utc::now();
            println!("🚫 Session cancelled: {}", session_id);
        }

        Ok(())
    }

    pub async fn get_status(&self) -> AgentStatusReport {
        let agents = self.agents.read().await;
        let sessions = self.sessions.read().await;
        
        let active_sessions = sessions
            .values()
            .filter(|s| s.status == AgentStatus::Running)
            .count();

        AgentStatusReport {
            available_agents: agents.values().cloned().collect(),
            active_sessions,
            sessions: sessions.values().cloned().collect(),
        }
    }

    // ===== Custom Agent CRUD =====

    pub async fn register_custom_agent(&self, mut definition: AgentDefinition) -> anyhow::Result<()> {
        // Generate agent_id if empty
        if definition.agent_id.is_empty() {
            definition.agent_id = format!("custom-{}", uuid::Uuid::new_v4());
        }
        definition.agent_type = AgentType::Custom;
        definition.source = "custom".to_string();

        let agent_id = definition.agent_id.clone();
        let mut agents = self.agents.write().await;
        agents.insert(agent_id, definition);
        println!("✅ Custom agent registered");
        Ok(())
    }

    pub async fn update_custom_agent(&self, agent_id: &str, mut definition: AgentDefinition) -> anyhow::Result<()> {
        let mut agents = self.agents.write().await;
        
        if let Some(existing) = agents.get(agent_id) {
            if existing.source == "built-in" {
                return Err(anyhow::anyhow!("Cannot modify built-in agent: {}", agent_id));
            }
        } else {
            return Err(anyhow::anyhow!("Agent not found: {}", agent_id));
        }

        definition.agent_id = agent_id.to_string();
        definition.agent_type = AgentType::Custom;
        agents.insert(agent_id.to_string(), definition);
        println!("✅ Custom agent updated: {}", agent_id);
        Ok(())
    }

    pub async fn delete_custom_agent(&self, agent_id: &str) -> anyhow::Result<()> {
        let mut agents = self.agents.write().await;
        
        if let Some(existing) = agents.get(agent_id) {
            if existing.source == "built-in" {
                return Err(anyhow::anyhow!("Cannot delete built-in agent: {}", agent_id));
            }
        } else {
            return Err(anyhow::anyhow!("Agent not found: {}", agent_id));
        }

        agents.remove(agent_id);
        println!("🗑️ Custom agent deleted: {}", agent_id);
        Ok(())
    }

    pub async fn save_agent_to_file(&self, agent_id: &str, dir: &PathBuf) -> anyhow::Result<()> {
        let agents = self.agents.read().await;
        let agent = agents.get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}.json", agent_id));
        let content = serde_json::to_string_pretty(agent)?;
        std::fs::write(&path, content)?;
        println!("💾 Agent saved to: {:?}", path);
        Ok(())
    }

    // ===== Loading from Directory =====

    pub async fn load_agents_from_dir(&self, dir: &PathBuf) -> anyhow::Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut agents = self.agents.write().await;
        
        let entries = std::fs::read_dir(dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(mut agent) = serde_json::from_str::<AgentDefinition>(&content) {
                        // Generate agent_id from filename if missing
                        if agent.agent_id.is_empty() {
                            agent.agent_id = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                        }
                        agents.insert(agent.agent_id.clone(), agent);
                    }
                }
            }
        }

        println!("📂 Loaded agents from: {:?}", dir);
        Ok(())
    }

    // ===== API Key Management (with file persistence) =====

    /// Load API keys from JSON file on disk
    fn load_api_keys_from_file(path: &PathBuf) -> HashMap<String, ApiKeyConfig> {
        if !path.exists() {
            return HashMap::new();
        }

        match std::fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<ApiKeyConfig>>(&content) {
                    Ok(keys) => {
                        let count = keys.len();
                        let map: HashMap<String, ApiKeyConfig> = keys
                            .into_iter()
                            .map(|k| (k.id.clone(), k))
                            .collect();
                        println!("🔑 Loaded {} API key(s) from {:?}", count, path);
                        map
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to parse API keys file {:?}: {}", path, e);
                        HashMap::new()
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ Failed to read API keys file {:?}: {}", path, e);
                HashMap::new()
            }
        }
    }

    /// Persist all API keys to JSON file on disk
    fn save_api_keys_to_file(keys: &HashMap<String, ApiKeyConfig>, path: &PathBuf) {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("⚠️ Failed to create API keys directory {:?}: {}", parent, e);
                return;
            }
        }

        let keys_vec: Vec<&ApiKeyConfig> = keys.values().collect();
        match serde_json::to_string_pretty(&keys_vec) {
            Ok(content) => {
                if let Err(e) = std::fs::write(path, content) {
                    eprintln!("⚠️ Failed to write API keys file {:?}: {}", path, e);
                } else {
                    println!("💾 API keys persisted to {:?}", path);
                }
            }
            Err(e) => {
                eprintln!("⚠️ Failed to serialize API keys: {}", e);
            }
        }
    }

    pub async fn list_api_keys(&self) -> Vec<ApiKeyConfig> {
        let keys = self.api_keys.read().await;
        keys.values().cloned().collect()
    }

    pub async fn get_api_key(&self, id: &str) -> Option<ApiKeyConfig> {
        let keys = self.api_keys.read().await;
        keys.get(id).cloned()
    }

    pub async fn save_api_key(&self, config: ApiKeyConfig) {
        let mut keys = self.api_keys.write().await;
        keys.insert(config.id.clone(), config);
        Self::save_api_keys_to_file(&keys, &self.api_keys_path);
    }

    pub async fn delete_api_key(&self, id: &str) -> anyhow::Result<()> {
        let mut keys = self.api_keys.write().await;
        keys.remove(id)
            .ok_or_else(|| anyhow::anyhow!("API Key not found: {}", id))?;
        Self::save_api_keys_to_file(&keys, &self.api_keys_path);
        Ok(())
    }
}
