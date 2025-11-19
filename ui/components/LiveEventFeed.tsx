/**
 * LiveEventFeed Component
 * 
 * Displays real-time events as they arrive via WebSocket.
 * Shows the most recent 10 events with auto-scrolling.
 */

'use client';

import { useWebSocket, WsEventMessage } from '@/hooks/useWebSocket';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { formatDistanceToNow } from 'date-fns';
import { Activity, Circle } from 'lucide-react';

interface LiveEventFeedProps {
  /**
   * Maximum number of events to display
   * @default 10
   */
  maxEvents?: number;
}

export function LiveEventFeed({ maxEvents = 10 }: LiveEventFeedProps) {
  const { messages, connected, reconnect } = useWebSocket('/ws/events');

  // Filter event messages and get most recent
  const events = messages
    .filter((m): m is WsEventMessage => m.type === 'event')
    .slice(-maxEvents)
    .reverse(); // Show newest first

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <Activity className="h-5 w-5" />
            Live Events
          </CardTitle>
          
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1.5">
              <Circle
                className={`h-2 w-2 ${
                  connected ? 'fill-green-500 text-green-500' : 'fill-red-500 text-red-500'
                }`}
              />
              <span className="text-xs text-muted-foreground">
                {connected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
            
            {!connected && (
              <button
                onClick={reconnect}
                className="text-xs text-primary hover:underline"
              >
                Reconnect
              </button>
            )}
          </div>
        </div>
      </CardHeader>
      
      <CardContent>
        {events.length === 0 ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            {connected ? (
              <>
                <Activity className="mx-auto h-12 w-12 opacity-20 mb-2" />
                Waiting for events...
              </>
            ) : (
              <>
                <Circle className="mx-auto h-12 w-12 opacity-20 mb-2" />
                No connection
              </>
            )}
          </div>
        ) : (
          <div className="space-y-2 max-h-[400px] overflow-y-auto">
            {events.map((event) => (
              <div
                key={event.id}
                className="flex items-start gap-3 rounded-lg border p-3 animate-fade-in"
              >
                {/* Chain badge */}
                <div className="flex-shrink-0 pt-0.5">
                  <Badge variant="outline" className="text-xs">
                    {event.chain_name}
                  </Badge>
                </div>
                
                {/* Event details */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between gap-2 mb-1">
                    <span className="font-medium text-sm truncate">
                      {event.event_name}
                    </span>
                    <span className="text-xs text-muted-foreground whitespace-nowrap">
                      {formatDistanceToNow(new Date(event.timestamp), { addSuffix: true })}
                    </span>
                  </div>
                  
                  <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-muted-foreground">
                    <div className="truncate" title={event.contract_address}>
                      <span className="font-medium">Contract:</span>{' '}
                      {event.contract_address.slice(0, 8)}...
                    </div>
                    <div>
                      <span className="font-medium">Block:</span> {event.block_number}
                    </div>
                    <div className="col-span-2 truncate" title={event.transaction_hash}>
                      <span className="font-medium">Tx:</span>{' '}
                      {event.transaction_hash.slice(0, 10)}...{event.transaction_hash.slice(-8)}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
