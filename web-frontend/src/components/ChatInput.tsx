import React, { useState, useRef, useEffect } from 'react';
import { Send, Loader2 } from 'lucide-react';

interface ChatInputProps {
  onSend: (message: string) => void;
  isStreaming: boolean;
}

const ChatInput: React.FC<ChatInputProps> = ({ onSend, isStreaming }) => {
  const [input, setInput] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSend = () => {
    const trimmed = input.trim();
    if (!trimmed || isStreaming) return;
    onSend(trimmed);
    setInput('');
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 120) + 'px';
    }
  }, [input]);

  return (
    <div className="border-t border-bg-tertiary bg-bg-secondary/80 backdrop-blur-sm px-4 py-3">
      <div className="flex items-end gap-3 max-w-4xl mx-auto">
        <textarea
          ref={textareaRef}
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="输入你的问题..."
          rows={1}
          className="flex-1 bg-bg-tertiary/50 border border-bg-tertiary rounded-xl px-4 py-2.5 text-sm text-text-primary placeholder-text-muted resize-none focus:outline-none focus:border-primary/50 transition-colors"
          disabled={isStreaming}
        />
        <button
          onClick={handleSend}
          disabled={!input.trim() || isStreaming}
          className={`flex-shrink-0 w-10 h-10 rounded-xl flex items-center justify-center transition-all cursor-pointer ${
            input.trim() && !isStreaming
              ? 'bg-gradient-to-br from-primary to-primary-dark text-white hover:opacity-90'
              : 'bg-bg-tertiary text-text-muted cursor-not-allowed'
          }`}
        >
          {isStreaming ? <Loader2 className="w-4 h-4 animate-spin" /> : <Send className="w-4 h-4" />}
        </button>
      </div>
      <div className="text-center mt-1">
        <span className="text-xs text-text-muted">Shift + Enter 换行</span>
      </div>
    </div>
  );
};

export default ChatInput;
