'use client';

import { LineChart, Line, Area, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface TooltipPayload {
  color: string;
  name: string;
  value: number;
}

const CustomTimeSeriesTooltip = ({ 
  active, 
  payload, 
  label, 
  formatTooltip 
}: { 
  active?: boolean; 
  payload?: TooltipPayload[]; 
  label?: string;
  formatTooltip?: (value: number) => string;
}) => {
  if (active && payload && payload.length) {
    return (
      <div className="bg-white/95 backdrop-blur-sm p-3 rounded-lg shadow-lg border border-slate-200">
        <p className="text-xs text-slate-500 mb-2">
          {label ? new Date(label).toLocaleString('en-US', {
            month: 'short',
            day: 'numeric',
            hour: 'numeric',
            minute: '2-digit',
          }) : ''}
        </p>
        {payload.map((entry, index) => (
          <div key={index} className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full" style={{ backgroundColor: entry.color }} />
            <span className="text-sm text-slate-700 font-medium">{entry.name}:</span>
            <span className="text-sm text-slate-900 font-semibold">
              {formatTooltip ? formatTooltip(entry.value) : entry.value}
            </span>
          </div>
        ))}
      </div>
    );
  }
  return null;
};

export interface TimeSeriesDataPoint {
  timestamp: string;
  [key: string]: string | number;
}

export interface TimeSeriesChartProps {
  title: string;
  description?: string;
  data: TimeSeriesDataPoint[];
  lines: {
    dataKey: string;
    name: string;
    color: string;
    type?: 'line' | 'area';
  }[];
  yAxisLabel?: string;
  xAxisLabel?: string;
  height?: number;
  formatYAxis?: (value: number) => string;
  formatTooltip?: (value: number) => string;
  loading?: boolean;
}

/**
 * Reusable timeseries chart component for displaying line and area charts
 * Supports multiple datasets, custom formatting, and responsive sizing
 */
export function TimeSeriesChart({
  title,
  description,
  data,
  lines,
  yAxisLabel,
  height = 300,
  formatYAxis,
  formatTooltip,
  loading = false,
}: TimeSeriesChartProps) {
  // Format timestamp for display
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

    if (diffDays < 1) {
      // Show time for today
      return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
    } else if (diffDays < 7) {
      // Show day and time for last week
      return date.toLocaleString('en-US', { weekday: 'short', hour: 'numeric' });
    } else {
      // Show date for older data
      return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-slate-900">{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex items-center justify-center" style={{ height }}>
            <div className="animate-pulse text-slate-500">Loading chart data...</div>
          </div>
        ) : data.length === 0 ? (
          <div className="flex items-center justify-center text-slate-500" style={{ height }}>
            No data available for the selected time range
          </div>
        ) : (
          <ResponsiveContainer width="100%" height={height}>
            <LineChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
              <XAxis
                dataKey="timestamp"
                tickFormatter={formatTimestamp}
                tick={{ fontSize: 12, fill: '#64748b' }}
                tickLine={{ stroke: '#cbd5e1' }}
                axisLine={{ stroke: '#cbd5e1' }}
              />
              <YAxis
                label={yAxisLabel ? { value: yAxisLabel, angle: -90, position: 'insideLeft', style: { fontSize: 12, fill: '#64748b' } } : undefined}
                tickFormatter={formatYAxis}
                tick={{ fontSize: 12, fill: '#64748b' }}
                tickLine={{ stroke: '#cbd5e1' }}
                axisLine={{ stroke: '#cbd5e1' }}
              />
              <Tooltip content={<CustomTimeSeriesTooltip formatTooltip={formatTooltip} />} />
              <Legend
                wrapperStyle={{ paddingTop: '20px' }}
                iconType="line"
                iconSize={16}
              />
              {lines.map((line, index) =>
                line.type === 'area' ? (
                  <Area
                    key={index}
                    type="monotone"
                    dataKey={line.dataKey}
                    name={line.name}
                    stroke={line.color}
                    fill={line.color}
                    fillOpacity={0.2}
                    strokeWidth={2}
                    dot={false}
                    activeDot={{ r: 6, strokeWidth: 0 }}
                  />
                ) : (
                  <Line
                    key={index}
                    type="monotone"
                    dataKey={line.dataKey}
                    name={line.name}
                    stroke={line.color}
                    strokeWidth={2}
                    dot={false}
                    activeDot={{ r: 6, strokeWidth: 0 }}
                  />
                )
              )}
            </LineChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
