import React, { useRef, useEffect } from 'react';
import { useChat } from '../hooks/useChat';
import MessageBubble from './MessageBubble';
import ChatInput from './ChatInput';
import QuickActionBar from './QuickActionBar';
import { Bot } from 'lucide-react';

interface ChatPanelProps {
  agentId: string;
  sessionId?: string;
}

const ChatPanel: React.FC<ChatPanelProps> = ({ agentId, sessionId }) => {
  const { messages, isStreaming, sendMessage } = useChat(agentId);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = (content: string) => {
    sendMessage(content, sessionId);
  };

  const handleQuickReply = (value: string) => {
    sendMessage(value, sessionId);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto px-4 py-4">
        {messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <div className="w-16 h-16 rounded-full bg-gradient-to-br from-primary/20 to-primary-dark/20 flex items-center justify-center mb-4">
              <Bot className="w-8 h-8 text-primary-light" />
            </div>
            <h2 className="text-xl font-semibold text-text-primary mb-2">Agent Hub</h2>
            <p className="text-text-muted text-sm max-w-md">
              你好！我是 AI 智能助手。我可以帮你分析代码、制定计划、解答问题。输入你的问题开始对话吧。
            </p>
          </div>
        ) : (
          <div className="max-w-4xl mx-auto">
            {messages.map(msg => (
              <MessageBubble
                key={msg.id}
                message={msg}
                onQuickReply={handleQuickReply}
              />
            ))}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      {/* Quick Action Bar */}
      <QuickActionBar onAction={handleSend} />

      {/* Input Area */}
      <ChatInput onSend={handleSend} isStreaming={isStreaming} />
    </div>
  );
};

export default ChatPanel;
