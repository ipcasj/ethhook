/**
 * CompactMetricCard Component
 * Compact metric display inspired by Bloomberg Terminal and modern financial dashboards
 * Features: Dense information display, trend indicators, sparklines support
 */

'use client';

import { LucideIcon, TrendingUp, TrendingDown } from 'lucide-react';
import { cn } from '@/lib/utils';

interface CompactMetricCardProps {
  label: string;
  value: string | number;
  icon?: LucideIcon;
  iconColor?: string;
  subtext?: string;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  onClick?: () => void;
  loading?: boolean;
  className?: string;
}

export function CompactMetricCard({
  label,
  value,
  icon: Icon,
  iconColor = 'text-blue-500',
  subtext,
  trend,
  onClick,
  loading = false,
  className,
}: CompactMetricCardProps) {
  return (
    <div
      onClick={onClick}
      className={cn(
        'group relative bg-white rounded-lg border border-slate-200 p-3 transition-all',
        onClick && 'cursor-pointer hover:border-blue-300 hover:shadow-md',
        className
      )}
    >
      <div className="flex items-start justify-between gap-2">
        {/* Left: Label and Value */}
        <div className="flex-1 min-w-0">
          <p className="text-xs font-medium text-slate-500 uppercase tracking-wide mb-1">
            {label}
          </p>
          <div className="flex items-baseline gap-2">
            <p className={cn(
              'text-2xl font-bold text-slate-900 tabular-nums',
              loading && 'animate-pulse'
            )}>
              {loading ? '—' : value}
            </p>
            {trend && (
              <div className={cn(
                'flex items-center gap-0.5 text-xs font-medium',
                trend.isPositive ? 'text-green-600' : 'text-red-600'
              )}>
                {trend.isPositive ? (
                  <TrendingUp className="w-3 h-3" />
                ) : (
                  <TrendingDown className="w-3 h-3" />
                )}
                <span>{Math.abs(trend.value)}%</span>
              </div>
            )}
          </div>
          {subtext && (
            <p className="text-xs text-slate-500 mt-1 truncate">{subtext}</p>
          )}
        </div>

        {/* Right: Icon */}
        {Icon && (
          <div className={cn(
            'flex items-center justify-center w-10 h-10 rounded-lg bg-slate-50 transition-colors',
            onClick && 'group-hover:bg-blue-50'
          )}>
            <Icon className={cn('w-5 h-5', iconColor)} />
          </div>
        )}
      </div>
    </div>
  );
}
