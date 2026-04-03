import { ApiResult, AgentDefinition, Session, ApiKeyConfig } from '../types';

const API_BASE = '/api/agent';

async function fetchJson<T>(url: string, options?: RequestInit): Promise<ApiResult<T>> {
  const response = await fetch(url, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  return response.json();
}

// ===== Sessions =====

export async function createSession(agentId: string, title?: string) {
  return fetchJson<Session>(`${API_BASE}/sessions`, {
    method: 'POST',
    body: JSON.stringify({ agent_id: agentId, title }),
  });
}

export async function listSessions() {
  return fetchJson<Session[]>(`${API_BASE}/sessions`);
}

export async function getSession(id: string) {
  return fetchJson<Session>(`${API_BASE}/sessions/${id}`);
}

export async function deleteSession(id: string) {
  return fetchJson<string>(`${API_BASE}/sessions/${id}`, { method: 'DELETE' });
}

// ===== Agents =====

export async function listAgents() {
  return fetchJson<AgentDefinition[]>(`${API_BASE}/agents`);
}

export async function getAgent(id: string) {
  return fetchJson<AgentDefinition>(`${API_BASE}/agents/${id}`);
}

export async function createAgent(data: {
  name: string;
  description: string;
  when_to_use?: string;
  tools?: string[];
  model?: string;
  system_prompt: string;
}) {
  return fetchJson<AgentDefinition>(`${API_BASE}/agents`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateAgent(id: string, data: Partial<{
  name: string;
  description: string;
  when_to_use: string;
  tools: string[];
  model: string;
  system_prompt: string;
}>) {
  return fetchJson<string>(`${API_BASE}/agents/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deleteAgent(id: string) {
  return fetchJson<string>(`${API_BASE}/agents/${id}`, { method: 'DELETE' });
}

// ===== Tools =====

export async function listTools() {
  return fetchJson<{ name: string; description: string }[]>(`${API_BASE}/tools`);
}

// ===== Health =====

export async function healthCheck() {
  return fetchJson<{ status: string; timestamp: string }>(`${API_BASE}/health`);
}

// ===== Native Callback =====

export async function sendNativeCallback(
  callbackId: string,
  sessionId: string,
  result: unknown,
  success: boolean,
) {
  return fetchJson<{ received: boolean; callback_id: string }>(`${API_BASE}/native-callback`, {
    method: 'POST',
    body: JSON.stringify({
      callback_id: callbackId,
      session_id: sessionId,
      result,
      success,
    }),
  });
}

// ===== API Keys =====

export async function listApiKeys() {
  return fetchJson<ApiKeyConfig[]>(`${API_BASE}/api-keys`);
}

export async function getApiKey(id: string) {
  return fetchJson<ApiKeyConfig>(`${API_BASE}/api-keys/${id}`);
}

export async function createApiKey(data: {
  name: string;
  api_key: string;
  base_url: string;
  default_model: string;
}) {
  return fetchJson<ApiKeyConfig>(`${API_BASE}/api-keys`, {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateApiKey(id: string, data: Partial<{
  name: string;
  api_key: string;
  base_url: string;
  default_model: string;
  is_active: boolean;
}>) {
  return fetchJson<ApiKeyConfig>(`${API_BASE}/api-keys/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deleteApiKey(id: string) {
  return fetchJson<string>(`${API_BASE}/api-keys/${id}`, { method: 'DELETE' });
}
