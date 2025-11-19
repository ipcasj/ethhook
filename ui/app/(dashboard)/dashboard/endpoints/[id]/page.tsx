'use client';

import { useParams, useRouter, useSearchParams } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { FilterBar, FilterState } from '@/components/ui/filter-bar';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  ArrowLeft,
  Activity,
  CheckCircle2,
  XCircle,
  Clock,
  TrendingUp,
  AlertCircle,
  ChevronDown,
  ChevronUp,
  Settings,
  ExternalLink,
} from 'lucide-react';
import {
  AreaChart,
  Area,
  LineChart,
  Line,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { formatDateTime, formatNumber } from '@/lib/utils';

// TypeScript interfaces
interface EndpointStatistics {
  endpoint_id: string;
  name: string;
  webhook_url: string;
  status: string;
  events_total: number;
  events_24h: number;
  deliveries_total: number;
  successful_deliveries: number;
  failed_deliveries: number;
  success_rate: number;
  avg_delivery_time_ms: number | null;
  p50_latency_ms: number | null;
  p95_latency_ms: number | null;
  p99_latency_ms: number | null;
  health_score: number;
  first_event_at: string | null;
  last_event_at: string | null;
}

interface SimpleTimeseriesDataPoint {
  timestamp: string;
  count: number;
}

interface SimpleTimeseriesResponse {
  data_points: SimpleTimeseriesDataPoint[];
  time_range: string;
  granularity: string;
}

interface DeliveryAttempt {
  id: string;
  event_id: string;
  attempt_number: number;
  http_status_code: number | null;
  success: boolean;
  duration_ms: number | null;
  attempted_at: string;
  error_message: string | null;
  response_body: string | null;
}

interface DeliveriesResponse {
  deliveries: DeliveryAttempt[];
  total: number;
  limit: number;
  offset: number;
}

interface Endpoint {
  id: string;
  application_id: string;
  name: string;
  webhook_url: string;
}

export default function EndpointDetailPage() {
  const params = useParams();
  const router = useRouter();
  const searchParams = useSearchParams();
  const endpointId = params.id as string;

  const [timeRange, setTimeRange] = useState<'24h' | '7d' | '30d'>('24h');
  const [deliveryFilter, setDeliveryFilter] = useState<'all' | 'success' | 'failed'>('all');
  const [expandedDelivery, setExpandedDelivery] = useState<string | null>(null);

  // Initialize filters from URL (note: chain filtering less relevant for single endpoint)
  const [filters, setFilters] = useState<FilterState>(() => {
    const successParam = searchParams.get('success');
    const success = successParam === 'true' ? true : successParam === 'false' ? false : undefined;
    const startDate = searchParams.get('start_date') ? new Date(searchParams.get('start_date')!) : undefined;
    const endDate = searchParams.get('end_date') ? new Date(searchParams.get('end_date')!) : undefined;
    
    return { chainIds: [], success, startDate, endDate };
  });

  // Update URL when filters change
  const updateURL = (newFilters: FilterState) => {
    const params = new URLSearchParams(searchParams.toString());
    
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

  // Fetch endpoint details
  const { data: endpoint, isLoading: endpointLoading } = useQuery<Endpoint>({
    queryKey: ['endpoint', endpointId],
    queryFn: async () => {
      const token = localStorage.getItem('token');
      const response = await fetch(`http://104.248.15.178:3000/api/v1/endpoints/${endpointId}`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!response.ok) throw new Error('Failed to fetch endpoint');
      return response.json();
    },
    refetchInterval: 30000,
  });

  // Fetch endpoint statistics
  const { data: statistics, isLoading: statsLoading } = useQuery<EndpointStatistics>({
    queryKey: ['endpoint-statistics', endpointId],
    queryFn: async () => {
      const token = localStorage.getItem('token');
      const response = await fetch(`http://104.248.15.178:3000/api/v1/endpoints/${endpointId}/statistics`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!response.ok) throw new Error('Failed to fetch statistics');
      return response.json();
    },
    refetchInterval: 30000,
  });

  // Fetch timeseries data with filters
  const { data: timeseries } = useQuery<SimpleTimeseriesResponse>({
    queryKey: ['endpoint-timeseries', endpointId, timeRange, filters],
    queryFn: async () => {
      const token = localStorage.getItem('token');
      const params = new URLSearchParams({
        time_range: timeRange,
        granularity: 'auto',
      });
      
      if (filters.success !== undefined) {
        params.set('success', String(filters.success));
      }
      if (filters.startDate) {
        params.set('start_date', filters.startDate.toISOString());
      }
      if (filters.endDate) {
        params.set('end_date', filters.endDate.toISOString());
      }
      
      const response = await fetch(
        `http://104.248.15.178:3000/api/v1/endpoints/${endpointId}/timeseries?${params.toString()}`,
        {
          headers: { Authorization: `Bearer ${token}` },
        }
      );
      if (!response.ok) throw new Error('Failed to fetch timeseries');
      return response.json();
    },
    refetchInterval: 60000,
  });

  // Fetch deliveries with pagination
  const { data: deliveries } = useQuery<DeliveriesResponse>({
    queryKey: ['endpoint-deliveries', endpointId, deliveryFilter],
    queryFn: async () => {
      const token = localStorage.getItem('token');
      const response = await fetch(
        `http://104.248.15.178:3000/api/v1/endpoints/${endpointId}/deliveries?limit=20&offset=0&status=${deliveryFilter}`,
        {
          headers: { Authorization: `Bearer ${token}` },
        }
      );
      if (!response.ok) throw new Error('Failed to fetch deliveries');
      return response.json();
    },
    refetchInterval: 30000,
  });

  // Transform timeseries data for chart
  const chartData = timeseries?.data_points.map((point) => ({
    time: new Date(point.timestamp).toLocaleTimeString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }),
    events: point.count,
  })) || [];

  // Calculate latency distribution data
  const latencyData = statistics
    ? [
        { percentile: 'P50', latency: statistics.p50_latency_ms || 0 },
        { percentile: 'P95', latency: statistics.p95_latency_ms || 0 },
        { percentile: 'P99', latency: statistics.p99_latency_ms || 0 },
      ]
    : [];

  if (endpointLoading || statsLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-gray-500">Loading endpoint details...</div>
      </div>
    );
  }

  if (!endpoint || !statistics) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="flex items-center space-x-2 text-red-500">
          <AlertCircle className="h-5 w-5" />
          <span>Failed to load endpoint details</span>
        </div>
      </div>
    );
  }

  // Health score color
  const getHealthScoreColor = (score: number) => {
    if (score >= 80) return 'text-green-600';
    if (score >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getHealthScoreBg = (score: number) => {
    if (score >= 80) return 'bg-green-50';
    if (score >= 60) return 'bg-yellow-50';
    return 'bg-red-50';
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <button
            onClick={() => router.push(`/dashboard/applications/${endpoint.application_id}`)}
            className="p-2 hover:bg-gray-100 rounded-lg transition-colors"
          >
            <ArrowLeft className="h-5 w-5" />
          </button>
          <div>
            <h1 className="text-2xl font-bold text-gray-900">{endpoint.name}</h1>
            <p className="text-sm text-gray-500 flex items-center space-x-2 mt-1">
              <span>{endpoint.webhook_url}</span>
              <a
                href={endpoint.webhook_url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 hover:text-blue-700"
              >
                <ExternalLink className="h-4 w-4" />
              </a>
            </p>
          </div>
        </div>
        <button
          onClick={() => router.push(`/dashboard/endpoints/${endpointId}/settings`)}
          className="px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors flex items-center space-x-2"
        >
          <Settings className="h-4 w-4" />
          <span>Settings</span>
        </button>
      </div>

      {/* Health Score Badge */}
      <div className={`inline-flex items-center space-x-2 px-4 py-2 rounded-lg ${getHealthScoreBg(statistics.health_score)}`}>
        <Activity className={`h-5 w-5 ${getHealthScoreColor(statistics.health_score)}`} />
        <span className={`font-semibold ${getHealthScoreColor(statistics.health_score)}`}>
          Health Score: {statistics.health_score.toFixed(1)}/100
        </span>
      </div>

      {/* Metric Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Total Events */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Total Events</p>
              <p className="text-2xl font-bold text-gray-900 mt-2">{formatNumber(statistics.events_total)}</p>
              <p className="text-xs text-gray-500 mt-1">{statistics.events_24h} in last 24h</p>
            </div>
            <Activity className="h-8 w-8 text-blue-600" />
          </div>
        </div>

        {/* Success Rate */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Success Rate</p>
              <p className="text-2xl font-bold text-gray-900 mt-2">{statistics.success_rate.toFixed(1)}%</p>
              <p className="text-xs text-gray-500 mt-1">
                {formatNumber(statistics.successful_deliveries)} / {formatNumber(statistics.deliveries_total)}
              </p>
            </div>
            <CheckCircle2 className={`h-8 w-8 ${statistics.success_rate >= 95 ? 'text-green-600' : statistics.success_rate >= 80 ? 'text-yellow-600' : 'text-red-600'}`} />
          </div>
        </div>

        {/* Avg Response Time */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Avg Response Time</p>
              <p className="text-2xl font-bold text-gray-900 mt-2">
                {statistics.avg_delivery_time_ms ? `${statistics.avg_delivery_time_ms.toFixed(0)}ms` : 'N/A'}
              </p>
              <p className="text-xs text-gray-500 mt-1">
                P95: {statistics.p95_latency_ms ? `${statistics.p95_latency_ms.toFixed(0)}ms` : 'N/A'}
              </p>
            </div>
            <Clock className="h-8 w-8 text-purple-600" />
          </div>
        </div>

        {/* Status */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Endpoint Status</p>
              <p className="text-2xl font-bold text-gray-900 mt-2 capitalize">{statistics.status}</p>
              <p className="text-xs text-gray-500 mt-1">
                {statistics.last_event_at ? `Last event: ${formatDateTime(statistics.last_event_at)}` : 'No events yet'}
              </p>
            </div>
            <TrendingUp className="h-8 w-8 text-indigo-600" />
          </div>
        </div>
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
              {(['24h', '7d', '30d'] as const).map((range) => (
                <Button
                  key={range}
                  onClick={() => setTimeRange(range)}
                  variant={timeRange === range ? 'default' : 'outline'}
                  size="sm"
                >
                  {range === '24h' ? '24h' : range === '7d' ? '7d' : '30d'}
                </Button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Charts Row 1 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Events Over Time */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Events Over Time</h3>
          <ResponsiveContainer width="100%" height={300}>
            <AreaChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="time" style={{ fontSize: '12px' }} />
              <YAxis style={{ fontSize: '12px' }} />
              <Tooltip />
              <Area type="monotone" dataKey="events" stroke="#3b82f6" fill="#3b82f6" fillOpacity={0.2} />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {/* Latency Distribution */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Latency Percentiles</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={latencyData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="percentile" style={{ fontSize: '12px' }} />
              <YAxis style={{ fontSize: '12px' }} label={{ value: 'ms', angle: -90, position: 'insideLeft' }} />
              <Tooltip />
              <Bar dataKey="latency" fill="#8b5cf6" />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Delivery History */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-gray-900">Delivery History</h3>
            <div className="flex space-x-2">
              {(['all', 'success', 'failed'] as const).map((filter) => (
                <button
                  key={filter}
                  onClick={() => setDeliveryFilter(filter)}
                  className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${
                    deliveryFilter === filter
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                  }`}
                >
                  {filter.charAt(0).toUpperCase() + filter.slice(1)}
                </button>
              ))}
            </div>
          </div>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Attempt
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  HTTP Code
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Duration
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Attempted At
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Details
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {deliveries?.deliveries.map((delivery) => (
                <>
                  <tr key={delivery.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      {delivery.success ? (
                        <CheckCircle2 className="h-5 w-5 text-green-600" />
                      ) : (
                        <XCircle className="h-5 w-5 text-red-600" />
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      #{delivery.attempt_number}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {delivery.http_status_code || 'N/A'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {delivery.duration_ms ? `${delivery.duration_ms}ms` : 'N/A'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {formatDateTime(delivery.attempted_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      <button
                        onClick={() => setExpandedDelivery(expandedDelivery === delivery.id ? null : delivery.id)}
                        className="text-blue-600 hover:text-blue-700 flex items-center space-x-1"
                      >
                        {expandedDelivery === delivery.id ? (
                          <>
                            <ChevronUp className="h-4 w-4" />
                            <span>Hide</span>
                          </>
                        ) : (
                          <>
                            <ChevronDown className="h-4 w-4" />
                            <span>Show</span>
                          </>
                        )}
                      </button>
                    </td>
                  </tr>
                  {expandedDelivery === delivery.id && (
                    <tr>
                      <td colSpan={6} className="px-6 py-4 bg-gray-50">
                        <div className="space-y-2">
                          {delivery.error_message && (
                            <div>
                              <p className="text-xs font-medium text-gray-700">Error Message:</p>
                              <p className="text-sm text-red-600 mt-1">{delivery.error_message}</p>
                            </div>
                          )}
                          {delivery.response_body && (
                            <div>
                              <p className="text-xs font-medium text-gray-700">Response Body:</p>
                              <pre className="text-xs text-gray-600 mt-1 p-2 bg-white rounded border border-gray-200 overflow-x-auto">
                                {delivery.response_body}
                              </pre>
                            </div>
                          )}
                        </div>
                      </td>
                    </tr>
                  )}
                </>
              ))}
            </tbody>
          </table>
          {(!deliveries || deliveries.deliveries.length === 0) && (
            <div className="text-center py-12 text-gray-500">No delivery attempts found</div>
          )}
        </div>
      </div>
    </div>
  );
}
