import React, { useState, useCallback } from 'react';
import ChatPanel from './components/ChatPanel';
import SessionSidebar from './components/SessionSidebar';
import AgentSelector from './components/AgentSelector';
import AgentManager from './components/AgentManager';
import ApiKeyManager from './components/ApiKeyManager';
import { useSessions } from './hooks/useSessions';
import { useAgents } from './hooks/useAgents';
import { useApiKeys } from './hooks/useApiKeys';
import { Zap, Wifi, Settings, Key } from 'lucide-react';

const App: React.FC = () => {
  const [currentAgentId, setCurrentAgentId] = useState('builtin-orchestrator');
  const [agentManagerOpen, setAgentManagerOpen] = useState(false);
  const [apiKeyManagerOpen, setApiKeyManagerOpen] = useState(false);

  const {
    sessions,
    currentSessionId,
    setCurrentSessionId,
    createNewSession,
    removeSession,
  } = useSessions();

  const {
    agents,
    addAgent,
    editAgent,
    removeAgent,
  } = useAgents();

  const {
    apiKeys,
    loading: apiKeysLoading,
    error: apiKeysError,
    addApiKey,
    editApiKey,
    removeApiKey,
    toggleActive,
  } = useApiKeys();

  const handleNewChat = useCallback(async () => {
    await createNewSession(currentAgentId, '新对话');
  }, [createNewSession, currentAgentId]);

  const handleSaveAgent = useCallback(async (data: {
    name: string;
    description: string;
    when_to_use: string;
    system_prompt: string;
    tools: string[];
    model: string;
  }) => {
    await addAgent(data);
    setAgentManagerOpen(false);
  }, [addAgent]);

  const handleDeleteAgent = useCallback(async (id: string) => {
    await removeAgent(id);
  }, [removeAgent]);

  const sidebarSessions = sessions.map(s => ({
    id: s.id,
    title: s.title || '新对话',
    createdAt: s.createdAt,
  }));

  return (
    <div className="h-screen flex flex-col bg-bg-primary text-text-primary overflow-hidden">
      {/* Top Navigation Bar */}
      <header className="h-14 flex-shrink-0 flex items-center justify-between px-4 bg-bg-secondary/80 backdrop-blur-sm gradient-border z-10">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-primary to-primary-dark flex items-center justify-center">
            <Zap className="w-4 h-4 text-white" />
          </div>
          <h1 className="text-lg font-bold gradient-text">Agent Hub</h1>
        </div>

        <div className="flex items-center gap-3">
          {/* Agent Selector */}
          <AgentSelector
            agents={agents}
            currentAgentId={currentAgentId}
            onSelect={setCurrentAgentId}
          />

          {/* API Key Manager button */}
          <button
            onClick={() => setApiKeyManagerOpen(true)}
            className="p-2 hover:bg-bg-tertiary rounded-lg transition-colors cursor-pointer"
            title="API Key 管理"
          >
            <Key className="w-4 h-4 text-text-muted hover:text-text-primary" />
          </button>

          {/* Agent Manager button */}
          <button
            onClick={() => setAgentManagerOpen(true)}
            className="p-2 hover:bg-bg-tertiary rounded-lg transition-colors cursor-pointer"
            title="管理 Agent"
          >
            <Settings className="w-4 h-4 text-text-muted hover:text-text-primary" />
          </button>

          {/* Connection status */}
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-success pulse-dot" />
            <Wifi className="w-4 h-4 text-text-muted" />
          </div>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Sidebar */}
        <SessionSidebar
          sessions={sidebarSessions}
          currentSessionId={currentSessionId}
          onSelectSession={setCurrentSessionId}
          onNewChat={handleNewChat}
          onDeleteSession={removeSession}
        />

        {/* Chat Area */}
        <main className="flex-1 flex flex-col min-w-0">
          <ChatPanel
            agentId={currentAgentId}
            sessionId={currentSessionId || undefined}
          />
        </main>
      </div>

      {/* Agent Manager Modal */}
      <AgentManager
        agents={agents}
        isOpen={agentManagerOpen}
        onClose={() => setAgentManagerOpen(false)}
        onSave={handleSaveAgent}
        onDelete={handleDeleteAgent}
      />

      {/* API Key Manager Modal */}
      <ApiKeyManager
        isOpen={apiKeyManagerOpen}
        onClose={() => setApiKeyManagerOpen(false)}
        apiKeys={apiKeys}
        onAdd={addApiKey}
        onEdit={editApiKey}
        onDelete={removeApiKey}
        onToggleActive={toggleActive}
        loading={apiKeysLoading}
        error={apiKeysError}
      />
    </div>
  );
};

export default App;
