import React from 'react';
import { AgentDefinition } from '../types';
import { ChevronDown, Crown, Bot, Wrench } from 'lucide-react';

interface AgentSelectorProps {
  agents: AgentDefinition[];
  currentAgentId: string;
  onSelect: (agentId: string) => void;
}

const AgentSelector: React.FC<AgentSelectorProps> = ({ agents, currentAgentId, onSelect }) => {
  const orchestrator = agents.find(a => a.is_orchestrator);
  const builtinAgents = agents.filter(a => a.source === 'built-in' && !a.is_orchestrator);
  const customAgents = agents.filter(a => a.source !== 'built-in');
  const current = agents.find(a => a.agent_id === currentAgentId);

  return (
    <div className="relative group">
      <button className="flex items-center gap-2 px-3 py-1.5 bg-bg-tertiary rounded-lg border border-bg-tertiary hover:border-primary/30 transition-colors cursor-pointer">
        <Bot className="w-4 h-4 text-primary-light" />
        <span className="text-sm text-text-primary">{current?.name || '自动调度'}</span>
        <ChevronDown className="w-3 h-3 text-text-muted" />
      </button>

      {/* Dropdown */}
      <div className="absolute top-full left-0 mt-1 w-64 bg-bg-secondary border border-bg-tertiary rounded-lg shadow-xl opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-50">
        {/* Orchestrator */}
        {orchestrator && (
          <div className="p-1 border-b border-bg-tertiary">
            <div className="text-xs text-text-muted px-2 py-1 uppercase">智能调度</div>
            <button
              onClick={() => onSelect(orchestrator.agent_id)}
              className={`w-full flex items-center gap-2 px-2 py-2 rounded text-left hover:bg-bg-tertiary/50 transition-colors cursor-pointer ${
                currentAgentId === orchestrator.agent_id ? 'bg-bg-tertiary/50' : ''
              }`}
            >
              <Crown className="w-4 h-4 text-warning" />
              <div>
                <div className="text-sm text-text-primary">{orchestrator.name}</div>
                <div className="text-xs text-text-muted truncate">{orchestrator.description}</div>
              </div>
            </button>
          </div>
        )}

        {/* Built-in Agents */}
        {builtinAgents.length > 0 && (
          <div className="p-1 border-b border-bg-tertiary">
            <div className="text-xs text-text-muted px-2 py-1 uppercase">内置 Agent</div>
            {builtinAgents.map(agent => (
              <button
                key={agent.agent_id}
                onClick={() => onSelect(agent.agent_id)}
                className={`w-full flex items-center gap-2 px-2 py-2 rounded text-left hover:bg-bg-tertiary/50 transition-colors cursor-pointer ${
                  currentAgentId === agent.agent_id ? 'bg-bg-tertiary/50' : ''
                }`}
              >
                <Bot className="w-4 h-4 text-accent" />
                <div>
                  <div className="text-sm text-text-primary">{agent.name}</div>
                  <div className="text-xs text-text-muted truncate">{agent.description}</div>
                </div>
              </button>
            ))}
          </div>
        )}

        {/* Custom Agents */}
        {customAgents.length > 0 && (
          <div className="p-1 border-b border-bg-tertiary">
            <div className="text-xs text-text-muted px-2 py-1 uppercase">自定义 Agent</div>
            {customAgents.map(agent => (
              <button
                key={agent.agent_id}
                onClick={() => onSelect(agent.agent_id)}
                className={`w-full flex items-center gap-2 px-2 py-2 rounded text-left hover:bg-bg-tertiary/50 transition-colors cursor-pointer ${
                  currentAgentId === agent.agent_id ? 'bg-bg-tertiary/50' : ''
                }`}
              >
                <Wrench className="w-4 h-4 text-text-secondary" />
                <div>
                  <div className="text-sm text-text-primary">{agent.name}</div>
                  <div className="text-xs text-text-muted truncate">{agent.description}</div>
                </div>
              </button>
            ))}
          </div>
        )}

        {/* Manage button */}
        <div className="p-1">
          <button className="w-full text-center text-xs text-primary-light hover:text-primary py-2 cursor-pointer transition-colors">
            管理 Agent →
          </button>
        </div>
      </div>
    </div>
  );
};

export default AgentSelector;
