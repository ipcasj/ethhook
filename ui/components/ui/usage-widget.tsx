'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { TrendingUp, AlertTriangle, CheckCircle } from 'lucide-react';

export interface UsageWidgetProps {
  title: string;
  description?: string;
  current: number;
  limit: number;
  periodLabel?: string;
  projectedTotal?: number;
  loading?: boolean;
}

/**
 * Widget for displaying usage metrics with progress bar
 * Shows current usage vs limit with projection for the period
 */
export function UsageWidget({
  title,
  description,
  current,
  limit,
  periodLabel = 'this month',
  projectedTotal,
  loading = false,
}: UsageWidgetProps) {
  const percentage = limit > 0 ? (current / limit) * 100 : 0;
  const isWarning = percentage >= 80 && percentage < 95;
  const isDanger = percentage >= 95;

  // Calculate status
  const getStatus = () => {
    if (isDanger) return { color: 'text-rose-600', icon: AlertTriangle, label: 'Near Limit' };
    if (isWarning) return { color: 'text-amber-600', icon: TrendingUp, label: 'High Usage' };
    return { color: 'text-emerald-600', icon: CheckCircle, label: 'Healthy' };
  };

  const status = getStatus();
  const StatusIcon = status.icon;

  // Get progress bar color
  const getProgressColor = () => {
    if (isDanger) return 'bg-rose-500';
    if (isWarning) return 'bg-amber-500';
    return 'bg-indigo-600';
  };

  return (
    <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-slate-900">{title}</CardTitle>
            {description && <CardDescription>{description}</CardDescription>}
          </div>
          <div className="flex items-center gap-2">
            <StatusIcon className={`w-5 h-5 ${status.color}`} />
            <span className={`text-sm font-medium ${status.color}`}>{status.label}</span>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="animate-pulse space-y-3">
            <div className="h-4 bg-slate-200 rounded w-1/2"></div>
            <div className="h-2 bg-slate-200 rounded"></div>
            <div className="h-3 bg-slate-200 rounded w-3/4"></div>
          </div>
        ) : (
          <div className="space-y-4">
            {/* Current Usage */}
            <div>
              <div className="flex justify-between items-baseline mb-2">
                <span className="text-sm text-slate-600">Usage {periodLabel}</span>
                <span className="text-lg font-bold text-slate-900">
                  {current.toLocaleString()} <span className="text-sm text-slate-500 font-normal">/ {limit.toLocaleString()}</span>
                </span>
              </div>
              <div className="relative h-3 w-full overflow-hidden rounded-full bg-slate-200">
                <div 
                  className={`h-full transition-all duration-300 rounded-full ${getProgressColor()}`}
                  style={{ width: `${Math.min(percentage, 100)}%` }}
                />
              </div>
              <div className="flex justify-between items-center mt-2">
                <span className="text-xs text-slate-500">
                  {percentage.toFixed(1)}% used
                </span>
                <span className="text-xs text-slate-500">
                  {(limit - current).toLocaleString()} remaining
                </span>
              </div>
            </div>

            {/* Projection */}
            {projectedTotal !== undefined && projectedTotal > current && (
              <div className="pt-3 border-t border-slate-200">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <TrendingUp className="w-4 h-4 text-slate-500" />
                    <span className="text-sm text-slate-600">Projected by period end</span>
                  </div>
                  <span className="text-sm font-semibold text-slate-900">
                    {projectedTotal.toLocaleString()}
                  </span>
                </div>
                {projectedTotal > limit && (
                  <p className="text-xs text-rose-600 mt-2 flex items-center gap-1">
                    <AlertTriangle className="w-3 h-3" />
                    Projected to exceed limit by {(projectedTotal - limit).toLocaleString()}
                  </p>
                )}
              </div>
            )}

            {/* Additional Stats */}
            <div className="grid grid-cols-2 gap-4 pt-3 border-t border-slate-200">
              <div>
                <p className="text-xs text-slate-500">Daily Average</p>
                <p className="text-sm font-semibold text-slate-900">
                  {Math.round(current / Math.max(1, new Date().getDate())).toLocaleString()}
                </p>
              </div>
              <div>
                <p className="text-xs text-slate-500">Days Remaining</p>
                <p className="text-sm font-semibold text-slate-900">
                  {new Date(new Date().getFullYear(), new Date().getMonth() + 1, 0).getDate() - new Date().getDate()}
                </p>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
