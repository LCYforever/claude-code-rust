// ===== MessageBlock Types =====

export interface RichTextNode {
  type: string;
  content: string;
  url?: string;
  style?: string;
}

export interface TaskStep {
  label: string;
  status: string;
  detail?: string;
}

export interface StockCardData {
  name: string;
  code: string;
  price: number;
  change: number;
  change_percent: number;
  volume?: number;
  chart_data?: ChartPoint[];
}

export interface ChartPoint {
  time: string;
  value: number;
}

export interface ActionPayload {
  type: string;
  url?: string;
  data?: Record<string, unknown>;
}

export interface QuickReply {
  label: string;
  value: string;
  icon?: string;
}

export type DelegationStatus = 'pending' | 'running' | 'completed' | 'failed';
export type NativeCallStatus = 'pending' | 'executing' | 'success' | 'failed';

export type MessageBlock =
  | { type: 'text'; content: string }
  | { type: 'rich_text'; nodes: RichTextNode[] }
  | { type: 'task_progress'; steps: TaskStep[]; collapsed: boolean }
  | { type: 'stock_card'; data: StockCardData }
  | { type: 'table'; headers: string[]; rows: string[][]; title?: string }
  | { type: 'chart'; chart_type: string; data: unknown; title?: string }
  | { type: 'action_link'; label: string; icon?: string; action: ActionPayload }
  | { type: 'image'; url: string; alt?: string }
  | { type: 'quick_replies'; options: QuickReply[] }
  | { type: 'navigate'; target: string; params?: Record<string, unknown>; label?: string }
  | { type: 'native_call'; action: string; method: string;
      params?: Record<string, unknown>; callback_id?: string;
      auto_execute?: boolean; label?: string;
      status?: NativeCallStatus; result?: unknown }
  | { type: 'delegation'; target_agent_id: string; target_agent_name: string;
      task: string; status: DelegationStatus };

// ===== Agent Event Types (SSE) =====

export interface AgentEvent {
  event: string;
  data: Record<string, unknown>;
}

// ===== Message & Session =====

export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  blocks: MessageBlock[];
  timestamp: string;
  agentTag?: string;
  isStreaming?: boolean;
}

export interface Session {
  id: string;
  agentId: string;
  title: string;
  createdAt: string;
  messages: Message[];
}

// ===== Agent =====

export interface AgentDefinition {
  agent_id: string;
  name: string;
  description: string;
  when_to_use: string;
  tools: string[];
  model: string;
  source: string;
  is_orchestrator: boolean;
}

// ===== API Key Config =====

export interface ApiKeyConfig {
  id: string;
  name: string;
  api_key: string;
  base_url: string;
  default_model: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

// ===== API Response =====

export interface ApiResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// ===== Native Bridge =====

export interface NativeBridge {
  call(method: string, params?: Record<string, unknown>): Promise<unknown>;
}

declare global {
  interface Window {
    NativeBridge?: NativeBridge;
  }
}
