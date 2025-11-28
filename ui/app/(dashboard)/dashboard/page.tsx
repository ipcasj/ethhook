'use client';

import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { CompactMetricCard } from '@/components/ui/compact-metric-card';
import { CompactEventTable } from '@/components/ui/compact-event-table';
import { InfoBanner } from '@/components/ui/info-banner';
import { InsightCard } from '@/components/ui/insight-card';
// import { useWebSocket, WsStatsMessage } from '@/hooks/useWebSocket'; // Disabled - backend uses SSE
import { api } from '@/lib/api-client';
import { DashboardStats, Event, User } from '@/lib/types';
import { Activity, Box, Webhook, CheckCircle, Plus, TrendingUp, Zap, Clock, BarChart3 } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { AreaChart, Area, PieChart, Pie, Cell, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';

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

type TimeRange = '24h' | '7d' | '30d';

export default function DashboardPage() {
  const router = useRouter();
  const [timeRange, setTimeRange] = useState<TimeRange>('24h');

  // Fetch user profile to check admin status
  const { data: userProfile } = useQuery({
    queryKey: ['user-profile'],
    queryFn: () => api.get<User>('/users/profile'),
    staleTime: 5 * 60 * 1000, // Cache for 5 minutes
  });

  const isAdmin = userProfile?.is_admin ?? false;

  // Fetch initial stats via REST API
  const { data: stats, isLoading: statsLoading } = useQuery<DashboardStats>({
    queryKey: ['dashboard-stats'],
    queryFn: () => api.get<DashboardStats>('/statistics/dashboard'),
    refetchInterval: 30000, // Refresh every 30 seconds as fallback
  });

  // WebSocket disabled - backend uses SSE at /stats/stream instead
  // TODO: Implement SSE support for real-time stats
  // const { messages: statsMessages } = useWebSocket('/ws/stats');
  
  // Use REST API stats directly (no WebSocket merge)
  const mergedStats: DashboardStats | undefined = stats;

  const { data: recentEvents, isLoading: eventsLoading } = useQuery<{ events: Event[] }>({
    queryKey: ['recent-events'],
    queryFn: () => api.get<{ events: Event[] }>('/events?limit=15'),
    refetchInterval: 5000, // Refresh every 5 seconds
  });

  // Fetch timeseries data
  const granularity = timeRange === '24h' ? 'hour' : 'day';
  const { data: timeseriesData, isLoading: timeseriesLoading } = useQuery<TimeseriesResponse>({
    queryKey: ['timeseries-stats', timeRange],
    queryFn: () => api.get<TimeseriesResponse>(`/statistics/timeseries?time_range=${timeRange}&granularity=${granularity}`),
    refetchInterval: 60000, // Refresh every minute
  });

  // Fetch chain distribution
  const { data: chainData, isLoading: chainLoading } = useQuery<ChainDistributionResponse>({
    queryKey: ['chain-distribution'],
    queryFn: () => api.get<ChainDistributionResponse>('/statistics/chain-distribution'),
    refetchInterval: 60000, // Refresh every minute
  });

  // Calculate insights
  const successRate = stats?.success_rate ?? 0;
  const avgDeliveryTime = stats?.avg_delivery_time_ms ?? 0;

  return (
    <div className="space-y-3 sm:space-y-4 px-2 sm:px-0">
      {/* Page header - Compact on mobile */}
      <div>
        <h1 className="text-2xl sm:text-3xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
          Dashboard
        </h1>
        <p className="text-slate-600 mt-1 text-sm sm:text-base">
          Real-time overview of your webhook infrastructure
        </p>
      </div>

      {/* Info Banner */}
      <InfoBanner
        title="Welcome to Your Webhook Dashboard"
        description="Monitor blockchain events, track webhook deliveries, and manage your endpoints from this central hub. All metrics update in real-time."
        tips={[
          'Create applications to organize your endpoints by project or environment',
          'Configure endpoints to receive specific events from smart contracts',
          'Monitor delivery success rates and identify issues quickly',
          'Use filters on the Events page to find specific transactions'
        ]}
        defaultCollapsed={false}
      />

      {/* Quick actions - Responsive */}
      <div className="flex flex-col sm:flex-row gap-2 sm:gap-3">
        <Link href="/dashboard/applications" className="flex-1 sm:flex-initial">
          <Button className="w-full sm:w-auto bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-indigo-500/30 text-sm sm:text-base">
            <Plus className="w-4 h-4 mr-2" />
            Create Application
          </Button>
        </Link>
        <Link href="/dashboard/endpoints" className="flex-1 sm:flex-initial">
          <Button variant="outline" className="w-full sm:w-auto border-indigo-200 text-indigo-700 hover:bg-indigo-50 text-sm sm:text-base">
            <Plus className="w-4 h-4 mr-2" />
            Add Endpoint
          </Button>
        </Link>
      </div>

      {/* Compact Metrics - Responsive Grid */}
      <div className="grid gap-2 sm:gap-3 grid-cols-2 lg:grid-cols-4">
        <CompactMetricCard
          label="Active Endpoints"
          value={mergedStats?.active_endpoints ?? 0}
          icon={Webhook}
          iconColor="text-indigo-500"
          subtext="listening for events"
          loading={statsLoading}
          onClick={() => router.push('/dashboard/endpoints')}
        />
        <CompactMetricCard
          label="Events (24h)"
          value={mergedStats?.events_today ?? 0}
          icon={Activity}
          iconColor="text-purple-500"
          subtext={`${mergedStats?.events_total ?? 0} total events`}
          loading={statsLoading}
          onClick={() => router.push('/dashboard/events')}
        />
        <CompactMetricCard
          label="Total Deliveries"
          value={mergedStats?.total_deliveries ?? 0}
          icon={Box}
          iconColor="text-blue-500"
          subtext={`${mergedStats?.successful_deliveries ?? 0} successful`}
          loading={statsLoading}
        />
        <CompactMetricCard
          label="Success Rate"
          value={`${mergedStats?.success_rate?.toFixed(1) ?? 0}%`}
          icon={CheckCircle}
          iconColor="text-emerald-500"
          subtext={avgDeliveryTime ? `${avgDeliveryTime.toFixed(0)}ms avg` : 'No data'}
          loading={statsLoading}
        />
      </div>

      {/* Second row of metrics */}
      <div className="grid gap-2 sm:gap-3 grid-cols-2 lg:grid-cols-4">
        <CompactMetricCard
          label="Avg Response Time"
          value={avgDeliveryTime ? `${avgDeliveryTime.toFixed(0)}ms` : '—'}
          icon={Zap}
          iconColor="text-amber-500"
          subtext="webhook delivery"
          loading={statsLoading}
        />
        <CompactMetricCard
          label="Failed Deliveries"
          value={(mergedStats?.total_deliveries ?? 0) - (mergedStats?.successful_deliveries ?? 0)}
          icon={Activity}
          iconColor="text-rose-500"
          subtext="requires attention"
          loading={statsLoading}
        />
        <CompactMetricCard
          label="Processing"
          value="Live"
          icon={Clock}
          iconColor="text-green-500"
          subtext="real-time updates"
          loading={false}
        />
        <CompactMetricCard
          label="System Health"
          value={successRate >= 95 ? '✓' : '⚠'}
          icon={BarChart3}
          iconColor={successRate >= 95 ? 'text-green-500' : 'text-amber-500'}
          subtext={successRate >= 95 ? 'All systems operational' : 'Degraded performance'}
          loading={statsLoading}
        />
      </div>

      {/* Alchemy API Usage Widget - Admin Only */}
      {isAdmin && <AlchemyCUWidget />}

      {/* Analytics Section - Compact Horizontal Bar - Responsive */}
      <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg">
        <CardHeader className="pb-2 pt-3 px-3 sm:px-6">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
            <CardTitle className="text-sm font-semibold text-slate-900 flex items-center gap-2">
              <BarChart3 className="w-4 h-4 text-indigo-600" />
              Analytics
            </CardTitle>
            <div className="flex gap-1">
              <Button
                size="sm"
                variant={timeRange === '24h' ? 'default' : 'outline'}
                onClick={() => setTimeRange('24h')}
                className={timeRange === '24h' ? 'bg-indigo-600 hover:bg-indigo-700 h-6 px-2 text-xs' : 'h-6 px-2 text-xs'}
              >
                24h
              </Button>
              <Button
                size="sm"
                variant={timeRange === '7d' ? 'default' : 'outline'}
                onClick={() => setTimeRange('7d')}
                className={timeRange === '7d' ? 'bg-indigo-600 hover:bg-indigo-700 h-6 px-2 text-xs' : 'h-6 px-2 text-xs'}
              >
                7d
              </Button>
              <Button
                size="sm"
                variant={timeRange === '30d' ? 'default' : 'outline'}
                onClick={() => setTimeRange('30d')}
                className={timeRange === '30d' ? 'bg-indigo-600 hover:bg-indigo-700 h-6 px-2 text-xs' : 'h-6 px-2 text-xs'}
              >
                30d
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent className="pt-2 pb-3 px-3 sm:px-6">
          {/* Horizontal Trend Analytics - Responsive Grid */}
          <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2">
            {/* Peak Events Hour/Day */}
            <div className="bg-gradient-to-br from-blue-50 to-indigo-50 rounded-lg p-2 border border-blue-100">
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-medium text-slate-700">Peak Activity</span>
                <TrendingUp className="w-3 h-3 text-blue-600" />
              </div>
              <div className="text-lg font-bold text-slate-900">
                {timeseriesLoading ? '...' : 
                  timeseriesData?.data_points?.length 
                    ? Math.max(...timeseriesData.data_points.map(p => p.event_count)).toLocaleString()
                    : '0'
                }
              </div>
              <div className="text-xs text-slate-600">max {granularity}</div>
            </div>

            {/* Delivery Rate */}
            <div className="bg-gradient-to-br from-purple-50 to-violet-50 rounded-lg p-2 border border-purple-100">
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-medium text-slate-700">Delivery Rate</span>
                <Box className="w-3 h-3 text-purple-600" />
              </div>
              <div className="text-lg font-bold text-slate-900">
                {timeseriesLoading ? '...' : 
                  timeseriesData?.data_points?.length 
                    ? `${((timeseriesData.data_points.reduce((sum, p) => sum + p.delivery_count, 0) / timeseriesData.data_points.reduce((sum, p) => sum + p.event_count, 0)) * 100 || 0).toFixed(0)}%`
                    : '0%'
                }
              </div>
              <div className="text-xs text-slate-600">events→webhooks</div>
            </div>

            {/* Best Success Rate */}
            <div className="bg-gradient-to-br from-emerald-50 to-green-50 rounded-lg p-2 border border-emerald-100">
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-medium text-slate-700">Best Period</span>
                <CheckCircle className="w-3 h-3 text-emerald-600" />
              </div>
              <div className="text-lg font-bold text-slate-900">
                {timeseriesLoading ? '...' : 
                  timeseriesData?.data_points?.length 
                    ? `${Math.max(...timeseriesData.data_points.map(p => p.success_rate)).toFixed(1)}%`
                    : '0%'
                }
              </div>
              <div className="text-xs text-slate-600">top success</div>
            </div>

            {/* Fastest Response */}
            <div className="bg-gradient-to-br from-amber-50 to-orange-50 rounded-lg p-2 border border-amber-100">
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-medium text-slate-700">Fastest</span>
                <Zap className="w-3 h-3 text-amber-600" />
              </div>
              <div className="text-lg font-bold text-slate-900">
                {timeseriesLoading ? '...' : 
                  timeseriesData?.data_points?.filter(p => p.avg_latency_ms !== null).length
                    ? `${Math.min(...timeseriesData.data_points.filter(p => p.avg_latency_ms !== null).map(p => p.avg_latency_ms as number)).toFixed(0)}ms`
                    : '—'
                }
              </div>
              <div className="text-xs text-slate-600">best time</div>
            </div>

            {/* Top Chain */}
            <div className="bg-gradient-to-br from-slate-50 to-gray-50 rounded-lg p-2 border border-slate-200">
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-medium text-slate-700">Top Chain</span>
                <Activity className="w-3 h-3 text-slate-600" />
              </div>
              <div className="text-sm font-bold text-slate-900 truncate">
                {chainLoading ? '...' : 
                  chainData?.distributions?.length
                    ? chainData.distributions.reduce((max, d) => d.event_count > max.event_count ? d : max).chain_name
                    : '—'
                }
              </div>
              <div className="text-xs text-slate-600">
                {chainLoading ? '' : 
                  chainData?.distributions?.length
                    ? `${chainData.distributions.reduce((max, d) => d.event_count > max.event_count ? d : max).percentage.toFixed(0)}%`
                    : 'no data'
                }
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Analytics Charts - Compact Side by Side - Responsive Grid */}
      <div className="grid gap-3 grid-cols-1 md:grid-cols-2 lg:grid-cols-2">
        {/* Timeseries Chart - Compact Version */}
        <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg">
          <CardHeader className="pb-2 pt-3">
            <CardTitle className="text-sm font-semibold text-slate-900">Events Over Time</CardTitle>
            <CardDescription className="text-xs text-slate-600">Event volume for {timeRange}</CardDescription>
          </CardHeader>
          <CardContent className="pt-2 pb-3">
            {timeseriesLoading ? (
              <div className="h-[120px] flex items-center justify-center text-slate-400 text-xs">
                Loading chart...
              </div>
            ) : timeseriesData && timeseriesData.data_points && timeseriesData.data_points.length > 0 ? (
              <ResponsiveContainer width="100%" height={120}>
                <AreaChart data={timeseriesData.data_points} margin={{ top: 5, right: 5, left: -20, bottom: 5 }}>
                  <defs>
                    <linearGradient id="colorEvents" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#6366f1" stopOpacity={0.8}/>
                      <stop offset="95%" stopColor="#6366f1" stopOpacity={0}/>
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
                  <XAxis 
                    dataKey="timestamp" 
                    stroke="#64748b"
                    fontSize={10}
                    tickFormatter={(value) => {
                      const date = new Date(value);
                      return timeRange === '24h' 
                        ? date.toLocaleTimeString('en-US', { hour: '2-digit' })
                        : date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
                    }}
                  />
                  <YAxis stroke="#64748b" fontSize={10} />
                  <Tooltip
                    contentStyle={{ backgroundColor: '#fff', border: '1px solid #e2e8f0', borderRadius: '8px', fontSize: '12px' }}
                    labelFormatter={(value) => new Date(value).toLocaleString()}
                    formatter={(value: number) => [value.toLocaleString(), 'Events']}
                  />
                  <Area 
                    type="monotone" 
                    dataKey="event_count" 
                    stroke="#6366f1" 
                    strokeWidth={2}
                    fillOpacity={1} 
                    fill="url(#colorEvents)" 
                  />
                </AreaChart>
              </ResponsiveContainer>
            ) : (
              <div className="h-[120px] flex flex-col items-center justify-center text-slate-400">
                <BarChart3 className="w-8 h-8 mb-1 opacity-50" />
                <p className="text-xs">No data for this time range</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Chain Distribution Pie Chart - Compact Version */}
        <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg">
          <CardHeader className="pb-2 pt-3">
            <CardTitle className="text-sm font-semibold text-slate-900">Events by Chain</CardTitle>
            <CardDescription className="text-xs text-slate-600">Distribution across networks</CardDescription>
          </CardHeader>
          <CardContent className="pt-2 pb-3">
            {chainLoading ? (
              <div className="h-[120px] flex items-center justify-center text-slate-400 text-xs">
                Loading chart...
              </div>
            ) : chainData && chainData.distributions && chainData.distributions.length > 0 ? (
              <ResponsiveContainer width="100%" height={120}>
                <PieChart>
                  <Pie
                    data={chainData.distributions}
                    cx="50%"
                    cy="50%"
                    labelLine={false}
                    label={(entry: unknown) => {
                      const data = entry as { chain_name: string; percentage: number };
                      return `${data.chain_name} ${data.percentage.toFixed(0)}%`;
                    }}
                    outerRadius={50}
                    fontSize={10}
                    fill="#8884d8"
                    dataKey="event_count"
                  >
                    {chainData.distributions?.map((entry, index) => {
                      const colors = ['#6366f1', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981'];
                      return <Cell key={`cell-${index}`} fill={colors[index % colors.length]} />;
                    })}
                  </Pie>
                  <Tooltip 
                    contentStyle={{ backgroundColor: '#fff', border: '1px solid #e2e8f0', borderRadius: '8px', fontSize: '11px' }}
                    formatter={(value: number, name: string, props?: unknown) => {
                      const payload = (props as { payload?: { chain_name: string; percentage: number } })?.payload;
                      return [
                        `${value.toLocaleString()} (${payload?.percentage?.toFixed(1) || 0}%)`,
                        payload?.chain_name || name
                      ];
                    }}
                  />
                  <Legend 
                    iconSize={8}
                    wrapperStyle={{ fontSize: '10px' }}
                  />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="h-[120px] flex flex-col items-center justify-center text-slate-400">
                <Activity className="w-8 h-8 mb-1 opacity-50" />
                <p className="text-xs">No chain data available</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Events & Insights - Responsive 2 Column Layout */}
      <div className="grid gap-3 sm:gap-4 grid-cols-1 lg:grid-cols-5">
        {/* Left Column (40%): Recent Events Table */}
        <div className="lg:col-span-2">
          <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg">
            <CardHeader>
              <CardTitle className="text-slate-900">Recent Events</CardTitle>
              <CardDescription className="text-slate-600">Latest 15 webhook events</CardDescription>
            </CardHeader>
            <CardContent>
              {eventsLoading ? (
                <div className="text-center py-8 text-muted-foreground">Loading events...</div>
              ) : !recentEvents?.events || recentEvents.events.length === 0 ? (
                <div className="text-center py-8">
                  <Activity className="w-12 h-12 mx-auto text-muted-foreground mb-3" />
                  <p className="text-muted-foreground">No events yet</p>
                  <p className="text-sm text-muted-foreground mt-1">
                    Events will appear here once your endpoints start receiving blockchain events
                  </p>
                </div>
              ) : (
                <>
                  <CompactEventTable
                    events={recentEvents.events}
                    maxHeight="400px"
                    onEventClick={(event) => router.push(`/dashboard/events?id=${event.id}`)}
                  />
                  <div className="mt-4 text-center">
                    <Link href="/dashboard/events">
                      <Button variant="outline" size="sm">View All {mergedStats?.events_total ?? 0} Events</Button>
                    </Link>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Right Column (60%): Insights & Analytics */}
        <div className="lg:col-span-3 space-y-3 sm:space-y-4">
          {/* Performance Insight */}
          <InsightCard
            type={successRate >= 95 ? 'success' : successRate >= 80 ? 'warning' : 'info'}
            title={successRate >= 95 ? 'Excellent Performance' : 'Performance Alert'}
            description={
              successRate >= 95
                ? `Your webhook delivery success rate is ${successRate.toFixed(1)}%. All systems are operating optimally with an average delivery time of ${avgDeliveryTime.toFixed(0)}ms.`
                : `Your success rate is ${successRate.toFixed(1)}%. Consider reviewing failed deliveries to identify potential issues with endpoint configurations or network connectivity.`
            }
            stats={[
              { label: 'Success Rate', value: `${successRate.toFixed(1)}%` },
              { label: 'Avg Response', value: `${avgDeliveryTime.toFixed(0)}ms` },
              { label: 'Failed Today', value: (stats?.total_deliveries ?? 0) - (stats?.successful_deliveries ?? 0) }
            ]}
            action={
              successRate < 95
                ? {
                    label: 'View Failed Deliveries',
                    onClick: () => router.push('/dashboard/events?status=failed')
                  }
                : undefined
            }
          />

          {/* Activity Recommendation */}
          <InsightCard
            type="recommendation"
            title="Event Activity Overview"
            description={
              (mergedStats?.events_today ?? 0) > 0
                ? `You've captured ${mergedStats?.events_today ?? 0} events today out of ${mergedStats?.events_total ?? 0} total events. ${
                    (mergedStats?.active_endpoints ?? 0) > 0
                      ? `Your ${mergedStats?.active_endpoints} active endpoints are processing events in real-time.`
                      : 'Configure more endpoints to capture additional blockchain events.'
                  }`
                : 'No events captured today. Make sure your endpoints are properly configured and active to start receiving blockchain events.'
            }
            stats={[
              { label: 'Today', value: mergedStats?.events_today ?? 0 },
              { label: 'Total', value: mergedStats?.events_total ?? 0 },
              { label: 'Active Endpoints', value: mergedStats?.active_endpoints ?? 0 }
            ]}
            action={{
              label: 'Configure Endpoints',
              onClick: () => router.push('/dashboard/endpoints')
            }}
          />

          {/* Quick Actions Insight */}
          <InsightCard
            type="info"
            title="Getting Started"
            description="Follow these steps to set up webhook monitoring: 1) Create an Application to organize your project, 2) Add Endpoints with webhook URLs to receive events, 3) Configure which blockchain networks and smart contracts to monitor, 4) Test your webhooks and monitor delivery status in real-time."
            stats={[
              { label: 'Step 1', value: 'Create App' },
              { label: 'Step 2', value: 'Add Endpoint' },
              { label: 'Step 3', value: 'Monitor Events' }
            ]}
            action={{
              label: 'Create Application',
              onClick: () => router.push('/dashboard/applications')
            }}
          />
        </div>
      </div>
    </div>
  );
}

// Alchemy CU Usage Widget Component
function AlchemyCUWidget() {
  const { data: cuStats, isLoading } = useQuery({
    queryKey: ['alchemy-cu-stats'],
    queryFn: () => api.get<{
      cu_consumed_today: number;
      cu_consumed_month: number;
      monthly_limit: number;
      usage_percent: number;
      estimated_monthly_burn: number;
      days_until_limit: number | null;
      breakdown_by_operation: Array<{
        operation: string;
        cu_consumed: number;
        api_calls: number;
        avg_cu_per_call: number;
      }>;
      breakdown_by_chain: Array<{
        chain_name: string;
        cu_consumed: number;
        percentage: number;
      }>;
      alert_level: string;
    }>('/statistics/alchemy-usage'),
    refetchInterval: 300000, // Refresh every 5 minutes
  });

  if (isLoading || !cuStats) {
    return null;
  }

  const alertColors = {
    ok: { bg: 'bg-emerald-50', border: 'border-emerald-200', text: 'text-emerald-700', icon: 'text-emerald-600' },
    warning: { bg: 'bg-amber-50', border: 'border-amber-200', text: 'text-amber-700', icon: 'text-amber-600' },
    critical: { bg: 'bg-rose-50', border: 'border-rose-200', text: 'text-rose-700', icon: 'text-rose-600' },
  };

  const colors = alertColors[cuStats.alert_level as keyof typeof alertColors] || alertColors.ok;

  // Format numbers
  const formatCU = (cu: number) => {
    if (cu >= 1_000_000) return `${(cu / 1_000_000).toFixed(1)}M`;
    if (cu >= 1_000) return `${(cu / 1_000).toFixed(1)}K`;
    return cu.toString();
  };

  return (
    <Card className={`${colors.bg} backdrop-blur-sm border-2 ${colors.border} shadow-lg`}>
      <CardHeader className="pb-2 pt-3 px-3 sm:px-6">
        <div className="flex items-center justify-between">
          <CardTitle className={`text-sm font-semibold ${colors.text} flex items-center gap-2`}>
            <Zap className={`w-4 h-4 ${colors.icon}`} />
            Alchemy API Usage
            {cuStats.alert_level === 'critical' && (
              <span className="ml-2 px-2 py-0.5 bg-rose-100 text-rose-700 text-xs font-medium rounded-full animate-pulse">
                CRITICAL
              </span>
            )}
            {cuStats.alert_level === 'warning' && (
              <span className="ml-2 px-2 py-0.5 bg-amber-100 text-amber-700 text-xs font-medium rounded-full">
                WARNING
              </span>
            )}
          </CardTitle>
        </div>
        <CardDescription className={`text-xs ${colors.text}`}>
          Monthly Compute Units (CUs) consumption tracking
        </CardDescription>
      </CardHeader>
      <CardContent className="pt-2 pb-3 px-3 sm:px-6 space-y-3">
        {/* Usage Progress Bar */}
        <div className="space-y-1">
          <div className="flex justify-between text-xs">
            <span className={colors.text}>
              {formatCU(cuStats.cu_consumed_month)} / {formatCU(cuStats.monthly_limit)} CUs
            </span>
            <span className={`font-semibold ${colors.text}`}>
              {cuStats.usage_percent.toFixed(1)}%
            </span>
          </div>
          <div className="w-full bg-white/50 rounded-full h-3 overflow-hidden border border-slate-200">
            <div
              className={`h-full transition-all duration-500 ${
                cuStats.alert_level === 'critical'
                  ? 'bg-gradient-to-r from-rose-500 to-rose-600'
                  : cuStats.alert_level === 'warning'
                  ? 'bg-gradient-to-r from-amber-500 to-amber-600'
                  : 'bg-gradient-to-r from-emerald-500 to-emerald-600'
              }`}
              style={{ width: `${Math.min(cuStats.usage_percent, 100)}%` }}
            />
          </div>
        </div>

        {/* Key Metrics Grid */}
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
          <div className="bg-white/70 rounded-lg p-2 border border-slate-200">
            <div className="text-xs text-slate-600 mb-0.5">Today</div>
            <div className={`text-base font-bold ${colors.text}`}>
              {formatCU(cuStats.cu_consumed_today)}
            </div>
          </div>
          <div className="bg-white/70 rounded-lg p-2 border border-slate-200">
            <div className="text-xs text-slate-600 mb-0.5">Est. Monthly</div>
            <div className={`text-base font-bold ${colors.text}`}>
              {formatCU(cuStats.estimated_monthly_burn)}
            </div>
          </div>
          <div className="bg-white/70 rounded-lg p-2 border border-slate-200">
            <div className="text-xs text-slate-600 mb-0.5">Days Remaining</div>
            <div className={`text-base font-bold ${colors.text}`}>
              {cuStats.days_until_limit ? Math.floor(cuStats.days_until_limit) : '∞'}
            </div>
          </div>
          <div className="bg-white/70 rounded-lg p-2 border border-slate-200">
            <div className="text-xs text-slate-600 mb-0.5">Plan Limit</div>
            <div className={`text-base font-bold ${colors.text}`}>
              {formatCU(cuStats.monthly_limit)}
            </div>
          </div>
        </div>

        {/* CU Breakdown by Operation */}
        <div className="space-y-1.5">
          <div className="text-xs font-semibold text-slate-700">CU Breakdown</div>
          <div className="space-y-1">
            {cuStats.breakdown_by_operation.map((op) => (
              <div key={op.operation} className="flex items-center justify-between text-xs bg-white/50 rounded px-2 py-1">
                <span className="text-slate-700 truncate flex-1">{op.operation.replace(/_/g, ' ')}</span>
                <span className={`font-semibold ${colors.text} ml-2`}>
                  {formatCU(op.cu_consumed)} ({op.api_calls.toLocaleString()} calls)
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Alert Message */}
        {cuStats.alert_level !== 'ok' && (
          <div className={`${colors.bg} border ${colors.border} rounded-lg p-2 text-xs ${colors.text}`}>
            <div className="font-semibold mb-1">
              {cuStats.alert_level === 'critical' ? '🚨 Urgent Action Required' : '⚠️ Action Recommended'}
            </div>
            <div>
              {cuStats.alert_level === 'critical'
                ? `You've used ${cuStats.usage_percent.toFixed(1)}% of your monthly limit. Service may be interrupted soon. Upgrade your Alchemy plan immediately.`
                : `You've used ${cuStats.usage_percent.toFixed(1)}% of your monthly limit. Consider upgrading your Alchemy plan or implementing log filtering optimizations.`}
            </div>
          </div>
        )}

        {/* Optimization Tip */}
        {cuStats.usage_percent < 70 && (
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-2 text-xs text-blue-700">
            <div className="font-semibold mb-1">💡 Cost Optimization Tip</div>
            <div>
              Enable endpoint-based log filtering to reduce CU consumption by 50-90%. Configure contract addresses and event topics in your endpoint settings.
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
