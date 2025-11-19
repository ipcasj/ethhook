/**
 * FilterBar Component
 * Advanced filtering controls for analytics views
 * Supports chain filtering, status filtering, and date range selection
 */

'use client';

import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Calendar } from '@/components/ui/calendar';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Calendar as CalendarIcon, X, Filter } from 'lucide-react';
import { format } from 'date-fns';
import { cn } from '@/lib/utils';

export interface FilterState {
  chainIds: number[];
  success?: boolean;
  startDate?: Date;
  endDate?: Date;
}

interface FilterBarProps {
  filters: FilterState;
  onFiltersChange: (filters: FilterState) => void;
  onApply?: () => void;
  className?: string;
}

const CHAIN_OPTIONS = [
  { id: 1, name: 'Ethereum' },
  { id: 10, name: 'Optimism' },
  { id: 137, name: 'Polygon' },
  { id: 8453, name: 'Base' },
  { id: 42161, name: 'Arbitrum' },
  { id: 11155111, name: 'Sepolia' },
];

const STATUS_OPTIONS = [
  { value: undefined, label: 'All Status' },
  { value: true, label: 'Success Only' },
  { value: false, label: 'Failed Only' },
];

export function FilterBar({
  filters,
  onFiltersChange,
  onApply,
  className,
}: FilterBarProps) {
  const [localFilters, setLocalFilters] = useState<FilterState>(filters);
  const [isOpen, setIsOpen] = useState(false);

  // Update local state when props change
  useEffect(() => {
    setLocalFilters(filters);
  }, [filters]);

  const handleChainToggle = (chainId: number) => {
    const newChainIds = localFilters.chainIds.includes(chainId)
      ? localFilters.chainIds.filter((id) => id !== chainId)
      : [...localFilters.chainIds, chainId];
    
    setLocalFilters({ ...localFilters, chainIds: newChainIds });
  };

  const handleStatusChange = (value: string) => {
    const success = value === 'true' ? true : value === 'false' ? false : undefined;
    setLocalFilters({ ...localFilters, success });
  };

  const handleDateSelect = (type: 'start' | 'end', date: Date | undefined) => {
    if (type === 'start') {
      setLocalFilters({ ...localFilters, startDate: date });
    } else {
      setLocalFilters({ ...localFilters, endDate: date });
    }
  };

  const handleApply = () => {
    onFiltersChange(localFilters);
    onApply?.();
    setIsOpen(false);
  };

  const handleReset = () => {
    const resetFilters: FilterState = {
      chainIds: [],
      success: undefined,
      startDate: undefined,
      endDate: undefined,
    };
    setLocalFilters(resetFilters);
    onFiltersChange(resetFilters);
    onApply?.();
  };

  const activeFilterCount = 
    localFilters.chainIds.length +
    (localFilters.success !== undefined ? 1 : 0) +
    (localFilters.startDate ? 1 : 0) +
    (localFilters.endDate ? 1 : 0);

  return (
    <div className={cn('flex flex-wrap items-center gap-2', className)}>
      {/* Chain Filter */}
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-9">
            <Filter className="mr-2 h-4 w-4" />
            Chains
            {localFilters.chainIds.length > 0 && (
              <Badge variant="secondary" className="ml-2 h-5 px-1.5">
                {localFilters.chainIds.length}
              </Badge>
            )}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-64 p-3" align="start">
          <div className="space-y-2">
            <h4 className="font-medium text-sm mb-2">Select Chains</h4>
            {CHAIN_OPTIONS.map((chain) => (
              <label
                key={chain.id}
                className="flex items-center space-x-2 cursor-pointer hover:bg-slate-50 p-2 rounded"
              >
                <input
                  type="checkbox"
                  checked={localFilters.chainIds.includes(chain.id)}
                  onChange={() => handleChainToggle(chain.id)}
                  className="rounded border-slate-300"
                />
                <span className="text-sm">{chain.name}</span>
              </label>
            ))}
          </div>
        </PopoverContent>
      </Popover>

      {/* Status Filter */}
      <Select
        value={
          localFilters.success === undefined
            ? 'all'
            : localFilters.success
            ? 'true'
            : 'false'
        }
        onValueChange={handleStatusChange}
      >
        <SelectTrigger className="h-9 w-[140px]">
          <SelectValue placeholder="Status" />
        </SelectTrigger>
        <SelectContent>
          {STATUS_OPTIONS.map((option) => (
            <SelectItem
              key={option.value?.toString() || 'all'}
              value={option.value?.toString() || 'all'}
            >
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {/* Date Range Filter */}
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-9">
            <CalendarIcon className="mr-2 h-4 w-4" />
            {localFilters.startDate && localFilters.endDate
              ? `${format(localFilters.startDate, 'MMM d')} - ${format(localFilters.endDate, 'MMM d')}`
              : 'Date Range'}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <div className="p-3 space-y-3">
            <div>
              <label className="text-xs font-medium text-slate-600 mb-1.5 block">
                Start Date
              </label>
              <Calendar
                mode="single"
                selected={localFilters.startDate}
                onSelect={(date) => handleDateSelect('start', date)}
                className="rounded-md border"
              />
            </div>
            <div>
              <label className="text-xs font-medium text-slate-600 mb-1.5 block">
                End Date
              </label>
              <Calendar
                mode="single"
                selected={localFilters.endDate}
                onSelect={(date) => handleDateSelect('end', date)}
                disabled={(date) =>
                  localFilters.startDate ? date < localFilters.startDate : false
                }
                className="rounded-md border"
              />
            </div>
          </div>
        </PopoverContent>
      </Popover>

      {/* Apply & Reset Buttons */}
      <div className="flex gap-2 ml-auto">
        {activeFilterCount > 0 && (
          <>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleReset}
              className="h-9"
            >
              <X className="mr-1 h-4 w-4" />
              Reset
            </Button>
            <Button
              size="sm"
              onClick={handleApply}
              className="h-9"
            >
              Apply Filters
            </Button>
          </>
        )}
      </div>

      {/* Active Filters Display */}
      {activeFilterCount > 0 && (
        <div className="w-full flex flex-wrap gap-2 mt-2 pt-2 border-t">
          {localFilters.chainIds.map((chainId) => {
            const chain = CHAIN_OPTIONS.find((c) => c.id === chainId);
            return (
              <Badge key={chainId} variant="secondary" className="gap-1">
                {chain?.name || `Chain ${chainId}`}
                <X
                  className="h-3 w-3 cursor-pointer"
                  onClick={() => handleChainToggle(chainId)}
                />
              </Badge>
            );
          })}
          {localFilters.success !== undefined && (
            <Badge variant="secondary" className="gap-1">
              {localFilters.success ? 'Success Only' : 'Failed Only'}
              <X
                className="h-3 w-3 cursor-pointer"
                onClick={() =>
                  setLocalFilters({ ...localFilters, success: undefined })
                }
              />
            </Badge>
          )}
          {localFilters.startDate && localFilters.endDate && (
            <Badge variant="secondary" className="gap-1">
              {format(localFilters.startDate, 'MMM d')} -{' '}
              {format(localFilters.endDate, 'MMM d')}
              <X
                className="h-3 w-3 cursor-pointer"
                onClick={() =>
                  setLocalFilters({
                    ...localFilters,
                    startDate: undefined,
                    endDate: undefined,
                  })
                }
              />
            </Badge>
          )}
        </div>
      )}
    </div>
  );
}
