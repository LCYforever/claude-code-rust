import React from 'react';
import { Plus, Trash2, MessageSquare } from 'lucide-react';

interface SessionSidebarProps {
  sessions: { id: string; title: string; createdAt: string }[];
  currentSessionId: string | null;
  onSelectSession: (id: string) => void;
  onNewChat: () => void;
  onDeleteSession: (id: string) => void;
}

const SessionSidebar: React.FC<SessionSidebarProps> = ({
  sessions,
  currentSessionId,
  onSelectSession,
  onNewChat,
  onDeleteSession,
}) => {
  return (
    <aside className="w-72 flex-shrink-0 bg-bg-secondary border-r border-bg-tertiary flex flex-col">
      {/* New Chat Button */}
      <div className="p-3">
        <button
          onClick={onNewChat}
          className="w-full py-2.5 rounded-lg bg-gradient-to-r from-primary to-primary-dark text-white text-sm font-medium hover:opacity-90 transition-opacity flex items-center justify-center gap-2 cursor-pointer"
        >
          <Plus className="w-4 h-4" />
          New Chat
        </button>
      </div>

      {/* Session List */}
      <div className="flex-1 overflow-y-auto px-3 space-y-1">
        <div className="text-xs text-text-muted uppercase tracking-wider px-2 py-2">最近对话</div>
        {sessions.length === 0 ? (
          <div className="text-center py-8 text-text-muted text-sm">
            暂无对话记录
          </div>
        ) : (
          sessions.map(session => (
            <div
              key={session.id}
              onClick={() => onSelectSession(session.id)}
              className={`group px-3 py-2.5 rounded-lg cursor-pointer transition-all flex items-center justify-between ${
                currentSessionId === session.id
                  ? 'bg-bg-tertiary/50 border-l-3 border-primary'
                  : 'hover:bg-bg-tertiary/30'
              }`}
            >
              <div className="flex items-center gap-2 min-w-0">
                <MessageSquare className="w-4 h-4 text-text-muted flex-shrink-0" />
                <div className="min-w-0">
                  <div className="text-sm text-text-primary truncate">{session.title}</div>
                  <div className="text-xs text-text-muted mt-0.5">
                    {new Date(session.createdAt).toLocaleDateString('zh-CN')}
                  </div>
                </div>
              </div>
              <button
                onClick={(e) => { e.stopPropagation(); onDeleteSession(session.id); }}
                className="opacity-0 group-hover:opacity-100 p-1 hover:bg-bg-tertiary rounded transition-all cursor-pointer"
              >
                <Trash2 className="w-3.5 h-3.5 text-text-muted hover:text-danger" />
              </button>
            </div>
          ))
        )}
      </div>

      {/* Sidebar Footer */}
      <div className="p-3 border-t border-bg-tertiary">
        <div className="text-xs text-text-muted">
          <span>{sessions.length} 个对话</span>
        </div>
      </div>
    </aside>
  );
};

export default SessionSidebar;
