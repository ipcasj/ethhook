/**
 * InsightCard Component
 * Displays analytical insights and recommendations
 * Inspired by Stripe Dashboard's insights and Datadog's recommendations
 */

'use client';

import { LucideIcon, TrendingUp, AlertCircle, CheckCircle, Info } from 'lucide-react';
import { cn } from '@/lib/utils';

interface InsightCardProps {
  title: string;
  description: string;
  type?: 'info' | 'success' | 'warning' | 'recommendation';
  action?: {
    label: string;
    onClick: () => void;
  };
  stats?: Array<{
    label: string;
    value: string | number;
  }>;
  className?: string;
}

export function InsightCard({
  title,
  description,
  type = 'info',
  action,
  stats,
  className,
}: InsightCardProps) {
  const config = {
    info: {
      icon: Info,
      iconBg: 'bg-blue-100',
      iconColor: 'text-blue-600',
      borderColor: 'border-blue-200',
      bgColor: 'bg-blue-50/50',
    },
    success: {
      icon: CheckCircle,
      iconBg: 'bg-green-100',
      iconColor: 'text-green-600',
      borderColor: 'border-green-200',
      bgColor: 'bg-green-50/50',
    },
    warning: {
      icon: AlertCircle,
      iconBg: 'bg-amber-100',
      iconColor: 'text-amber-600',
      borderColor: 'border-amber-200',
      bgColor: 'bg-amber-50/50',
    },
    recommendation: {
      icon: TrendingUp,
      iconBg: 'bg-purple-100',
      iconColor: 'text-purple-600',
      borderColor: 'border-purple-200',
      bgColor: 'bg-purple-50/50',
    },
  };

  const { icon: Icon, iconBg, iconColor, borderColor, bgColor } = config[type];

  return (
    <div
      className={cn(
        'border rounded-lg p-4 transition-all',
        borderColor,
        bgColor,
        className
      )}
    >
      <div className="flex items-start gap-3">
        {/* Icon */}
        <div className={cn('flex items-center justify-center w-8 h-8 rounded-lg flex-shrink-0', iconBg)}>
          <Icon className={cn('w-4 h-4', iconColor)} />
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <h3 className="text-sm font-semibold text-slate-900 mb-1">{title}</h3>
          <p className="text-sm text-slate-600 leading-relaxed mb-3">{description}</p>

          {/* Stats */}
          {stats && stats.length > 0 && (
            <div className="flex flex-wrap gap-4 mb-3">
              {stats.map((stat, index) => (
                <div key={index} className="flex flex-col">
                  <span className="text-xs text-slate-500">{stat.label}</span>
                  <span className="text-sm font-semibold text-slate-900 tabular-nums">
                    {stat.value}
                  </span>
                </div>
              ))}
            </div>
          )}

          {/* Action */}
          {action && (
            <button
              onClick={action.onClick}
              className="text-sm font-medium text-blue-600 hover:text-blue-700 transition-colors"
            >
              {action.label} →
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
