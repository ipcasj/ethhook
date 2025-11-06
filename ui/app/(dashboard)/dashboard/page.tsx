'use client';

import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { api } from '@/lib/api-client';
import { DashboardStats, Event } from '@/lib/types';
import { Activity, Box, Webhook, CheckCircle, Plus } from 'lucide-react';
import Link from 'next/link';
import { formatDateTime } from '@/lib/utils';

export default function DashboardPage() {
  const { data: stats, isLoading: statsLoading } = useQuery<DashboardStats>({
    queryKey: ['dashboard-stats'],
    queryFn: () => api.get<DashboardStats>('/statistics/dashboard'),
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const { data: recentEvents, isLoading: eventsLoading } = useQuery<{ events: Event[] }>({
    queryKey: ['recent-events'],
    queryFn: () => api.get<{ events: Event[] }>('/events?limit=10'),
    refetchInterval: 5000, // Refresh every 5 seconds
  });

  return (
    <div className="space-y-6">
      {/* Page header */}
      <div>
        <h1 className="text-3xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
          Dashboard
        </h1>
        <p className="text-slate-600 mt-1">
          Overview of your webhook infrastructure
        </p>
      </div>

      {/* Quick actions */}
      <div className="flex gap-3">
        <Link href="/dashboard/applications">
          <Button className="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-indigo-500/30">
            <Plus className="w-4 h-4 mr-2" />
            Create Application
          </Button>
        </Link>
        <Link href="/dashboard/endpoints">
          <Button variant="outline" className="border-indigo-200 text-indigo-700 hover:bg-indigo-50">
            <Plus className="w-4 h-4 mr-2" />
            Add Endpoint
          </Button>
        </Link>
      </div>

      {/* Metrics cards */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg hover:shadow-xl transition-shadow">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-slate-600">
              Active Endpoints
            </CardTitle>
            <Webhook className="w-4 h-4 text-indigo-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-slate-900">
              {statsLoading ? '...' : stats?.active_endpoints ?? 0}
            </div>
            <p className="text-xs text-slate-500 mt-1">listening for events</p>
          </CardContent>
        </Card>

        <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg hover:shadow-xl transition-shadow">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-slate-600">
              Events (24h)
            </CardTitle>
            <Activity className="w-4 h-4 text-purple-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-slate-900">
              {statsLoading ? '...' : stats?.events_today ?? 0}
            </div>
            <p className="text-xs text-slate-500 mt-1">
              {statsLoading ? '' : `${stats?.events_total ?? 0} total`}
            </p>
          </CardContent>
        </Card>

        <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg hover:shadow-xl transition-shadow">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-slate-600">
              Deliveries
            </CardTitle>
            <Box className="w-4 h-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-slate-900">
              {statsLoading ? '...' : stats?.total_deliveries ?? 0}
            </div>
            <p className="text-xs text-slate-500 mt-1">
              {statsLoading ? '' : `${stats?.successful_deliveries ?? 0} successful`}
            </p>
          </CardContent>
        </Card>

        <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg hover:shadow-xl transition-shadow">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-slate-600">
              Success Rate
            </CardTitle>
            <CheckCircle className="w-4 h-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-slate-900">
              {statsLoading ? '...' : `${stats?.success_rate?.toFixed(1) ?? 0}%`}
            </div>
            <p className="text-xs text-slate-500 mt-1">
              {statsLoading || !stats?.avg_delivery_time_ms ? '' : `${stats.avg_delivery_time_ms.toFixed(0)}ms avg`}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Recent events */}
      <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg">
        <CardHeader>
          <CardTitle className="text-slate-900">Recent Events</CardTitle>
          <CardDescription className="text-slate-600">Latest webhook events from your endpoints</CardDescription>
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
            <div className="space-y-2">
              {recentEvents.events.map((event) => (
                <div
                  key={event.id}
                  className="flex items-center gap-4 p-3 border rounded-lg hover:bg-slate-50 transition-colors"
                >
                  {/* Status Badge */}
                  <Badge
                    variant={
                      event.status === 'delivered'
                        ? 'default'
                        : event.status === 'failed'
                        ? 'destructive'
                        : 'secondary'
                    }
                    className="shrink-0"
                  >
                    {event.status}
                  </Badge>

                  {/* Event Info */}
                  <div className="flex-1 min-w-0 grid grid-cols-1 md:grid-cols-3 gap-2">
                    {/* Column 1: Event Type & Contract */}
                    <div className="space-y-0.5">
                      <p className="font-mono text-xs font-medium text-slate-900 truncate">
                        {event.event_type || 'Unknown Event'}
                      </p>
                      <p className="font-mono text-xs text-slate-500 truncate">
                        {event.contract_address.slice(0, 6)}...{event.contract_address.slice(-4)}
                      </p>
                    </div>

                    {/* Column 2: Chain & Block */}
                    <div className="space-y-0.5">
                      <p className="text-xs text-slate-600">
                        Chain {event.chain_id} • Block {event.block_number.toLocaleString()}
                      </p>
                      <p className="font-mono text-xs text-slate-500 truncate">
                        Tx: {event.transaction_hash.slice(0, 8)}...{event.transaction_hash.slice(-6)}
                      </p>
                    </div>

                    {/* Column 3: Delivery Info */}
                    <div className="space-y-0.5 text-right md:text-left">
                      <p className="text-xs text-slate-600">
                        {event.attempts} {event.attempts === 1 ? 'attempt' : 'attempts'}
                      </p>
                      <p className="text-xs text-slate-500">
                        {formatDateTime(event.created_at)}
                      </p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
          {recentEvents?.events && recentEvents.events.length > 0 && (
            <div className="mt-4 text-center">
              <Link href="/dashboard/events">
                <Button variant="outline">View All Events</Button>
              </Link>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
