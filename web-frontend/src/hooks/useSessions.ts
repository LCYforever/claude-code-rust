import { useState, useEffect, useCallback } from 'react';
import { Session } from '../types';
import { listSessions, createSession, deleteSession } from '../api/client';

export function useSessions() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchSessions = useCallback(async () => {
    setLoading(true);
    const result = await listSessions();
    if (result.success && result.data) {
      setSessions(result.data);
    }
    setLoading(false);
  }, []);

  const createNewSession = useCallback(async (agentId: string, title?: string) => {
    const result = await createSession(agentId, title);
    if (result.success && result.data) {
      const newSession = result.data;
      setSessions(prev => [newSession, ...prev]);
      setCurrentSessionId(newSession.id);
      return newSession;
    }
    return null;
  }, []);

  const removeSession = useCallback(async (id: string) => {
    const result = await deleteSession(id);
    if (result.success) {
      setSessions(prev => prev.filter(s => s.id !== id));
      if (currentSessionId === id) {
        setCurrentSessionId(null);
      }
    }
  }, [currentSessionId]);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  return {
    sessions,
    currentSessionId,
    setCurrentSessionId,
    loading,
    createNewSession,
    removeSession,
    refreshSessions: fetchSessions,
  };
}
