'use client';

import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useParams, useRouter, useSearchParams } from 'next/navigation';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { FilterBar, FilterState } from '@/components/ui/filter-bar';
import { api } from '@/lib/api-client';
import { 
  ArrowLeft, 
  Activity, 
  CheckCircle2, 
  XCircle, 
  Clock, 
  Zap,
  TrendingUp,
  Server,
  AlertCircle
} from 'lucide-react';
import { formatDateTime, formatNumber } from '@/lib/utils';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar, PieChart, Pie, Cell, LineChart, Line } from 'recharts';

// Types
interface ApplicationStatistics {
  application_id: string;
  events_total: number;
  events_24h: number;
  endpoints_count: number;
  active_endpoints: number;
  total_deliveries: number;
  successful_deliveries: number;
  failed_deliveries: number;
  success_rate: number;
  avg_delivery_time_ms: number;
  min_delivery_time_ms: number | null;
  max_delivery_time_ms: number | null;
  first_event_at: string | null;
  last_event_at: string | null;
}

interface TimeseriesDataPoint {
  timestamp: string;
  event_count: number;
  delivery_count: number;
  successful_deliveries: number;
  failed_deliveries: number;
  success_rate: number;
  avg_latency_ms: number | null;
}

interface TimeseriesResponse {
  data_points: TimeseriesDataPoint[];
  time_range: string;
  granularity: string;
}

interface EndpointPerformance {
  endpoint_id: string;
  name: string;
  url: string;
  events_count: number;
  success_rate: number;
  avg_latency_ms: number | null;
  last_event_at: string | null;
}

interface EndpointsPerformanceResponse {
  endpoints: EndpointPerformance[];
}

interface Application {
  id: string;
  name: string;
  description: string | null;
}

