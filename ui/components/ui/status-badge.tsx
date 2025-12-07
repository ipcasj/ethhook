/**
 * StatusBadge Component
 * User-friendly status indicators with improved color scheme
 * Based on industry standards from Stripe, GitHub, and DataDog
 */

'use client';

import { CheckCircle2, XCircle, Clock, AlertCircle } from 'lucide-react';
import { cn } from '@/lib/utils';

interface StatusBadgeProps {
  status: 'active' | 'inactive' | 'delivered' | 'failed' | 'pending' | 'processing' | 'error' | 'warning' | 'paused';
  showIcon?: boolean;
  showDot?: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export function StatusBadge({ status, showIcon = false, showDot = true, size = 'md', className }: StatusBadgeProps) {
  const config = {
    active: {
      label: 'Active',
      icon: CheckCircle2,
      className: 'bg-green-100 text-green-800 border-green-200',
      dotClassName: 'bg-green-600',
      pulse: true,
    },
    inactive: {
      label: 'Inactive',
      icon: XCircle,
      className: 'bg-slate-100 text-slate-800 border-slate-200',
      dotClassName: 'bg-slate-600',
      pulse: false,
    },
    delivered: {
      label: 'Delivered',
      icon: CheckCircle2,
      className: 'bg-green-100 text-green-800 border-green-200',
      dotClassName: 'bg-green-600',
      pulse: false,
    },
    failed: {
      label: 'Failed',
      icon: XCircle,
      className: 'bg-red-100 text-red-800 border-red-200',
      dotClassName: 'bg-red-600',
      pulse: false,
    },
    error: {
      label: 'Error',
      icon: XCircle,
      className: 'bg-red-100 text-red-800 border-red-200',
      dotClassName: 'bg-red-600',
      pulse: false,
    },
    pending: {
      label: 'Pending',
      icon: Clock,
      className: 'bg-yellow-100 text-yellow-800 border-yellow-200',
      dotClassName: 'bg-yellow-600',
      pulse: false,
    },
    processing: {
      label: 'Processing',
      icon: AlertCircle,
      className: 'bg-blue-100 text-blue-800 border-blue-200',
      dotClassName: 'bg-blue-600',
      pulse: true,
    },
    warning: {
      label: 'Warning',
      icon: AlertCircle,
      className: 'bg-yellow-100 text-yellow-800 border-yellow-200',
      dotClassName: 'bg-yellow-600',
      pulse: false,
    },
    paused: {
      label: 'Paused',
      icon: Clock,
      className: 'bg-slate-100 text-slate-800 border-slate-200',
      dotClassName: 'bg-slate-600',
      pulse: false,
    },
  };

  const { label, icon: Icon, className: statusClassName, dotClassName, pulse } = config[status];

  const sizeClasses = {
    sm: 'text-xs px-2 py-0.5',
    md: 'text-xs px-2.5 py-1',
    lg: 'text-sm px-3 py-1.5',
  };

  const iconSizes = {
    sm: 'w-3 h-3',
    md: 'w-3.5 h-3.5',
    lg: 'w-4 h-4',
  };

  const dotSizes = {
    sm: 'w-1.5 h-1.5',
    md: 'w-2 h-2',
    lg: 'w-2.5 h-2.5',
  };

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 font-medium rounded-full border',
        sizeClasses[size],
        statusClassName,
        className
      )}
    >
      {showDot && (
        <span className="relative flex">
          {pulse && (
            <span
              className={cn(
                'animate-ping absolute inline-flex rounded-full opacity-75',
                dotSizes[size],
                dotClassName
              )}
            />
          )}
          <span
            className={cn(
              'relative inline-flex rounded-full',
              dotSizes[size],
              dotClassName
            )}
          />
        </span>
      )}
      {showIcon && <Icon className={iconSizes[size]} />}
      {label}
    </span>
  );
}
