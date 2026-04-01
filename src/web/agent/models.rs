//! Agent API Models
//!
//! Structured message protocol: MessageBlock enum, AgentEvent SSE events,
//! ChatRequest, Agent CRUD request/response types, etc.

use serde::{Deserialize, Serialize};

// ===== MessageBlock =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageBlock {
    #[serde(rename = "text")]
    Text { content: String },

    #[serde(rename = "rich_text")]
    RichText { nodes: Vec<RichTextNode> },

    #[serde(rename = "task_progress")]
    TaskProgress {
        steps: Vec<TaskStep>,
        collapsed: bool,
    },

    #[serde(rename = "stock_card")]
    StockCard { data: StockCardData },

    #[serde(rename = "table")]
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },

    #[serde(rename = "chart")]
    Chart {
        chart_type: String,
        data: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },

    #[serde(rename = "action_link")]
    ActionLink {
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
        action: ActionPayload,
    },

    #[serde(rename = "image")]
    Image {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
    },

    #[serde(rename = "quick_replies")]
    QuickReplies { options: Vec<QuickReply> },

    #[serde(rename = "navigate")]
    Navigate {
        target: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },

    #[serde(rename = "native_call")]
    NativeCall {
        action: String,
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        callback_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        auto_execute: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },

    #[serde(rename = "delegation")]
    Delegation {
        target_agent_id: String,
        target_agent_name: String,
        task: String,
        status: DelegationStatus,
    },
}

// ===== Sub-types for MessageBlock =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub label: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockCardData {
    pub name: String,
    pub code: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_data: Option<Vec<ChartPoint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    pub time: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPayload {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickReply {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DelegationStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

// ===== AgentEvent (SSE events) =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AgentEvent {
    #[serde(rename = "text")]
    Text { data: TextEventData },

    #[serde(rename = "rich_text")]
    RichText { data: RichTextNode },

    #[serde(rename = "task_progress")]
    TaskProgress { data: TaskProgressData },

    #[serde(rename = "stock_card")]
    StockCard { data: StockCardData },

    #[serde(rename = "table")]
    Table { data: TableEventData },

    #[serde(rename = "chart")]
    ChartEvent { data: ChartEventData },

    #[serde(rename = "action_link")]
    ActionLink { data: ActionLinkData },

    #[serde(rename = "image")]
    ImageEvent { data: ImageEventData },

    #[serde(rename = "quick_replies")]
    QuickReplies { data: QuickRepliesData },

    #[serde(rename = "navigate")]
    Navigate { data: NavigateEventData },

    #[serde(rename = "native_call")]
    NativeCall { data: NativeCallEventData },

    #[serde(rename = "agent_tag")]
    AgentTag { data: AgentTagData },

    #[serde(rename = "delegation")]
    Delegation { data: DelegationEventData },

    #[serde(rename = "delegation_result")]
    DelegationResult { data: DelegationResultData },

    #[serde(rename = "tool_call_start")]
    ToolCallStart { data: ToolCallData },

    #[serde(rename = "tool_call_end")]
    ToolCallEnd { data: ToolCallData },

    #[serde(rename = "done")]
    Done,

    #[serde(rename = "error")]
    Error { data: ErrorEventData },
}

// ===== Event Data Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEventData {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressData {
    pub steps: Vec<TaskStep>,
    pub collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableEventData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartEventData {
    pub chart_type: String,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLinkData {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub action: ActionPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEventData {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickRepliesData {
    pub options: Vec<QuickReply>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateEventData {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeCallEventData {
    pub action: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_execute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTagData {
    pub agent_id: String,
    pub agent_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationEventData {
    pub target_agent_id: String,
    pub target_agent_name: String,
    pub task: String,
    pub status: DelegationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationResultData {
    pub agent_id: String,
    pub agent_name: String,
    pub task: String,
    pub result: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallData {
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEventData {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

// ===== Request / Response Types =====

#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    #[serde(default = "default_agent_id")]
    pub agent_id: String,
    pub message: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

fn default_agent_id() -> String {
    "builtin-orchestrator".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSessionRequest {
    #[serde(default = "default_agent_id")]
    pub agent_id: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub created_at: String,
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub role: String,
    pub content: String,
    pub blocks: Vec<MessageBlock>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub when_to_use: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default = "default_model")]
    pub model: String,
    pub system_prompt: String,
}

fn default_model() -> String {
    "sonnet".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when_to_use: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentResponse {
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub when_to_use: String,
    pub tools: Vec<String>,
    pub model: String,
    pub source: String,
    pub is_orchestrator: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolResponse {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResult<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

// ===== Native Callback =====

#[derive(Debug, Clone, Deserialize)]
pub struct NativeCallbackRequest {
    pub callback_id: String,
    pub session_id: String,
    pub result: serde_json::Value,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NativeCallbackResponse {
    pub received: bool,
    pub callback_id: String,
}
