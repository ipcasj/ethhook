'use client';

import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { api } from '@/lib/api-client';
import { Event, EventListResponse, EndpointListResponse } from '@/lib/types';
import { Activity, RefreshCw, ExternalLink } from 'lucide-react';
import { formatDateTime, truncate, truncateAddress } from '@/lib/utils';

export default function EventsPage() {
  const [selectedEvent, setSelectedEvent] = useState<Event | null>(null);
  const [filterEndpoint, setFilterEndpoint] = useState('');
  const [filterStatus, setFilterStatus] = useState('');

  // Fetch events with real-time updates (every 3 seconds)
  const { data: eventsData, isLoading: eventsLoading, refetch } = useQuery<EventListResponse>({
    queryKey: ['events', filterEndpoint, filterStatus],
    queryFn: () => {
      let url = '/events?limit=50';
      if (filterEndpoint) url += `&endpoint_id=${filterEndpoint}`;
      if (filterStatus) url += `&status=${filterStatus}`;
      return api.get<EventListResponse>(url);
    },
    refetchInterval: 3000, // Poll every 3 seconds
  });

  // Fetch endpoints for filter
  const { data: endpointsData } = useQuery<EndpointListResponse>({
    queryKey: ['endpoints'],
    queryFn: () => api.get<EndpointListResponse>('/endpoints'),
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'delivered':
        return 'default';
      case 'failed':
        return 'destructive';
      case 'pending':
        return 'secondary';
      default:
        return 'secondary';
    }
  };

  const getEndpointName = (endpointId: string) => {
    return endpointsData?.endpoints.find(e => e.id === endpointId)?.name || 'Unknown';
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Events</h1>
          <p className="text-gray-500 dark:text-gray-400 mt-1">
            View all webhook delivery events and their status
          </p>
        </div>
        <Button onClick={() => refetch()} variant="outline">
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle>Filters</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            <div className="flex-1">
              <select
                className="w-full h-10 px-3 rounded-md border border-input bg-background"
                value={filterEndpoint}
                onChange={(e) => setFilterEndpoint(e.target.value)}
              >
                <option value="">All Endpoints</option>
                {endpointsData?.endpoints.map(endpoint => (
                  <option key={endpoint.id} value={endpoint.id}>{endpoint.name}</option>
                ))}
              </select>
            </div>
            <div className="flex-1">
              <select
                className="w-full h-10 px-3 rounded-md border border-input bg-background"
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value)}
              >
                <option value="">All Status</option>
                <option value="delivered">Delivered</option>
                <option value="pending">Pending</option>
                <option value="failed">Failed</option>
              </select>
            </div>
            {(filterEndpoint || filterStatus) && (
              <Button
                variant="outline"
                onClick={() => {
                  setFilterEndpoint('');
                  setFilterStatus('');
                }}
              >
                Clear Filters
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Events Table */}
      <Card>
        <CardHeader>
          <CardTitle>Event History</CardTitle>
          <CardDescription>
            All webhook events sent to your endpoints
            {eventsData && <span className="ml-2">• {eventsData.events.length} events</span>}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {eventsLoading ? (
            <div className="text-center py-12">
              <p className="text-muted-foreground">Loading events...</p>
            </div>
          ) : !eventsData?.events || eventsData.events.length === 0 ? (
            <div className="text-center py-12">
              <Activity className="w-12 h-12 mx-auto text-muted-foreground mb-3" />
              <p className="text-muted-foreground">No events yet</p>
              <p className="text-sm text-muted-foreground mt-1">
                Events will appear here once your endpoints start receiving blockchain events
              </p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Event Type</TableHead>
                  <TableHead>Endpoint</TableHead>
                  <TableHead>Chain</TableHead>
                  <TableHead>Contract</TableHead>
                  <TableHead>Block</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Attempts</TableHead>
                  <TableHead>Time</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {eventsData.events.map((event) => (
                  <TableRow key={event.id} className="cursor-pointer hover:bg-muted/50">
                    <TableCell className="font-medium font-mono text-xs">
                      {truncate(event.event_type, 30)}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {getEndpointName(event.endpoint_id)}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline">{event.chain_id}</Badge>
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      {truncateAddress(event.contract_address)}
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      {event.block_number}
                    </TableCell>
                    <TableCell>
                      <Badge variant={getStatusColor(event.status)}>
                        {event.status}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-center">
                      {event.attempts}
                    </TableCell>
                    <TableCell className="text-muted-foreground text-sm">
                      {formatDateTime(event.created_at)}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setSelectedEvent(event)}
                      >
                        <ExternalLink className="w-4 h-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Event Detail Dialog */}
      <Dialog open={!!selectedEvent} onOpenChange={(open) => !open && setSelectedEvent(null)}>
        <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Event Details</DialogTitle>
            <DialogDescription>
              Full information about this webhook event
            </DialogDescription>
          </DialogHeader>
          {selectedEvent && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Event Type</p>
                  <p className="text-sm font-mono mt-1">{selectedEvent.event_type}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Status</p>
                  <Badge variant={getStatusColor(selectedEvent.status)} className="mt-1">
                    {selectedEvent.status}
                  </Badge>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Chain ID</p>
                  <p className="text-sm mt-1">{selectedEvent.chain_id}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Block Number</p>
                  <p className="text-sm font-mono mt-1">{selectedEvent.block_number}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Transaction Hash</p>
                  <p className="text-sm font-mono mt-1 break-all">{selectedEvent.transaction_hash}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Log Index</p>
                  <p className="text-sm mt-1">{selectedEvent.log_index}</p>
                </div>
                <div className="col-span-2">
                  <p className="text-sm font-medium text-muted-foreground">Contract Address</p>
                  <p className="text-sm font-mono mt-1 break-all">{selectedEvent.contract_address}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Attempts</p>
                  <p className="text-sm mt-1">{selectedEvent.attempts}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Created At</p>
                  <p className="text-sm mt-1">{formatDateTime(selectedEvent.created_at)}</p>
                </div>
              </div>
              <div>
                <p className="text-sm font-medium text-muted-foreground mb-2">Event Payload</p>
                <pre className="bg-muted p-4 rounded-md overflow-x-auto text-xs">
                  {JSON.stringify(selectedEvent.payload, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
