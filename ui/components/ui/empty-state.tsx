/**
 * EmptyState Component
 * Professional empty states with icons, descriptions, and actions
 * Based on best practices from Stripe, Linear, and Vercel
 */

'use client';

import { LucideIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface EmptyStateProps {
  icon: LucideIcon;
  title: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
    icon?: LucideIcon;
    variant?: 'default' | 'outline' | 'ghost';
  };
  className?: string;
}

export function EmptyState({ 
  icon: Icon, 
  title, 
  description, 
  action,
  className 
}: EmptyStateProps) {
  const ActionIcon = action?.icon;

  return (
    <div className={cn(
      'flex flex-col items-center justify-center py-12 px-4 text-center',
      className
    )}>
      <div className="rounded-full bg-slate-100 p-3 mb-4">
        <Icon className="h-6 w-6 text-slate-600" />
      </div>
      <h3 className="text-lg font-semibold text-slate-900 mb-1">
        {title}
      </h3>
      {description && (
        <p className="text-sm text-slate-500 max-w-md mb-4">
          {description}
        </p>
      )}
      {action && (
        <Button 
          onClick={action.onClick} 
          variant={action.variant || 'outline'}
        >
          {ActionIcon && <ActionIcon className="h-4 w-4 mr-2" />}
          {action.label}
        </Button>
      )}
    </div>
  );
}
