/**
 * useWebSocket Hook
 * 
 * React hook for managing WebSocket connections with automatic reconnection,
 * JWT authentication, and message parsing.
 */

import { useEffect, useRef, useState, useCallback } from 'react';

export interface WsEventMessage {
  type: 'event';
  id: string;
  chain_id: number;
  chain_name: string;
  event_name: string;
  contract_address: string;
  block_number: number;
  transaction_hash: string;
  timestamp: string;
}

export interface WsStatsMessage {
  type: 'stats';
  events_total: number;
  events_24h: number;
  success_rate: number;
  active_endpoints: number;
}

export interface WsConnectedMessage {
  type: 'connected';
  message: string;
}

export interface WsPingMessage {
  type: 'ping';
  timestamp: number;
}

export interface WsErrorMessage {
  type: 'error';
  message: string;
}

export type WsMessage = 
  | WsEventMessage 
  | WsStatsMessage 
  | WsConnectedMessage 
  | WsPingMessage 
  | WsErrorMessage;

export interface UseWebSocketOptions {
  /**
   * Whether to automatically reconnect on disconnection
   * @default true
   */
  autoReconnect?: boolean;

  /**
   * Reconnection delay in milliseconds
   * @default 3000
   */
  reconnectDelay?: number;

  /**
   * Maximum number of reconnection attempts (0 = infinite)
   * @default 0
   */
  maxReconnectAttempts?: number;

  /**
   * Callback when connected
   */
  onConnect?: () => void;

  /**
   * Callback when disconnected
   */
  onDisconnect?: () => void;

  /**
   * Callback for errors
   */
  onError?: (error: Event) => void;
}

export interface UseWebSocketReturn {
  /** Array of received messages */
  messages: WsMessage[];
  
  /** Most recent message */
  lastMessage: WsMessage | null;
  
  /** Connection status */
  connected: boolean;
  
  /** Manual reconnect function */
  reconnect: () => void;
  
  /** Clear all messages */
  clearMessages: () => void;
}

/**
 * Hook to establish and manage WebSocket connection
 * 
 * @param endpoint - WebSocket endpoint: '/ws/events' or '/ws/stats'
 * @param options - Configuration options
 * @returns WebSocket state and control functions
 * 
 * @example
 * ```tsx
 * const { messages, connected } = useWebSocket('/ws/events', {
 *   onConnect: () => console.log('Connected!'),
 *   reconnectDelay: 5000
 * });
 * 
 * // Filter event messages
 * const events = messages.filter(m => m.type === 'event');
 * ```
 */
export function useWebSocket(
  endpoint: '/ws/events' | '/ws/stats',
  options: UseWebSocketOptions = {}
): UseWebSocketReturn {
  const {
    autoReconnect = true,
    reconnectDelay = 3000,
    maxReconnectAttempts = 0,
    onConnect,
    onDisconnect,
    onError,
  } = options;

  const [messages, setMessages] = useState<WsMessage[]>([]);
  const [lastMessage, setLastMessage] = useState<WsMessage | null>(null);
  const [connected, setConnected] = useState(false);
  
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined);

  const connect = useCallback(() => {
    // Get JWT token from localStorage
    const token = localStorage.getItem('auth_token');
    if (!token) {
      console.warn('[useWebSocket] No auth token found, skipping connection');
      return;
    }

    // Get WebSocket URL from environment or construct it
    const wsBaseUrl = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3000';
    const wsUrl = `${wsBaseUrl}/api/v1${endpoint}?token=${token}`;

    console.log(`[useWebSocket] Connecting to ${wsUrl}`);

    try {
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log(`[useWebSocket] Connected to ${endpoint}`);
        setConnected(true);
        reconnectAttemptsRef.current = 0; // Reset reconnect counter
        onConnect?.();
      };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WsMessage;
        
        // Ignore ping messages in message history
        if (message.type !== 'ping') {
          setMessages((prev) => [...prev, message]);
          setLastMessage(message);
        }

        // Log errors
        if (message.type === 'error') {
          console.error(`[useWebSocket] Server error: ${message.message}`);
        }
      } catch (error) {
        console.error('[useWebSocket] Failed to parse message:', error);
      }
    };

      ws.onerror = (event) => {
        // Only log if connection state indicates a real problem
        // The error event often fires during normal close/reconnect cycles
        if (wsRef.current?.readyState === WebSocket.OPEN) {
          console.error(`[useWebSocket] Error on ${endpoint}:`, event);
          onError?.(event);
        }
      };

      ws.onclose = () => {
        console.log(`[useWebSocket] Disconnected from ${endpoint}`);
        setConnected(false);
        wsRef.current = null;
        onDisconnect?.();

        // Attempt reconnection
        if (
          autoReconnect &&
          (maxReconnectAttempts === 0 || reconnectAttemptsRef.current < maxReconnectAttempts)
        ) {
          reconnectAttemptsRef.current += 1;
          console.log(
            `[useWebSocket] Reconnecting in ${reconnectDelay}ms (attempt ${reconnectAttemptsRef.current})...`
          );
          
          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, reconnectDelay);
        }
      };
    } catch (error) {
      console.error(`[useWebSocket] Failed to create WebSocket connection:`, error);
      setConnected(false);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [endpoint, autoReconnect, reconnectDelay, maxReconnectAttempts]);

  const reconnect = useCallback(() => {
    console.log('[useWebSocket] Manual reconnect triggered');
    
    // Close existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    // Clear reconnect timeout
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    // Reset counter and reconnect
    reconnectAttemptsRef.current = 0;
    connect();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const clearMessages = useCallback(() => {
    setMessages([]);
    setLastMessage(null);
  }, []);

  // Connect on mount only if token exists
  useEffect(() => {
    const token = localStorage.getItem('auth_token');
    if (!token) {
      console.warn('[useWebSocket] Skipping connection - no auth token');
      return;
    }

    connect();

    // Cleanup on unmount
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connect]);

  return {
    messages,
    lastMessage,
    connected,
    reconnect,
    clearMessages,
  };
}
