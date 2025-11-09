/**
 * StatusBadge Component
 * User-friendly status indicators with improved color scheme
 * Based on industry standards from Stripe, GitHub, and DataDog
 */

'use client';

import { CheckCircle2, XCircle, Clock, AlertCircle } from 'lucide-react';
import { cn } from '@/lib/utils';

interface StatusBadgeProps {
  status: 'active' | 'inactive' | 'delivered' | 'failed' | 'pending' | 'processing';
  showIcon?: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export function StatusBadge({ status, showIcon = true, size = 'md', className }: StatusBadgeProps) {
  const config = {
    active: {
      label: 'Active',
      icon: CheckCircle2,
      className: 'bg-emerald-100 text-emerald-700 border-emerald-200',
    },
    inactive: {
      label: 'Inactive',
      icon: XCircle,
      className: 'bg-slate-100 text-slate-600 border-slate-200',
    },
    delivered: {
      label: 'Delivered',
      icon: CheckCircle2,
      className: 'bg-emerald-100 text-emerald-700 border-emerald-200',
    },
    failed: {
      label: 'Failed',
      icon: XCircle,
      className: 'bg-rose-100 text-rose-700 border-rose-200',
    },
    pending: {
      label: 'Pending',
      icon: Clock,
      className: 'bg-amber-100 text-amber-700 border-amber-200',
    },
    processing: {
      label: 'Processing',
      icon: AlertCircle,
      className: 'bg-blue-100 text-blue-700 border-blue-200',
    },
  };

  const { label, icon: Icon, className: statusClassName } = config[status];

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

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 font-medium rounded-full border',
        sizeClasses[size],
        statusClassName,
        className
      )}
    >
      {showIcon && <Icon className={iconSizes[size]} />}
      {label}
    </span>
  );
}
