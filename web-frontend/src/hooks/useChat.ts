import { useState, useCallback, useRef } from 'react';
import { Message, MessageBlock } from '../types';
import { streamChat, SSECallbacks } from '../api/sse';
import { sendNativeCallback } from '../api/client';

// Generate a unique session ID for multi-turn conversation context
function generateSessionId(): string {
  return `session-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

export function useChat(agentId: string = 'builtin-orchestrator') {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const abortRef = useRef<AbortController | null>(null);
  // Persistent session ID for multi-turn conversation context management
  const sessionIdRef = useRef<string>(generateSessionId());

  const sendMessage = useCallback(async (content: string, sessionId?: string) => {
    // Use the provided sessionId or the persistent one for multi-turn context
    const activeSessionId = sessionId || sessionIdRef.current;
    // Add user message
    const userMsg: Message = {
      id: `msg-${Date.now()}-user`,
      role: 'user',
      content,
      blocks: [{ type: 'text', content }],
      timestamp: new Date().toISOString(),
    };

    // Add placeholder assistant message
    const assistantId = `msg-${Date.now()}-assistant`;
    const assistantMsg: Message = {
      id: assistantId,
      role: 'assistant',
      content: '',
      blocks: [],
      timestamp: new Date().toISOString(),
      isStreaming: true,
    };

    setMessages(prev => [...prev, userMsg, assistantMsg]);
    setIsStreaming(true);

    let textContent = '';

    const callbacks: SSECallbacks = {
      onText: (chunk) => {
        textContent += chunk;
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, content: textContent, blocks: [{ type: 'text' as const, content: textContent }] }
            : m
        ));
      },

      onAgentTag: (data) => {
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, agentTag: data.agent_name }
            : m
        ));
      },

      onDelegation: (data) => {
        const block: MessageBlock = {
          type: 'delegation',
          target_agent_id: data.target_agent_id as string,
          target_agent_name: data.target_agent_name as string,
          task: data.task as string,
          status: (data.status as 'pending' | 'running' | 'completed' | 'failed') || 'running',
        };
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, blocks: [...m.blocks, block] }
            : m
        ));
      },

      onDelegationResult: (data) => {
        textContent += `\n\n${data.result as string || ''}`;
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, content: textContent, blocks: [{ type: 'text' as const, content: textContent }] }
            : m
        ));
      },

      onNativeCall: async (data) => {
        const block: MessageBlock = {
          type: 'native_call',
          action: data.action as string,
          method: data.method as string,
          params: data.params as Record<string, unknown> | undefined,
          callback_id: data.callback_id as string | undefined,
          auto_execute: data.auto_execute as boolean | undefined,
          label: data.label as string | undefined,
          status: 'pending',
        };
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, blocks: [...m.blocks, block] }
            : m
        ));

        // Auto-execute if configured
        if (data.auto_execute && window.NativeBridge) {
          try {
            const result = await window.NativeBridge.call(
              data.method as string,
              data.params as Record<string, unknown> | undefined,
            );
            if (data.callback_id) {
              await sendNativeCallback(
                data.callback_id as string,
                activeSessionId,
                result,
                true,
              );
            }
          } catch (err) {
            console.error('Native call failed:', err);
            if (data.callback_id) {
              await sendNativeCallback(
                data.callback_id as string,
                activeSessionId,
                { error: String(err) },
                false,
              );
            }
          }
        }
      },

      onTaskProgress: (data) => {
        const block: MessageBlock = {
          type: 'task_progress',
          steps: (data.steps as { label: string; status: string; detail?: string }[]) || [],
          collapsed: (data.collapsed as boolean) ?? false,
        };
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, blocks: [...m.blocks, block] }
            : m
        ));
      },

      onQuickReplies: (data) => {
        const block: MessageBlock = {
          type: 'quick_replies',
          options: (data.options as { label: string; value: string; icon?: string }[]) || [],
        };
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, blocks: [...m.blocks, block] }
            : m
        ));
      },

      onDone: () => {
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, isStreaming: false }
            : m
        ));
        setIsStreaming(false);
      },

      onError: (error) => {
        console.error('SSE error:', error);
        setMessages(prev => prev.map(m =>
          m.id === assistantId
            ? { ...m, isStreaming: false, content: textContent || `Error: ${error}` }
            : m
        ));
        setIsStreaming(false);
      },
    };

    await streamChat(content, agentId, activeSessionId, callbacks);
  }, [agentId]);

  const clearMessages = useCallback(() => {
    setMessages([]);
    // Reset session ID to start a fresh conversation context
    sessionIdRef.current = generateSessionId();
  }, []);

  return {
    messages,
    isStreaming,
    sendMessage,
    clearMessages,
    // Expose sessionId for external use if needed
    sessionId: sessionIdRef.current,
  };
}
