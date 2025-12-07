/**
 * MetricCard Component
 * Professional metric display with trends and icons
 * Based on design patterns from Stripe, Datadog, and Vercel Analytics
 */

'use client';

import { LucideIcon, TrendingUp, TrendingDown, Minus } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { cn } from '@/lib/utils';

interface MetricCardProps {
  title: string;
  value: string | number;
  icon: LucideIcon;
  trend?: {
    value: number;
    direction: 'up' | 'down' | 'neutral';
    label?: string;
  };
  description?: string;
  loading?: boolean;
  className?: string;
}

export function MetricCard({ 
  title, 
  value, 
  icon: Icon, 
  trend, 
  description,
  loading,
  className 
}: MetricCardProps) {
  const TrendIcon = trend?.direction === 'up' 
    ? TrendingUp 
    : trend?.direction === 'down' 
    ? TrendingDown 
    : Minus;

  const trendColor = trend?.direction === 'up'
    ? 'text-green-600'
    : trend?.direction === 'down'
    ? 'text-red-600'
    : 'text-slate-600';

  if (loading) {
    return (
      <Card className={cn('p-6', className)}>
        <div className="animate-pulse space-y-3">
          <Skeleton className="h-4 w-1/2" />
          <Skeleton className="h-8 w-3/4" />
          <Skeleton className="h-3 w-1/3" />
        </div>
      </Card>
    );
  }

  return (
    <Card className={cn('p-6 hover:shadow-md transition-shadow', className)}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <p className="text-sm font-medium text-slate-600 mb-1">
            {title}
          </p>
          <p className="text-3xl font-bold text-slate-900 mb-2">
            {value}
          </p>
          {trend && (
            <div className="flex items-center gap-1 text-sm">
              <TrendIcon className={cn('h-4 w-4', trendColor)} />
              <span className={cn('font-medium', trendColor)}>
                {trend.value > 0 ? '+' : ''}{trend.value}%
              </span>
              {trend.label && (
                <span className="text-slate-500">{trend.label}</span>
              )}
            </div>
          )}
          {description && !trend && (
            <p className="text-sm text-slate-500">
              {description}
            </p>
          )}
        </div>
        <div className="rounded-full bg-blue-100 p-3">
          <Icon className="h-6 w-6 text-blue-600" />
        </div>
      </div>
    </Card>
  );
}
