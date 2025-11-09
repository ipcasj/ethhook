/**
 * InfoBanner Component
 * A collapsible information banner for displaying page instructions and context
 * Inspired by Stripe's documentation panels and Datadog's help sections
 */

'use client';

import { useState } from 'react';
import { Info, ChevronDown, ChevronUp, X } from 'lucide-react';
import { cn } from '@/lib/utils';

interface InfoBannerProps {
  title: string;
  description: string;
  tips?: string[];
  defaultCollapsed?: boolean;
  dismissible?: boolean;
  className?: string;
}

export function InfoBanner({
  title,
  description,
  tips,
  defaultCollapsed = false,
  dismissible = true,
  className,
}: InfoBannerProps) {
  const [isCollapsed, setIsCollapsed] = useState(defaultCollapsed);
  const [isDismissed, setIsDismissed] = useState(false);

  if (isDismissed) return null;

  return (
    <div
      className={cn(
        'bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-200 rounded-lg overflow-hidden',
        className
      )}
    >
      {/* Header */}
      <div className="flex items-center justify-between p-3 cursor-pointer" onClick={() => setIsCollapsed(!isCollapsed)}>
        <div className="flex items-center gap-2">
          <div className="flex items-center justify-center w-8 h-8 rounded-full bg-blue-100">
            <Info className="w-4 h-4 text-blue-600" />
          </div>
          <div>
            <h3 className="text-sm font-semibold text-slate-900">{title}</h3>
            {isCollapsed && (
              <p className="text-xs text-slate-600 mt-0.5 line-clamp-1">{description}</p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setIsCollapsed(!isCollapsed);
            }}
            className="p-1 hover:bg-blue-100 rounded transition-colors"
          >
            {isCollapsed ? (
              <ChevronDown className="w-4 h-4 text-slate-600" />
            ) : (
              <ChevronUp className="w-4 h-4 text-slate-600" />
            )}
          </button>
          {dismissible && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                setIsDismissed(true);
              }}
              className="p-1 hover:bg-blue-100 rounded transition-colors"
            >
              <X className="w-4 h-4 text-slate-600" />
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      {!isCollapsed && (
        <div className="px-4 pb-4 pt-1 space-y-3">
          <p className="text-sm text-slate-700 leading-relaxed">{description}</p>
          
          {tips && tips.length > 0 && (
            <div className="space-y-2">
              <p className="text-xs font-semibold text-slate-700 uppercase tracking-wide">Quick Tips</p>
              <ul className="space-y-1.5">
                {tips.map((tip, index) => (
                  <li key={index} className="flex items-start gap-2 text-sm text-slate-600">
                    <span className="text-blue-500 mt-1">•</span>
                    <span className="flex-1">{tip}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