export default function ApplicationDetailPage() {
  const params = useParams();
  const router = useRouter();
  const searchParams = useSearchParams();
  const appId = params.id as string;
  const [timeRange, setTimeRange] = useState<'24h' | '7d' | '30d'>('7d');

  // Initialize filters from URL
  const [filters, setFilters] = useState<FilterState>(() => {
    const chainIds = searchParams.get('chain_ids')?.split(',').map(Number).filter(Boolean) || [];
    const successParam = searchParams.get('success');
    const success = successParam === 'true' ? true : successParam === 'false' ? false : undefined;
    const startDate = searchParams.get('start_date') ? new Date(searchParams.get('start_date')!) : undefined;
    const endDate = searchParams.get('end_date') ? new Date(searchParams.get('end_date')!) : undefined;
    
    return { chainIds, success, startDate, endDate };
  });

  // Update URL when filters change
  const updateURL = (newFilters: FilterState) => {
    const params = new URLSearchParams(searchParams.toString());
    
    if (newFilters.chainIds.length > 0) {
      params.set('chain_ids', newFilters.chainIds.join(','));
    } else {
      params.delete('chain_ids');
    }
    
    if (newFilters.success !== undefined) {
      params.set('success', String(newFilters.success));
    } else {
      params.delete('success');
    }
    
    if (newFilters.startDate) {
      params.set('start_date', newFilters.startDate.toISOString());
    } else {
      params.delete('start_date');
    }
    
    if (newFilters.endDate) {
      params.set('end_date', newFilters.endDate.toISOString());
    } else {
      params.delete('end_date');
    }
    
    router.push(`?${params.toString()}`, { scroll: false });
  };

  const handleFiltersChange = (newFilters: FilterState) => {
    setFilters(newFilters);
    updateURL(newFilters);
  };

  // Fetch application details
  const { data: application } = useQuery<Application>({
    queryKey: ['application', appId],
    queryFn: () => api.get<Application>(`/applications/${appId}`),
  });

  // Fetch application statistics
  const { data: stats, isLoading: statsLoading } = useQuery<ApplicationStatistics>({
    queryKey: ['application-statistics', appId],
    queryFn: () => api.get<ApplicationStatistics>(`/applications/${appId}/statistics`),
    refetchInterval: 30000, // Refresh every 30s
  });

  // Fetch timeseries data with filters
  const { data: timeseries } = useQuery<TimeseriesResponse>({
    queryKey: ['application-timeseries', appId, timeRange, filters],
    queryFn: () => {
      const params = new URLSearchParams({
        time_range: timeRange,
        granularity: 'auto',
      });
      
      if (filters.chainIds.length > 0) {
        params.set('chain_ids', filters.chainIds.join(','));
      }
      if (filters.success !== undefined) {
        params.set('success', String(filters.success));
      }
      if (filters.startDate) {
        params.set('start_date', filters.startDate.toISOString());
      }
      if (filters.endDate) {
        params.set('end_date', filters.endDate.toISOString());
      }
      
      return api.get<TimeseriesResponse>(`/applications/${appId}/timeseries?${params.toString()}`);
    },
    refetchInterval: 60000, // Refresh every 60s
  });

  // Fetch endpoints performance
  const { data: endpointsData } = useQuery<EndpointsPerformanceResponse>({
    queryKey: ['application-endpoints-performance', appId],
    queryFn: () => api.get<EndpointsPerformanceResponse>(`/applications/${appId}/endpoints/performance`),
    refetchInterval: 30000,
  });

  // Format chart data
  const chartData = timeseries?.data_points.map(point => ({
    time: new Date(point.timestamp).toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric',
      hour: timeRange === '24h' ? 'numeric' : undefined
    }),
    events: point.event_count,
    deliveries: point.delivery_count,
    successRate: point.success_rate,
    latency: point.avg_latency_ms || 0,
  })) || [];

  // Endpoint performance chart data
  const endpointChartData = endpointsData?.endpoints.slice(0, 10).map(ep => ({
    name: ep.name.length > 20 ? ep.name.substring(0, 20) + '...' : ep.name,
    events: ep.events_count,
    successRate: ep.success_rate,
    latency: ep.avg_latency_ms || 0,
  })) || [];

  // Success/Failure pie chart data
  const deliveryStatusData = stats ? [
    { name: 'Success', value: stats.successful_deliveries, color: '#10b981' },
    { name: 'Failed', value: stats.failed_deliveries, color: '#ef4444' },
  ] : [];

  if (statsLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto"></div>
          <p className="mt-4 text-muted-foreground">Loading application details...</p>
        </div>
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <AlertCircle className="h-12 w-12 text-red-500 mx-auto mb-4" />
          <h2 className="text-xl font-semibold mb-2">Application Not Found</h2>
          <p className="text-muted-foreground mb-4">The requested application could not be found.</p>
          <Button onClick={() => router.push('/dashboard/applications')}>
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to Applications
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => router.push('/dashboard/applications')}
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <h1 className="text-3xl font-bold">{application?.name || 'Application Details'}</h1>
            {application?.description && (
              <p className="text-muted-foreground mt-1">{application.description}</p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => router.push(`/dashboard/applications/${appId}/settings`)}
          >
            Settings
          </Button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Events</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatNumber(stats.events_total)}</div>
            <p className="text-xs text-muted-foreground">
              {formatNumber(stats.events_24h)} in last 24h
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <CheckCircle2 className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.success_rate.toFixed(1)}%</div>
            <p className="text-xs text-muted-foreground">
              {formatNumber(stats.successful_deliveries)} / {formatNumber(stats.total_deliveries)} deliveries
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg Response Time</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.avg_delivery_time_ms.toFixed(1)}ms</div>
            <p className="text-xs text-muted-foreground">
              {stats.min_delivery_time_ms?.toFixed(1) || 'N/A'}ms min / {stats.max_delivery_time_ms?.toFixed(1) || 'N/A'}ms max
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Endpoints</CardTitle>
            <Server className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.active_endpoints}</div>
            <p className="text-xs text-muted-foreground">
              {stats.endpoints_count} total configured
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Filters and Time Range */}
      <Card className="bg-slate-50/50">
        <CardContent className="pt-6">
          <div className="flex flex-col lg:flex-row gap-4 items-start lg:items-center justify-between">
            {/* Filter Bar */}
            <FilterBar
              filters={filters}
              onFiltersChange={handleFiltersChange}
              className="flex-1"
            />
            
            {/* Time Range Selector */}
            <div className="flex gap-2">
              <Button
                variant={timeRange === '24h' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTimeRange('24h')}
              >
                24h
              </Button>
              <Button
                variant={timeRange === '7d' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTimeRange('7d')}
              >
                7d
              </Button>
              <Button
                variant={timeRange === '30d' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTimeRange('30d')}
              >
                30d
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Charts Row 1 */}
      <div className="grid gap-4 md:grid-cols-2">
        {/* Events Over Time */}
        <Card>
          <CardHeader>
            <CardTitle>Events Over Time</CardTitle>
            <CardDescription>Event volume for this application</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={250}>
              <AreaChart data={chartData}>
                <defs>
                  <linearGradient id="colorEvents" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" fontSize={12} />
                <YAxis fontSize={12} />
                <Tooltip />
                <Area 
                  type="monotone" 
                  dataKey="events" 
                  stroke="#3b82f6" 
                  fillOpacity={1} 
                  fill="url(#colorEvents)" 
                />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Success Rate Trend */}
        <Card>
          <CardHeader>
            <CardTitle>Success Rate Trend</CardTitle>
            <CardDescription>Delivery success rate over time</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={250}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" fontSize={12} />
                <YAxis fontSize={12} domain={[0, 100]} />
                <Tooltip formatter={(value: number) => `${value.toFixed(1)}%`} />
                <Line 
                  type="monotone" 
                  dataKey="successRate" 
                  stroke="#10b981" 
                  strokeWidth={2}
                  dot={{ r: 4 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      {/* Charts Row 2 */}
      <div className="grid gap-4 md:grid-cols-2">
        {/* Endpoint Performance Comparison */}
        <Card>
          <CardHeader>
            <CardTitle>Endpoint Performance</CardTitle>
            <CardDescription>Event volume by endpoint (top 10)</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={endpointChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="name" fontSize={10} angle={-45} textAnchor="end" height={80} />
                <YAxis fontSize={12} />
                <Tooltip />
                <Bar dataKey="events" fill="#3b82f6" />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Delivery Status Distribution */}
        <Card>
          <CardHeader>
            <CardTitle>Delivery Status</CardTitle>
            <CardDescription>Success vs. failed deliveries</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-center">
              <ResponsiveContainer width="100%" height={250}>
                <PieChart>
                  <Pie
                    data={deliveryStatusData}
                    cx="50%"
                    cy="50%"
                    labelLine={false}
                    label={(entry: unknown) => {
                      const data = entry as { name: string; value: number };
                      return `${data.name}: ${formatNumber(data.value)}`;
                    }}
                    outerRadius={80}
                    fill="#8884d8"
                    dataKey="value"
                  >
                    {deliveryStatusData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                  <Tooltip />
                </PieChart>
              </ResponsiveContainer>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Endpoints Table */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Endpoints</CardTitle>
              <CardDescription>Performance metrics for all endpoints</CardDescription>
            </div>
            <Button
              size="sm"
              onClick={() => router.push(`/dashboard/applications/${appId}/endpoints`)}
            >
              Manage Endpoints
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>URL</TableHead>
                <TableHead className="text-right">Events</TableHead>
                <TableHead className="text-right">Success Rate</TableHead>
                <TableHead className="text-right">Avg Latency</TableHead>
                <TableHead className="text-right">Last Event</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {endpointsData?.endpoints.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                    No endpoints configured for this application
                  </TableCell>
                </TableRow>
              ) : (
                endpointsData?.endpoints.map((endpoint) => (
                  <TableRow 
                    key={endpoint.endpoint_id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => router.push(`/dashboard/endpoints/${endpoint.endpoint_id}`)}
                  >
                    <TableCell className="font-medium">{endpoint.name}</TableCell>
                    <TableCell className="text-xs text-muted-foreground max-w-xs truncate">
                      {endpoint.url}
                    </TableCell>
                    <TableCell className="text-right">{formatNumber(endpoint.events_count)}</TableCell>
                    <TableCell className="text-right">
                      <span className={endpoint.success_rate >= 95 ? 'text-green-600' : endpoint.success_rate >= 80 ? 'text-yellow-600' : 'text-red-600'}>
                        {endpoint.success_rate.toFixed(1)}%
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      {endpoint.avg_latency_ms ? `${endpoint.avg_latency_ms.toFixed(1)}ms` : 'N/A'}
                    </TableCell>
                    <TableCell className="text-right text-xs text-muted-foreground">
                      {endpoint.last_event_at ? formatDateTime(endpoint.last_event_at) : 'Never'}
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}
