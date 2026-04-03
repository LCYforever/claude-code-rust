import { useState, useEffect, useCallback } from 'react';
import { ApiKeyConfig } from '../types';
import * as api from '../api/client';

export function useApiKeys() {
  const [apiKeys, setApiKeys] = useState<ApiKeyConfig[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchApiKeys = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.listApiKeys();
      if (result.success && result.data) {
        setApiKeys(result.data);
      } else {
        setError(result.error || '获取 API Key 列表失败');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '网络请求失败');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchApiKeys();
  }, [fetchApiKeys]);

  const addApiKey = useCallback(async (data: {
    name: string;
    api_key: string;
    base_url: string;
    default_model: string;
  }) => {
    setError(null);
    try {
      const result = await api.createApiKey(data);
      if (result.success && result.data) {
        setApiKeys(prev => [...prev, result.data!]);
        return result.data;
      } else {
        setError(result.error || '创建失败');
        return null;
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '网络请求失败');
      return null;
    }
  }, []);

  const editApiKey = useCallback(async (id: string, data: Partial<{
    name: string;
    api_key: string;
    base_url: string;
    default_model: string;
    is_active: boolean;
  }>) => {
    setError(null);
    try {
      const result = await api.updateApiKey(id, data);
      if (result.success && result.data) {
        setApiKeys(prev => prev.map(k => k.id === id ? result.data! : k));
        return true;
      } else {
        setError(result.error || '更新失败');
        return false;
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '网络请求失败');
      return false;
    }
  }, []);

  const removeApiKey = useCallback(async (id: string) => {
    setError(null);
    try {
      const result = await api.deleteApiKey(id);
      if (result.success) {
        setApiKeys(prev => prev.filter(k => k.id !== id));
        return true;
      } else {
        setError(result.error || '删除失败');
        return false;
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '网络请求失败');
      return false;
    }
  }, []);

  const toggleActive = useCallback(async (id: string) => {
    const key = apiKeys.find(k => k.id === id);
    if (!key) return false;
    return editApiKey(id, { is_active: !key.is_active });
  }, [apiKeys, editApiKey]);

  return {
    apiKeys,
    loading,
    error,
    addApiKey,
    editApiKey,
    removeApiKey,
    toggleActive,
    refresh: fetchApiKeys,
  };
}
