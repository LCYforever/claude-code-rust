import { useState, useEffect, useCallback } from 'react';
import { AgentDefinition } from '../types';
import { listAgents, createAgent, updateAgent, deleteAgent } from '../api/client';

export function useAgents() {
  const [agents, setAgents] = useState<AgentDefinition[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchAgents = useCallback(async () => {
    setLoading(true);
    const result = await listAgents();
    if (result.success && result.data) {
      setAgents(result.data);
    }
    setLoading(false);
  }, []);

  const addAgent = useCallback(async (data: {
    name: string;
    description: string;
    when_to_use?: string;
    tools?: string[];
    model?: string;
    system_prompt: string;
  }) => {
    const result = await createAgent(data);
    if (result.success) {
      await fetchAgents();
    }
    return result;
  }, [fetchAgents]);

  const editAgent = useCallback(async (id: string, data: Partial<{
    name: string;
    description: string;
    when_to_use: string;
    tools: string[];
    model: string;
    system_prompt: string;
  }>) => {
    const result = await updateAgent(id, data);
    if (result.success) {
      await fetchAgents();
    }
    return result;
  }, [fetchAgents]);

  const removeAgent = useCallback(async (id: string) => {
    const result = await deleteAgent(id);
    if (result.success) {
      setAgents(prev => prev.filter(a => a.agent_id !== id));
    }
    return result;
  }, []);

  useEffect(() => {
    fetchAgents();
  }, [fetchAgents]);

  const orchestratorAgent = agents.find(a => a.is_orchestrator);
  const builtinAgents = agents.filter(a => a.source === 'built-in' && !a.is_orchestrator);
  const customAgents = agents.filter(a => a.source !== 'built-in');

  return {
    agents,
    orchestratorAgent,
    builtinAgents,
    customAgents,
    loading,
    addAgent,
    editAgent,
    removeAgent,
    refreshAgents: fetchAgents,
  };
}
