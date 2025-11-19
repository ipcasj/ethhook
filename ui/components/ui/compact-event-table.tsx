/**
 * CompactEventTable Component
 * Condensed event display optimized for dashboard overview
 * Takes up ~40% of screen height and displays key information densely
 */

'use client';

import { StatusBadge } from './status-badge';
import { Event } from '@/lib/types';
import { cn, truncateAddress } from '@/lib/utils';

interface CompactEventTableProps {
  events: Event[];
  onEventClick?: (event: Event) => void;
  maxHeight?: string;
  className?: string;
}

export function CompactEventTable({
  events,
  onEventClick,
  maxHeight = '320px',
  className,
}: CompactEventTableProps) {
  const calculateEventStatus = (event: Event): 'delivered' | 'failed' | 'pending' => {
    if (event.status && ['delivered', 'failed', 'pending'].includes(event.status)) {
      return event.status as 'delivered' | 'failed' | 'pending';
    }
    if ((event.successful_deliveries ?? 0) > 0) return 'delivered';
    if ((event.delivery_count ?? 0) > 0) return 'failed';
    return 'pending';
  };

  const getEventType = (topics?: string[]) => {
    if (!topics || topics.length === 0) return 'Unknown';
    const eventSignature = topics[0];
    const eventTypes: Record<string, string> = {
      '0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef': 'Transfer',
      '0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c': 'Deposit',
      '0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65': 'Withdrawal',
      '0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925': 'Approval',
    };
    return eventTypes[eventSignature] || 'Event';
  };

  const getChainName = (chainId: number | null | undefined) => {
    if (!chainId) return 'Unknown';
    const chains: Record<number, string> = {
      1: 'Ethereum',
      10: 'Optimism',
      42161: 'Arbitrum',
      8453: 'Base',
      137: 'Polygon',
      11155111: 'Sepolia',
    };
    return chains[chainId] || `Chain ${chainId}`;
  };

  return (
    <div className={cn('border rounded-lg overflow-hidden bg-white', className)}>
      {/* Horizontal scroll wrapper */}
      <div className="overflow-x-auto">
        <div className="min-w-[600px]">
          {/* Header - Fixed width columns */}
          <div className="bg-slate-50 border-b px-4 py-2">
            <div className="grid grid-cols-12 gap-2 text-xs font-medium text-slate-600 uppercase tracking-wide">
              <div className="col-span-2">Status</div>
              <div className="col-span-3">Event</div>
              <div className="col-span-2">Chain</div>
              <div className="col-span-2">Block</div>
              <div className="col-span-2">Deliveries</div>
              <div className="col-span-1">Time</div>
            </div>
          </div>

          {/* Body - Fixed width columns */}
          <div style={{ maxHeight, overflowY: 'auto' }} className="divide-y">
        {events.length === 0 ? (
          <div className="px-4 py-8 text-center text-sm text-slate-500">
            No events to display
          </div>
        ) : (
          events.map((event) => (
            <div
              key={event.id}
              onClick={() => onEventClick?.(event)}
              className={cn(
                'grid grid-cols-12 gap-2 px-4 py-2.5 text-sm hover:bg-slate-50 transition-colors',
                onEventClick && 'cursor-pointer'
              )}
            >
              {/* Status */}
              <div className="col-span-2 flex items-center">
                <StatusBadge status={calculateEventStatus(event)} size="sm" />
              </div>

              {/* Event Type & Contract */}
              <div className="col-span-3 flex flex-col justify-center min-w-0">
                <span className="font-medium text-slate-900 truncate text-sm">
                  {getEventType(event.topics)}
                </span>
                <span className="font-mono text-xs text-slate-500 truncate">
                  {truncateAddress(event.contract_address)}
                </span>
              </div>

              {/* Chain */}
              <div className="col-span-2 flex items-center">
                <span className="text-slate-700 text-xs truncate">
                  {getChainName(event.chain_id)}
                </span>
              </div>

              {/* Block Number */}
              <div className="col-span-2 flex items-center">
                <span className="font-mono text-xs text-slate-700 truncate">
                  {event.block_number.toLocaleString()}
                </span>
              </div>

              {/* Deliveries */}
              <div className="col-span-2 flex flex-col justify-center min-w-0">
                <span className="text-slate-700 text-xs truncate">
                  {event.successful_deliveries ?? 0}/{event.delivery_count ?? 0}
                </span>
                <span className="text-xs text-slate-500 truncate">
                  {event.endpoint_name || 'No endpoint'}
                </span>
              </div>

              {/* Time */}
              <div className="col-span-1 flex items-center justify-end">
                <span className="text-xs text-slate-500 text-right">
                  {new Date(event.ingested_at).toLocaleTimeString([], {
                    hour: '2-digit',
                    minute: '2-digit',
                  })}
                </span>
              </div>
            </div>
          ))
        )}
          </div>
        </div>
      </div>
    </div>
  );
}
