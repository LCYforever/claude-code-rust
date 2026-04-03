import React, { useState, useEffect } from 'react';
import { ApiKeyConfig } from '../types';
import {
  X, Plus, Trash2, Save, Key, Globe, Cpu,
  Eye, EyeOff, ToggleLeft, ToggleRight, RefreshCw, Pencil,
} from 'lucide-react';

interface ApiKeyManagerProps {
  isOpen: boolean;
  onClose: () => void;
  apiKeys: ApiKeyConfig[];
  onAdd: (data: { name: string; api_key: string; base_url: string; default_model: string }) => Promise<ApiKeyConfig | null>;
  onEdit: (id: string, data: Partial<{ name: string; api_key: string; base_url: string; default_model: string; is_active: boolean }>) => Promise<boolean>;
  onDelete: (id: string) => Promise<boolean>;
  onToggleActive: (id: string) => Promise<boolean>;
  loading: boolean;
  error: string | null;
}

// 常见 Provider 预设
const PROVIDER_PRESETS = [
  { label: 'OpenAI', base_url: 'https://api.openai.com/v1', default_model: 'gpt-4o' },
  { label: 'Azure OpenAI', base_url: 'https://{your-resource}.openai.azure.com/openai', default_model: 'gpt-4o' },
  { label: 'Anthropic', base_url: 'https://api.anthropic.com/v1', default_model: 'claude-sonnet-4-20250514' },
  { label: 'DeepSeek', base_url: 'https://api.deepseek.com/v1', default_model: 'deepseek-chat' },
  { label: 'Moonshot (Kimi)', base_url: 'https://api.moonshot.cn/v1', default_model: 'moonshot-v1-128k' },
  { label: '通义千问', base_url: 'https://dashscope.aliyuncs.com/compatible-mode/v1', default_model: 'qwen-max' },
  { label: '智谱 GLM', base_url: 'https://open.bigmodel.cn/api/paas/v4', default_model: 'glm-4-plus' },
  { label: '自定义', base_url: '', default_model: '' },
];

const emptyForm = {
  name: '',
  api_key: '',
  base_url: '',
  default_model: '',
};

