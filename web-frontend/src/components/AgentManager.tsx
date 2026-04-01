import React, { useState } from 'react';
import { AgentDefinition } from '../types';
import { X, Crown, Bot, Wrench, Trash2, Save } from 'lucide-react';

interface AgentManagerProps {
  agents: AgentDefinition[];
  isOpen: boolean;
  onClose: () => void;
  onSave: (data: { name: string; description: string; when_to_use: string; system_prompt: string; tools: string[]; model: string }) => void;
  onDelete: (id: string) => void;
}

const AgentManager: React.FC<AgentManagerProps> = ({ agents, isOpen, onClose, onSave, onDelete }) => {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    when_to_use: '',
    system_prompt: '',
    tools: [] as string[],
    model: 'sonnet',
  });

  const orchestrator = agents.find(a => a.is_orchestrator);
  const builtinAgents = agents.filter(a => a.source === 'built-in' && !a.is_orchestrator);
  const customAgents = agents.filter(a => a.source !== 'built-in');

  const handleSelectAgent = (agent: AgentDefinition) => {
    setSelectedId(agent.agent_id);
    setFormData({
      name: agent.name,
      description: agent.description,
      when_to_use: agent.when_to_use,
      system_prompt: '',
      tools: agent.tools,
      model: agent.model,
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Overlay */}
      <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" onClick={onClose} />

      {/* Modal */}
      <div className="relative w-full max-w-4xl h-[80vh] bg-bg-secondary rounded-2xl border border-bg-tertiary shadow-2xl flex overflow-hidden">
        {/* Left - Agent List */}
        <div className="w-72 border-r border-bg-tertiary flex flex-col">
          <div className="p-4 border-b border-bg-tertiary">
            <h2 className="text-lg font-semibold text-text-primary">Agent 管理</h2>
          </div>
          <div className="flex-1 overflow-y-auto p-2 space-y-1">
            {/* Orchestrator */}
            {orchestrator && (
              <>
                <div className="text-xs text-text-muted px-2 py-1 uppercase">调度中心</div>
                <button
                  onClick={() => handleSelectAgent(orchestrator)}
                  className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left transition-colors cursor-pointer ${
                    selectedId === orchestrator.agent_id ? 'bg-bg-tertiary' : 'hover:bg-bg-tertiary/50'
                  }`}
                >
                  <Crown className="w-4 h-4 text-warning flex-shrink-0" />
                  <span className="text-sm text-text-primary truncate">{orchestrator.name}</span>
                </button>
              </>
            )}

            {/* Built-in */}
            <div className="text-xs text-text-muted px-2 py-1 uppercase mt-2">内置 Agent</div>
            {builtinAgents.map(agent => (
              <button
                key={agent.agent_id}
                onClick={() => handleSelectAgent(agent)}
                className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left transition-colors cursor-pointer ${
                  selectedId === agent.agent_id ? 'bg-bg-tertiary' : 'hover:bg-bg-tertiary/50'
                }`}
              >
                <Bot className="w-4 h-4 text-accent flex-shrink-0" />
                <span className="text-sm text-text-primary truncate">{agent.name}</span>
              </button>
            ))}

            {/* Custom */}
            <div className="text-xs text-text-muted px-2 py-1 uppercase mt-2">自定义 Agent</div>
            {customAgents.length === 0 ? (
              <div className="text-xs text-text-muted text-center py-4">暂无自定义 Agent</div>
            ) : (
              customAgents.map(agent => (
                <button
                  key={agent.agent_id}
                  onClick={() => handleSelectAgent(agent)}
                  className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left transition-colors cursor-pointer ${
                    selectedId === agent.agent_id ? 'bg-bg-tertiary' : 'hover:bg-bg-tertiary/50'
                  }`}
                >
                  <Wrench className="w-4 h-4 text-text-secondary flex-shrink-0" />
                  <span className="text-sm text-text-primary truncate">{agent.name}</span>
                </button>
              ))
            )}
          </div>
        </div>

        {/* Right - Form */}
        <div className="flex-1 flex flex-col">
          <div className="flex items-center justify-between p-4 border-b border-bg-tertiary">
            <h3 className="text-base font-semibold text-text-primary">
              {selectedId ? '编辑 Agent' : '新建 Agent'}
            </h3>
            <button onClick={onClose} className="p-1 hover:bg-bg-tertiary rounded cursor-pointer transition-colors">
              <X className="w-5 h-5 text-text-muted" />
            </button>
          </div>

          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            <div>
              <label className="block text-sm font-medium text-text-secondary mb-1">名称</label>
              <input
                type="text"
                value={formData.name}
                onChange={e => setFormData(prev => ({ ...prev, name: e.target.value }))}
                className="w-full bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary/50"
                placeholder="Agent 名称"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-text-secondary mb-1">描述</label>
              <input
                type="text"
                value={formData.description}
                onChange={e => setFormData(prev => ({ ...prev, description: e.target.value }))}
                className="w-full bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary/50"
                placeholder="Agent 功能描述"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-text-secondary mb-1">适用场景</label>
              <input
                type="text"
                value={formData.when_to_use}
                onChange={e => setFormData(prev => ({ ...prev, when_to_use: e.target.value }))}
                className="w-full bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary/50"
                placeholder="什么时候使用这个 Agent"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-text-secondary mb-1">
                System Prompt
                <span className="text-text-muted ml-2">({formData.system_prompt.length} 字符)</span>
              </label>
              <textarea
                value={formData.system_prompt}
                onChange={e => setFormData(prev => ({ ...prev, system_prompt: e.target.value }))}
                rows={8}
                className="w-full bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary/50 resize-y font-mono"
                placeholder="Agent 的系统提示词..."
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-text-secondary mb-1">模型</label>
              <select
                value={formData.model}
                onChange={e => setFormData(prev => ({ ...prev, model: e.target.value }))}
                className="bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary/50 cursor-pointer"
              >
                <option value="sonnet">Sonnet</option>
                <option value="opus">Opus</option>
                <option value="haiku">Haiku</option>
              </select>
            </div>
          </div>

          {/* Action buttons */}
          <div className="flex items-center justify-between p-4 border-t border-bg-tertiary">
            {selectedId && (
              <button
                onClick={() => selectedId && onDelete(selectedId)}
                className="flex items-center gap-1.5 px-3 py-2 text-sm text-danger hover:bg-danger/10 rounded-lg transition-colors cursor-pointer"
              >
                <Trash2 className="w-4 h-4" />
                删除
              </button>
            )}
            <div className="flex-1" />
            <button
              onClick={() => onSave(formData)}
              className="flex items-center gap-1.5 px-4 py-2 text-sm bg-gradient-to-r from-primary to-primary-dark text-white rounded-lg hover:opacity-90 transition-opacity cursor-pointer"
            >
              <Save className="w-4 h-4" />
              保存
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AgentManager;
