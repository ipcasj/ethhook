'use client';

import { PieChart, Pie, Cell, ResponsiveContainer, Legend, Tooltip } from 'recharts';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export interface ChainDistribution {
  chain_id: number;
  chain_name: string;
  event_count: number;
  percentage: number;
  [key: string]: string | number; // Allow additional properties for recharts compatibility
}

export interface ChainDistributionChartProps {
  title: string;
  description?: string;
  distributions: ChainDistribution[];
  height?: number;
  loading?: boolean;
}

const CHAIN_COLORS: Record<string, string> = {
  'Ethereum': '#627EEA',
  'Polygon': '#8247E5',
  'Arbitrum': '#28A0F0',
  'Optimism': '#FF0420',
  'Base': '#0052FF',
  'zkSync Era': '#8C8DFC',
  'Avalanche': '#E84142',
  'BNB Chain': '#F3BA2F',
    'Other': '#6b7280',
};

const CustomChainTooltip = ({ active, payload }: { active?: boolean; payload?: Array<{ payload: ChainDistribution }> }) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload;
    return (
      <div className="bg-white/95 backdrop-blur-sm border border-slate-200 rounded-lg shadow-xl p-3">
        <p className="text-sm font-semibold text-slate-900 mb-2">{data.chain_name}</p>
        <div className="space-y-1">
          <p className="text-xs text-slate-600">
            Events: <span className="font-semibold text-slate-900">{data.event_count.toLocaleString()}</span>
          </p>
          <p className="text-xs text-slate-600">
            Percentage: <span className="font-semibold text-slate-900">{data.percentage.toFixed(1)}%</span>
          </p>
        </div>
      </div>
    );
  }
  return null;
};

/**
 * Pie chart component for displaying chain distribution
 * Shows event distribution across different blockchain networks
 */
export function ChainDistributionChart({
  title,
  description,
  distributions,
  height = 300,
  loading = false,
}: ChainDistributionChartProps) {
  const renderCustomLabel = ({ cx, cy, midAngle, innerRadius, outerRadius, percentage }: {
    cx: number;
    cy: number;
    midAngle: number;
    innerRadius: number;
    outerRadius: number;
    percentage: number;
  }) => {
    if (percentage < 5) return null; // Don't show label for small slices
    
    const RADIAN = Math.PI / 180;
    const radius = innerRadius + (outerRadius - innerRadius) * 0.5;
    const x = cx + radius * Math.cos(-midAngle * RADIAN);
    const y = cy + radius * Math.sin(-midAngle * RADIAN);

    return (
      <text
        x={x}
        y={y}
        fill="white"
        textAnchor={x > cx ? 'start' : 'end'}
        dominantBaseline="central"
        className="text-xs font-semibold drop-shadow-md"
      >
        {`${percentage.toFixed(0)}%`}
      </text>
    );
  };

  const renderLegend = (props: { payload?: Array<{ value: string; payload: ChainDistribution; color?: string }> }) => {
    const { payload } = props;
    if (!payload) return null;
    return (
      <ul className="flex flex-wrap justify-center gap-4 mt-4">
        {payload.map((entry, index: number) => (
          <li key={`legend-${index}`} className="flex items-center gap-2">
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-sm text-slate-700">{entry.value}</span>
            <span className="text-xs text-slate-500">
              ({entry.payload.event_count.toLocaleString()})
            </span>
          </li>
        ))}
      </ul>
    );
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
        ) : distributions.length === 0 ? (
          <div className="flex items-center justify-center text-slate-500" style={{ height }}>
            No chain distribution data available
          </div>
        ) : (
          <ResponsiveContainer width="100%" height={height}>
            <PieChart>
              <Pie
                data={distributions}
                cx="50%"
                cy="40%"
                labelLine={false}
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                label={renderCustomLabel as any}
                outerRadius={80}
                fill="#8884d8"
                dataKey="event_count"
                nameKey="chain_name"
              >
                {distributions.map((entry, index) => (
                  <Cell
                    key={`cell-${index}`}
                    fill={CHAIN_COLORS[entry.chain_name] || CHAIN_COLORS['Other']}
                  />
                ))}
              </Pie>
              <Tooltip content={<CustomChainTooltip />} />
              {/* eslint-disable-next-line @typescript-eslint/no-explicit-any */}
              <Legend content={renderLegend as any} />
            </PieChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