const ApiKeyManager: React.FC<ApiKeyManagerProps> = ({
  isOpen,
  onClose,
  apiKeys,
  onAdd,
  onEdit,
  onDelete,
  onToggleActive,
  loading,
  error,
}) => {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [formData, setFormData] = useState(emptyForm);
  const [showKey, setShowKey] = useState(false);
  const [saving, setSaving] = useState(false);

  // Reset when modal opens/closes
  useEffect(() => {
    if (!isOpen) {
      setSelectedId(null);
      setIsEditing(false);
      setIsCreating(false);
      setFormData(emptyForm);
      setShowKey(false);
    }
  }, [isOpen]);

  const handleSelectKey = (key: ApiKeyConfig) => {
    setSelectedId(key.id);
    setIsEditing(false);
    setIsCreating(false);
    setFormData({
      name: key.name,
      api_key: key.api_key,
      base_url: key.base_url,
      default_model: key.default_model,
    });
    setShowKey(false);
  };

  const handleNewKey = () => {
    setSelectedId(null);
    setIsCreating(true);
    setIsEditing(false);
    setFormData(emptyForm);
    setShowKey(false);
  };

  const handleApplyPreset = (preset: typeof PROVIDER_PRESETS[0]) => {
    setFormData(prev => ({
      ...prev,
      name: prev.name || preset.label,
      base_url: preset.base_url,
      default_model: preset.default_model,
    }));
  };

  const handleSave = async () => {
    if (!formData.name.trim() || !formData.api_key.trim() || !formData.base_url.trim()) return;
    setSaving(true);
    try {
      if (isCreating) {
        const result = await onAdd(formData);
        if (result) {
          setSelectedId(result.id);
          setIsCreating(false);
          setIsEditing(false);
        }
      } else if (selectedId) {
        const ok = await onEdit(selectedId, formData);
        if (ok) {
          setIsEditing(false);
        }
      }
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!selectedId) return;
    if (!window.confirm('确定删除该 API Key 配置？此操作不可撤销。')) return;
    const ok = await onDelete(selectedId);
    if (ok) {
      setSelectedId(null);
      setIsEditing(false);
      setFormData(emptyForm);
    }
  };

  const maskKey = (key: string) => {
    if (key.length <= 8) return '••••••••';
    return key.slice(0, 4) + '••••••••' + key.slice(-4);
  };

  const selectedKey = apiKeys.find(k => k.id === selectedId);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Overlay */}
      <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" onClick={onClose} />

      {/* Modal */}
      <div className="relative w-full max-w-5xl h-[85vh] bg-bg-secondary rounded-2xl border border-bg-tertiary shadow-2xl flex overflow-hidden animate-fade-in">
        {/* Left Panel - Key List */}
        <div className="w-80 border-r border-bg-tertiary flex flex-col">
          <div className="p-4 border-b border-bg-tertiary flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Key className="w-5 h-5 text-primary" />
              <h2 className="text-lg font-semibold text-text-primary">API Key 管理</h2>
            </div>
            <button
              onClick={handleNewKey}
              className="p-1.5 hover:bg-bg-tertiary rounded-lg transition-colors cursor-pointer"
              title="新增 API Key"
            >
              <Plus className="w-4 h-4 text-primary" />
            </button>
          </div>

          <div className="flex-1 overflow-y-auto p-2 space-y-1">
            {loading && apiKeys.length === 0 && (
              <div className="flex items-center justify-center py-8">
                <RefreshCw className="w-5 h-5 text-text-muted animate-spin" />
              </div>
            )}
            {!loading && apiKeys.length === 0 && (
              <div className="text-center py-8">
                <Key className="w-8 h-8 text-text-muted mx-auto mb-2 opacity-50" />
                <p className="text-sm text-text-muted">暂无 API Key</p>
                <p className="text-xs text-text-muted mt-1">点击 + 添加第一个</p>
              </div>
            )}
            {apiKeys.map(key => (
              <button
                key={key.id}
                onClick={() => handleSelectKey(key)}
                className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-colors cursor-pointer ${
                  selectedId === key.id ? 'bg-bg-tertiary' : 'hover:bg-bg-tertiary/50'
                }`}
              >
                <div className={`w-2 h-2 rounded-full flex-shrink-0 ${key.is_active ? 'bg-success' : 'bg-text-muted'}`} />
                <div className="flex-1 min-w-0">
                  <div className="text-sm text-text-primary font-medium truncate">{key.name}</div>
                  <div className="text-xs text-text-muted truncate mt-0.5">{key.default_model}</div>
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Right Panel - Detail / Form */}
        <div className="flex-1 flex flex-col">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-bg-tertiary">
            <h3 className="text-base font-semibold text-text-primary">
              {isCreating ? '新增 API Key' : selectedId ? (isEditing ? '编辑配置' : '配置详情') : '选择或添加 API Key'}
            </h3>
            <button onClick={onClose} className="p-1 hover:bg-bg-tertiary rounded cursor-pointer transition-colors">
              <X className="w-5 h-5 text-text-muted" />
            </button>
          </div>

          {/* Content */}
          {!selectedId && !isCreating ? (
            // Empty state
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 rounded-2xl bg-bg-tertiary flex items-center justify-center mb-4">
                <Key className="w-8 h-8 text-text-muted" />
              </div>
              <p className="text-text-secondary text-sm">从左侧选择一个 API Key 查看详情</p>
              <p className="text-text-muted text-xs mt-1">或点击 + 新增配置</p>
              <button
                onClick={handleNewKey}
                className="mt-4 flex items-center gap-2 px-4 py-2 text-sm bg-gradient-to-r from-primary to-primary-dark text-white rounded-lg hover:opacity-90 transition-opacity cursor-pointer"
              >
                <Plus className="w-4 h-4" />
                新增 API Key
              </button>
            </div>
          ) : (
            <>
              {/* Form area */}
              <div className="flex-1 overflow-y-auto p-4 space-y-5">
                {/* Error display */}
                {error && (
                  <div className="bg-danger/10 border border-danger/20 rounded-lg px-3 py-2 text-sm text-danger">
                    {error}
                  </div>
                )}

                {/* Provider presets (only in create/edit mode) */}
                {(isCreating || isEditing) && (
                  <div>
                    <label className="block text-sm font-medium text-text-secondary mb-2">快速选择 Provider</label>
                    <div className="flex flex-wrap gap-2">
                      {PROVIDER_PRESETS.map(preset => (
                        <button
                          key={preset.label}
                          onClick={() => handleApplyPreset(preset)}
                          className="px-3 py-1.5 text-xs bg-bg-tertiary hover:bg-primary/20 hover:text-primary border border-bg-tertiary hover:border-primary/30 rounded-full transition-all cursor-pointer"
                        >
                          {preset.label}
                        </button>
                      ))}
                    </div>
                  </div>
                )}

                {/* Name */}
                <div>
                  <label className="flex items-center gap-1.5 text-sm font-medium text-text-secondary mb-1">
                    <Cpu className="w-3.5 h-3.5" />
                    名称
                  </label>
                  {isCreating || isEditing ? (
                    <input
                      type="text"
                      value={formData.name}
                      onChange={e => setFormData(prev => ({ ...prev, name: e.target.value }))}
                      className="w-full bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary/50"
                      placeholder="例如：OpenAI Production、DeepSeek 测试"
                    />
                  ) : (
                    <div className="w-full bg-bg-primary rounded-lg px-3 py-2 text-sm text-text-primary">
                      {selectedKey?.name || '-'}
                    </div>
                  )}
                </div>

                {/* API Key */}
                <div>
                  <label className="flex items-center gap-1.5 text-sm font-medium text-text-secondary mb-1">
                    <Key className="w-3.5 h-3.5" />
                    API Key
                  </label>
                  {isCreating || isEditing ? (
                    <div className="relative">
                      <input
                        type={showKey ? 'text' : 'password'}
                        value={formData.api_key}
                        onChange={e => setFormData(prev => ({ ...prev, api_key: e.target.value }))}
                        className="w-full bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 pr-10 text-sm text-text-primary font-mono focus:outline-none focus:border-primary/50"
                        placeholder="sk-xxxxxxxxxxxxxxxxxxxx"
                      />
                      <button
                        type="button"
                        onClick={() => setShowKey(!showKey)}
                        className="absolute right-2 top-1/2 -translate-y-1/2 p-1 hover:bg-bg-primary rounded cursor-pointer transition-colors"
                      >
                        {showKey ? <EyeOff className="w-4 h-4 text-text-muted" /> : <Eye className="w-4 h-4 text-text-muted" />}
                      </button>
                    </div>
                  ) : (
                    <div className="flex items-center gap-2">
                      <div className="flex-1 bg-bg-primary rounded-lg px-3 py-2 text-sm text-text-primary font-mono">
                        {showKey ? (selectedKey?.api_key || '') : maskKey(selectedKey?.api_key || '')}
                      </div>
                      <button
                        onClick={() => setShowKey(!showKey)}
                        className="p-2 hover:bg-bg-tertiary rounded-lg cursor-pointer transition-colors"
                      >
                        {showKey ? <EyeOff className="w-4 h-4 text-text-muted" /> : <Eye className="w-4 h-4 text-text-muted" />}
                      </button>
                    </div>
                  )}
                </div>

                {/* Base URL */}
                <div>
                  <label className="flex items-center gap-1.5 text-sm font-medium text-text-secondary mb-1">
                    <Globe className="w-3.5 h-3.5" />
                    Base URL
                  </label>
                  {isCreating || isEditing ? (
                    <input
                      type="text"
                      value={formData.base_url}
                      onChange={e => setFormData(prev => ({ ...prev, base_url: e.target.value }))}
                      className="w-full bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 text-sm text-text-primary font-mono focus:outline-none focus:border-primary/50"
                      placeholder="https://api.openai.com/v1"
                    />
                  ) : (
                    <div className="w-full bg-bg-primary rounded-lg px-3 py-2 text-sm text-text-primary font-mono">
                      {selectedKey?.base_url || '-'}
                    </div>
                  )}
                  <p className="text-xs text-text-muted mt-1">
                    兼容 OpenAI 格式的 API 地址（支持 OpenAI / Azure / Anthropic / DeepSeek / 通义千问 / Moonshot 等）
                  </p>
                </div>

                {/* Default Model */}
                <div>
                  <label className="flex items-center gap-1.5 text-sm font-medium text-text-secondary mb-1">
                    <Cpu className="w-3.5 h-3.5" />
                    Default Model
                  </label>
                  {isCreating || isEditing ? (
                    <input
                      type="text"
                      value={formData.default_model}
                      onChange={e => setFormData(prev => ({ ...prev, default_model: e.target.value }))}
                      className="w-full bg-bg-tertiary border border-bg-tertiary rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-primary/50"
                      placeholder="gpt-4o / claude-sonnet-4-20250514 / deepseek-chat ..."
                    />
                  ) : (
                    <div className="w-full bg-bg-primary rounded-lg px-3 py-2 text-sm text-text-primary">
                      {selectedKey?.default_model || '-'}
                    </div>
                  )}
                </div>

                {/* Status (read only mode) */}
                {selectedKey && !isCreating && !isEditing && (
                  <div>
                    <label className="block text-sm font-medium text-text-secondary mb-1">状态</label>
                    <div className="flex items-center gap-3">
                      <button
                        onClick={() => onToggleActive(selectedKey.id)}
                        className="flex items-center gap-2 cursor-pointer"
                      >
                        {selectedKey.is_active ? (
                          <ToggleRight className="w-6 h-6 text-success" />
                        ) : (
                          <ToggleLeft className="w-6 h-6 text-text-muted" />
                        )}
                        <span className={`text-sm ${selectedKey.is_active ? 'text-success' : 'text-text-muted'}`}>
                          {selectedKey.is_active ? '已启用' : '已禁用'}
                        </span>
                      </button>
                    </div>
                  </div>
                )}

                {/* Timestamps (read only) */}
                {selectedKey && !isCreating && !isEditing && (
                  <div className="flex gap-6 text-xs text-text-muted pt-2 border-t border-bg-tertiary">
                    <div>创建于: {new Date(selectedKey.created_at).toLocaleString('zh-CN')}</div>
                    <div>更新于: {new Date(selectedKey.updated_at).toLocaleString('zh-CN')}</div>
                  </div>
                )}
              </div>

              {/* Action buttons */}
              <div className="flex items-center justify-between p-4 border-t border-bg-tertiary">
                {/* Left actions */}
                <div className="flex items-center gap-2">
                  {selectedId && !isCreating && !isEditing && (
                    <>
                      <button
                        onClick={() => setIsEditing(true)}
                        className="flex items-center gap-1.5 px-3 py-2 text-sm text-text-secondary hover:bg-bg-tertiary rounded-lg transition-colors cursor-pointer"
                      >
                        <Pencil className="w-4 h-4" />
                        编辑
                      </button>
                      <button
                        onClick={handleDelete}
                        className="flex items-center gap-1.5 px-3 py-2 text-sm text-danger hover:bg-danger/10 rounded-lg transition-colors cursor-pointer"
                      >
                        <Trash2 className="w-4 h-4" />
                        删除
                      </button>
                    </>
                  )}
                </div>

                {/* Right actions */}
                <div className="flex items-center gap-2">
                  {(isCreating || isEditing) && (
                    <>
                      <button
                        onClick={() => {
                          if (isCreating) {
                            setIsCreating(false);
                            setFormData(emptyForm);
                          } else {
                            setIsEditing(false);
                            if (selectedKey) {
                              setFormData({
                                name: selectedKey.name,
                                api_key: selectedKey.api_key,
                                base_url: selectedKey.base_url,
                                default_model: selectedKey.default_model,
                              });
                            }
                          }
                        }}
                        className="px-4 py-2 text-sm text-text-secondary hover:bg-bg-tertiary rounded-lg transition-colors cursor-pointer"
                      >
                        取消
                      </button>
                      <button
                        onClick={handleSave}
                        disabled={saving || !formData.name.trim() || !formData.api_key.trim() || !formData.base_url.trim()}
                        className="flex items-center gap-1.5 px-4 py-2 text-sm bg-gradient-to-r from-primary to-primary-dark text-white rounded-lg hover:opacity-90 transition-opacity cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        <Save className="w-4 h-4" />
                        {saving ? '保存中...' : '保存'}
                      </button>
                    </>
                  )}
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default ApiKeyManager;
