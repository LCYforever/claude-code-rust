import { MessageBlock } from '../types';

const API_BASE = '/api/agent';

export interface SSECallbacks {
  onText?: (content: string) => void;
  onRichText?: (data: Record<string, unknown>) => void;
  onTaskProgress?: (data: Record<string, unknown>) => void;
  onStockCard?: (data: Record<string, unknown>) => void;
  onTable?: (data: Record<string, unknown>) => void;
  onChart?: (data: Record<string, unknown>) => void;
  onActionLink?: (data: Record<string, unknown>) => void;
  onImage?: (data: Record<string, unknown>) => void;
  onQuickReplies?: (data: Record<string, unknown>) => void;
  onNavigate?: (data: Record<string, unknown>) => void;
  onNativeCall?: (data: Record<string, unknown>) => void;
  onAgentTag?: (data: { agent_id: string; agent_name: string }) => void;
  onDelegation?: (data: Record<string, unknown>) => void;
  onDelegationResult?: (data: Record<string, unknown>) => void;
  onToolCallStart?: (data: Record<string, unknown>) => void;
  onToolCallEnd?: (data: Record<string, unknown>) => void;
  onDone?: () => void;
  onError?: (error: string) => void;
}

export async function streamChat(
  message: string,
  agentId: string = 'builtin-orchestrator',
  sessionId?: string,
  callbacks: SSECallbacks = {},
): Promise<void> {
  const response = await fetch(`${API_BASE}/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      agent_id: agentId,
      message,
      session_id: sessionId,
    }),
  });

  if (!response.ok) {
    callbacks.onError?.(`HTTP error: ${response.status}`);
    return;
  }

  const reader = response.body?.getReader();
  if (!reader) {
    callbacks.onError?.('No response body');
    return;
  }

  const decoder = new TextDecoder();
  let buffer = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });

    // Process SSE events line by line
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';

    let currentEvent = '';
    let currentData = '';

    for (const line of lines) {
      if (line.startsWith('event: ')) {
        currentEvent = line.substring(7).trim();
      } else if (line.startsWith('data: ')) {
        currentData = line.substring(6).trim();
      } else if (line === '' && currentEvent && currentData) {
        // Process complete event
        processSSEEvent(currentEvent, currentData, callbacks);
        currentEvent = '';
        currentData = '';
      }
    }
  }
}

function processSSEEvent(
  event: string,
  data: string,
  callbacks: SSECallbacks,
) {
  let parsed: Record<string, unknown> = {};
  try {
    parsed = JSON.parse(data);
  } catch {
    // If data is not JSON, wrap it
    parsed = { raw: data };
  }

  switch (event) {
    case 'text':
      callbacks.onText?.(parsed.content as string || '');
      break;
    case 'rich_text':
      callbacks.onRichText?.(parsed);
      break;
    case 'task_progress':
      callbacks.onTaskProgress?.(parsed);
      break;
    case 'stock_card':
      callbacks.onStockCard?.(parsed);
      break;
    case 'table':
      callbacks.onTable?.(parsed);
      break;
    case 'chart':
      callbacks.onChart?.(parsed);
      break;
    case 'action_link':
      callbacks.onActionLink?.(parsed);
      break;
    case 'image':
      callbacks.onImage?.(parsed);
      break;
    case 'quick_replies':
      callbacks.onQuickReplies?.(parsed);
      break;
    case 'navigate':
      callbacks.onNavigate?.(parsed);
      break;
    case 'native_call':
      callbacks.onNativeCall?.(parsed);
      break;
    case 'agent_tag':
      callbacks.onAgentTag?.(parsed as { agent_id: string; agent_name: string });
      break;
    case 'delegation':
      callbacks.onDelegation?.(parsed);
      break;
    case 'delegation_result':
      callbacks.onDelegationResult?.(parsed);
      break;
    case 'tool_call_start':
      callbacks.onToolCallStart?.(parsed);
      break;
    case 'tool_call_end':
      callbacks.onToolCallEnd?.(parsed);
      break;
    case 'done':
      callbacks.onDone?.();
      break;
    case 'error':
      callbacks.onError?.(parsed.message as string || 'Unknown error');
      break;
  }
}
