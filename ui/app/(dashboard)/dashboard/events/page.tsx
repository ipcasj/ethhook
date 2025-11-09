'use client';

import { useState, useCallback } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { StatusBadge } from '@/components/ui/status-badge';
import { InfoBanner } from '@/components/ui/info-banner';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { api } from '@/lib/api-client';
import { Event, EventListResponse, EndpointListResponse } from '@/lib/types';
import { Activity, RefreshCw, ExternalLink, ChevronLeft, ChevronRight, BarChart3 } from 'lucide-react';
import { formatDateTime, truncateAddress } from '@/lib/utils';
import { AreaChart, Area, PieChart, Pie, Cell, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';

export default function EventsPage() {
  const [selectedEvent, setSelectedEvent] = useState<Event | null>(null);
  const [filterEndpoint, setFilterEndpoint] = useState('');
  const [filterStatus, setFilterStatus] = useState('');
  const [page, setPage] = useState(1);
  const perPage = 50;

  // Handler to update endpoint filter and reset pagination
  const handleFilterEndpointChange = useCallback((value: string) => {
    setFilterEndpoint(value);
    setPage(1);
  }, []);

  // Handler to update status filter and reset pagination
  const handleFilterStatusChange = useCallback((value: string) => {
    setFilterStatus(value);
    setPage(1);
  }, []);

  // Fetch events with real-time updates (every 3 seconds)
  const { data: eventsData, isLoading: eventsLoading, refetch } = useQuery<EventListResponse>({
    queryKey: ['events', filterEndpoint, page],
    queryFn: () => {
      const offset = (page - 1) * perPage;
      let url = `/events?limit=${perPage}&offset=${offset}`;
      if (filterEndpoint) url += `&endpoint_id=${filterEndpoint}`;
      // Note: Backend doesn't support status filtering, we'll filter client-side
      return api.get<EventListResponse>(url);
    },
    refetchInterval: 3000, // Poll every 3 seconds
  });

  // Helper to calculate event status
  const calculateEventStatus = (event: Event): 'delivered' | 'failed' | 'pending' => {
    if (event.status === 'delivered' || event.status === 'failed' || event.status === 'pending') {
      return event.status;
    }
    if ((event.successful_deliveries ?? 0) > 0) return 'delivered';
    if ((event.delivery_count ?? 0) > 0) return 'failed';
    return 'pending';
  };

  // Client-side filtering by status
  const filteredEvents = eventsData?.events.filter(event => {
    if (!filterStatus) return true;
    return calculateEventStatus(event) === filterStatus;
  }) ?? [];

  // Fetch endpoints for filter
  const { data: endpointsData } = useQuery<EndpointListResponse>({
    queryKey: ['endpoints'],
    queryFn: () => api.get<EndpointListResponse>('/endpoints'),
  });

  // Fetch analytics data for charts
  interface TimeseriesResponse {
    data_points: Array<{
      timestamp: string;
      event_count: number;
      delivery_count: number;
      successful_deliveries: number;
      failed_deliveries: number;
      success_rate: number;
      avg_latency_ms: number | null;
    }>;
    time_range: string;
    granularity: string;
  }

  interface ChainDistributionResponse {
    distributions: Array<{
      chain_id: number;
      chain_name: string;
      event_count: number;
      percentage: number;
    }>;
    total_events: number;
  }

  const { data: timeseriesData, isLoading: timeseriesLoading } = useQuery<TimeseriesResponse>({
    queryKey: ['timeseries-stats-7d'],
    queryFn: () => api.get<TimeseriesResponse>('/statistics/timeseries?time_range=7d&granularity=day'),
    refetchInterval: 60000, // Refresh every minute
  });

  const { data: chainData, isLoading: chainLoading } = useQuery<ChainDistributionResponse>({
    queryKey: ['chain-distribution'],
    queryFn: () => api.get<ChainDistributionResponse>('/statistics/chain-distribution'),
    refetchInterval: 60000, // Refresh every minute
  });

  const getChainName = (chainId: number | null | undefined, contractAddress?: string) => {
    if (chainId) {
      const chains: Record<number, string> = {
        1: 'Ethereum',
        10: 'Optimism',
        42161: 'Arbitrum',
        8453: 'Base',
      };
      return chains[chainId] || `Chain ${chainId}`;
    }
    
    // Infer chain from contract address if chain_id not available
    if (contractAddress) {
      const address = contractAddress.toLowerCase();
      
      // Arbitrum contracts
      if (address === '0xaf88d065e77c8cc2239327c5edb3a432268e5831' || // USDC
          address === '0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9' || // USDT
          address === '0x82af49447d8a07e3bd95bd0d56f35241523fbab1') { // WETH
        return 'Arbitrum';
      }
      
      // Optimism contracts
      if (address === '0x0b2c639c8136258849498ef890c8c58e5b5e95cd' || // USDC
          address === '0x94b008aa00579c1307b0ef2c499ad98a8ce58e58' || // USDT
          address === '0x4200000000000000000000000000000000000006') { // WETH
        return 'Optimism';
      }
      
      // Base contracts
      if (address === '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913') { // USDC
        return 'Base';
      }
      
      // Default to Ethereum for known mainnet contracts
      if (address === '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48' || // USDC
          address === '0xdac17f958d2ee523a2206206994597c13d831ec7' || // USDT
          address === '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2') { // WETH
        return 'Ethereum';
      }
    }
    
    return 'Unknown';
  };

  const getEventType = (topics?: string[]) => {
    if (!topics || topics.length === 0) return 'Unknown';
    
    const eventSignature = topics[0];
    
    // Common event signatures
    const eventTypes: Record<string, string> = {
      '0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef': 'Transfer',
      '0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c': 'Deposit',
      '0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65': 'Withdrawal',
      '0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925': 'Approval',
    };
    
    return eventTypes[eventSignature] || 'Event';
  };

  const getContractName = (contractAddress?: string) => {
    if (!contractAddress) return 'Unknown';
    
    const address = contractAddress.toLowerCase();
    
    // Known contracts across all chains
    const contracts: Record<string, string> = {
      // Ethereum
      '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48': 'USDC',
      '0xdac17f958d2ee523a2206206994597c13d831ec7': 'USDT',
      '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2': 'WETH',
      // Arbitrum
      '0xaf88d065e77c8cc2239327c5edb3a432268e5831': 'USDC',
      '0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9': 'USDT',
      '0x82af49447d8a07e3bd95bd0d56f35241523fbab1': 'WETH',
      // Optimism
      '0x0b2c639c8136258849498ef890c8c58e5b5e95cd': 'USDC',
      '0x94b008aa00579c1307b0ef2c499ad98a8ce58e58': 'USDT',
      '0x4200000000000000000000000000000000000006': 'WETH',
      // Base
      '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913': 'USDC',
    };
    
    return contracts[address] || 'Contract';
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

      <InfoBanner
        title="Event History & Monitoring"
        description="View and filter all webhook events captured from blockchain networks. Use the filters below to find specific transactions or monitor endpoint delivery status. Events update in real-time every 3 seconds."
        tips={[
          'Click any event to view full details including payload, delivery attempts, and response codes',
          'Use pagination to navigate through thousands of historical events',
          'Filter by endpoint to see events for specific webhooks',
          'Filter by status to quickly identify failed deliveries that require attention'
        ]}
        defaultCollapsed={true}
      />

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
                onChange={(e) => handleFilterEndpointChange(e.target.value)}
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
                onChange={(e) => handleFilterStatusChange(e.target.value)}
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
                  handleFilterEndpointChange('');
                  handleFilterStatusChange('');
                }}
              >
                Clear Filters
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Analytics Charts */}
      <div className="grid gap-4 lg:grid-cols-2">
        {/* Timeseries Chart */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="w-5 h-5 text-indigo-600" />
              Events Over Time (7 Days)
            </CardTitle>
            <CardDescription>Daily event volume for the last week</CardDescription>
          </CardHeader>
          <CardContent>
            {timeseriesLoading ? (
              <div className="h-[300px] flex items-center justify-center text-slate-400">
                Loading chart...
              </div>
            ) : timeseriesData && timeseriesData.data_points.length > 0 ? (
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={timeseriesData.data_points}>
                  <defs>
                    <linearGradient id="colorEventsPage" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#6366f1" stopOpacity={0.8}/>
                      <stop offset="95%" stopColor="#6366f1" stopOpacity={0}/>
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
                  <XAxis 
                    dataKey="timestamp" 
                    stroke="#64748b"
                    fontSize={12}
                    tickFormatter={(value) => {
                      const date = new Date(value);
                      return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
                    }}
                  />
                  <YAxis stroke="#64748b" fontSize={12} />
                  <Tooltip
                    contentStyle={{ backgroundColor: '#fff', border: '1px solid #e2e8f0', borderRadius: '8px' }}
                    labelFormatter={(value) => new Date(value).toLocaleDateString()}
                    formatter={(value: number) => [value.toLocaleString(), 'Events']}
                  />
                  <Area 
                    type="monotone" 
                    dataKey="event_count" 
                    stroke="#6366f1" 
                    fillOpacity={1} 
                    fill="url(#colorEventsPage)" 
                  />
                </AreaChart>
              </ResponsiveContainer>
            ) : (
              <div className="h-[300px] flex flex-col items-center justify-center text-slate-400">
                <BarChart3 className="w-12 h-12 mb-2 opacity-50" />
                <p>No event data available</p>
                <p className="text-sm mt-1">Start capturing events to see analytics</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Chain Distribution Pie Chart */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="w-5 h-5 text-purple-600" />
              Events by Blockchain
            </CardTitle>
            <CardDescription>Distribution across different networks</CardDescription>
          </CardHeader>
          <CardContent>
            {chainLoading ? (
              <div className="h-[300px] flex items-center justify-center text-slate-400">
                Loading chart...
              </div>
            ) : chainData && chainData.distributions.length > 0 ? (
              <ResponsiveContainer width="100%" height={300}>
                <PieChart>
                  <Pie
                    data={chainData.distributions}
                    cx="50%"
                    cy="50%"
                    labelLine={false}
                    label={(entry: any) => `${entry.chain_name} ${entry.percentage.toFixed(1)}%`}
                    outerRadius={100}
                    fill="#8884d8"
                    dataKey="event_count"
                  >
                    {chainData.distributions.map((entry, index) => {
                      const colors = ['#6366f1', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981'];
                      return <Cell key={`cell-${index}`} fill={colors[index % colors.length]} />;
                    })}
                  </Pie>
                  <Tooltip 
                    contentStyle={{ backgroundColor: '#fff', border: '1px solid #e2e8f0', borderRadius: '8px' }}
                    formatter={(value: number, name: string, props?: any) => [
                      `${value.toLocaleString()} events (${props?.payload?.percentage?.toFixed(1) || 0}%)`,
                      props?.payload?.chain_name || name
                    ]}
                  />
                  <Legend />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="h-[300px] flex flex-col items-center justify-center text-slate-400">
                <Activity className="w-12 h-12 mb-2 opacity-50" />
                <p>No chain data available</p>
                <p className="text-sm mt-1">Configure endpoints to monitor chains</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Events Table */}
      <Card>
        <CardHeader>
          <CardTitle>Event History</CardTitle>
          <CardDescription>
            All webhook events sent to your endpoints
            {eventsData && (
              <span className="ml-2">
                • {filterStatus ? `${filteredEvents.length} of ` : ''}{eventsData.total.toLocaleString()} total events
              </span>
            )}
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
          ) : filteredEvents.length === 0 ? (
            <div className="text-center py-12">
              <Activity className="w-12 h-12 mx-auto text-muted-foreground mb-3" />
              <p className="text-muted-foreground">No {filterStatus} events found</p>
              <p className="text-sm text-muted-foreground mt-1">
                {filterStatus === 'failed' && 'All events have been delivered successfully'}
                {filterStatus === 'pending' && 'No events are currently pending delivery'}
                {filterStatus === 'delivered' && 'No delivered events to show'}
              </p>
              <Button 
                variant="outline" 
                size="sm" 
                className="mt-4"
                onClick={() => handleFilterStatusChange('')}
              >
                Clear Filters
              </Button>
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
                {filteredEvents.map((event) => {
                  const eventStatus = calculateEventStatus(event);
                  return (
                  <TableRow 
                    key={event.id} 
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setSelectedEvent(event)}
                  >
                    <TableCell className="font-medium font-mono text-xs">
                      {getEventType(event.topics)}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {event.endpoint_name || `${getContractName(event.contract_address)} Monitoring`}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline">{getChainName(event.chain_id, event.contract_address)}</Badge>
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      {truncateAddress(event.contract_address)}
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      {event.block_number.toLocaleString()}
                    </TableCell>
                    <TableCell>
                      <StatusBadge status={eventStatus} size="sm" showIcon={true} />
                    </TableCell>
                    <TableCell className="text-center">
                      {event.attempts ?? event.delivery_count ?? 0}
                    </TableCell>
                    <TableCell className="text-muted-foreground text-sm">
                      {formatDateTime(event.created_at || event.ingested_at || new Date().toISOString())}
                    </TableCell>
                    <TableCell className="text-right" onClick={(e) => e.stopPropagation()}>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setSelectedEvent(event)}
                      >
                        <ExternalLink className="w-4 h-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          )}
          
          {/* Pagination Controls */}
          {eventsData && eventsData.total > perPage && (
            <div className="flex items-center justify-between px-2 py-4">
              <div className="text-sm text-muted-foreground">
                Showing {((page - 1) * perPage) + 1} to {Math.min(page * perPage, eventsData.total)} of {eventsData.total} events
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(p => Math.max(1, p - 1))}
                  disabled={page === 1}
                >
                  <ChevronLeft className="w-4 h-4 mr-1" />
                  Previous
                </Button>
                <div className="text-sm text-muted-foreground">
                  Page {page} of {Math.ceil(eventsData.total / perPage)}
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(p => Math.min(Math.ceil(eventsData.total / perPage), p + 1))}
                  disabled={page >= Math.ceil(eventsData.total / perPage)}
                >
                  Next
                  <ChevronRight className="w-4 h-4 ml-1" />
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Event Detail Dialog */}
      <Dialog open={!!selectedEvent} onOpenChange={(open) => !open && setSelectedEvent(null)}>
        <DialogContent className="max-w-3xl max-h-[85vh]">
          <DialogHeader>
            <DialogTitle>Event Details</DialogTitle>
            <DialogDescription>
              Full information about this webhook event
            </DialogDescription>
          </DialogHeader>
          {selectedEvent && (
            <div className="space-y-4 overflow-y-auto max-h-[calc(85vh-120px)] pr-2">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Event Type</p>
                  <p className="text-sm font-mono mt-1">{getEventType(selectedEvent.topics)}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Status</p>
                  <div className="mt-1">
                    <StatusBadge status={calculateEventStatus(selectedEvent)} size="sm" showIcon={true} />
                  </div>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Chain ID</p>
                  <p className="text-sm mt-1">{selectedEvent.chain_id || 'N/A'}</p>
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
                  <p className="text-sm mt-1">{selectedEvent.attempts ?? selectedEvent.delivery_count ?? 0}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-muted-foreground">Created At</p>
                  <p className="text-sm mt-1">{formatDateTime(selectedEvent.created_at || selectedEvent.ingested_at || new Date().toISOString())}</p>
                </div>
              </div>
              <div>
                <p className="text-sm font-medium text-muted-foreground mb-2">Event Data</p>
                <pre className="bg-muted p-4 rounded-md overflow-x-auto text-xs">
                  {JSON.stringify({ topics: selectedEvent.topics, data: selectedEvent.data }, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
