import { useState, useRef, useCallback, useEffect } from 'react';

interface UseWebSocketOptions {
  url?: string;
  onMessage?: (data: Record<string, unknown>) => void;
  onOpen?: () => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const [isConnected, setIsConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);

  const connect = useCallback(() => {
    if (!options.url) return;

    const ws = new WebSocket(options.url);

    ws.onopen = () => {
      setIsConnected(true);
      options.onOpen?.();
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        options.onMessage?.(data);
      } catch {
        console.error('Failed to parse WebSocket message');
      }
    };

    ws.onclose = () => {
      setIsConnected(false);
      options.onClose?.();
    };

    ws.onerror = (error) => {
      options.onError?.(error);
    };

    wsRef.current = ws;
  }, [options]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
    setIsConnected(false);
  }, []);

  const send = useCallback((data: Record<string, unknown>) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(data));
    }
  }, []);

  useEffect(() => {
    return () => {
      wsRef.current?.close();
    };
  }, []);

  return {
    isConnected,
    connect,
    disconnect,
    send,
  };
}
