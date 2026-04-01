import React from 'react';
import { Message } from '../types';
import BlockRenderer from './BlockRenderer';
import StreamingIndicator from './StreamingIndicator';
import { Bot, User } from 'lucide-react';

interface MessageBubbleProps {
  message: Message;
  onQuickReply?: (value: string) => void;
}

const MessageBubble: React.FC<MessageBubbleProps> = ({ message, onQuickReply }) => {
  const isUser = message.role === 'user';

  return (
    <div className={`flex gap-3 mb-4 animate-fade-in ${isUser ? 'flex-row-reverse' : ''}`}>
      {/* Avatar */}
      <div className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center ${
        isUser
          ? 'bg-gradient-to-br from-primary to-primary-dark'
          : 'bg-bg-tertiary border border-bg-tertiary'
      }`}>
        {isUser ? <User className="w-4 h-4 text-white" /> : <Bot className="w-4 h-4 text-primary-light" />}
      </div>

      {/* Content */}
      <div className={`max-w-[75%] ${isUser ? 'items-end' : 'items-start'}`}>
        {/* Agent tag */}
        {!isUser && message.agentTag && (
          <div className="flex items-center gap-1 mb-1">
            <span className="text-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary-light border border-primary/20">
              {message.agentTag}
            </span>
          </div>
        )}

        {/* Message bubble */}
        <div className={`rounded-2xl px-4 py-3 ${
          isUser
            ? 'bg-gradient-to-br from-primary to-primary-dark text-white'
            : 'bg-bg-secondary border border-bg-tertiary'
        }`}>
          {message.blocks.map((block, idx) => (
            <BlockRenderer key={idx} block={block} onQuickReply={onQuickReply} />
          ))}
          {message.isStreaming && <StreamingIndicator />}
        </div>

        {/* Timestamp */}
        <div className={`text-xs text-text-muted mt-1 ${isUser ? 'text-right' : ''}`}>
          {new Date(message.timestamp).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })}
        </div>
      </div>
    </div>
  );
};

export default MessageBubble;
