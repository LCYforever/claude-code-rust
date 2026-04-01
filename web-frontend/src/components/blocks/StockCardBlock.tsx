import React from 'react';
import { StockCardData } from '../../types';
import { TrendingUp, TrendingDown } from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

interface StockCardBlockProps {
  data: StockCardData;
}

const StockCardBlock: React.FC<StockCardBlockProps> = ({ data }) => {
  const isPositive = data.change >= 0;

  return (
    <div className="bg-bg-tertiary rounded-lg border border-bg-tertiary p-4 my-2 max-w-sm">
      <div className="flex items-center justify-between mb-3">
        <div>
          <h3 className="text-text-primary font-semibold text-base">{data.name}</h3>
          <span className="text-text-muted text-xs">{data.code}</span>
        </div>
        <div className="text-right">
          <div className="text-xl font-bold text-text-primary">{data.price.toFixed(2)}</div>
          <div className={`flex items-center gap-1 text-sm ${isPositive ? 'text-success' : 'text-danger'}`}>
            {isPositive ? <TrendingUp className="w-3 h-3" /> : <TrendingDown className="w-3 h-3" />}
            <span>{isPositive ? '+' : ''}{data.change.toFixed(2)}</span>
            <span>({isPositive ? '+' : ''}{data.change_percent.toFixed(2)}%)</span>
          </div>
        </div>
      </div>

      {data.chart_data && data.chart_data.length > 0 && (
        <div className="h-24 mt-2">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data.chart_data}>
              <XAxis dataKey="time" hide />
              <YAxis hide domain={['auto', 'auto']} />
              <Tooltip
                contentStyle={{ background: '#1A1A24', border: '1px solid #252533', borderRadius: '8px', color: '#F1F5F9' }}
                labelStyle={{ color: '#94A3B8' }}
              />
              <Line
                type="monotone"
                dataKey="value"
                stroke={isPositive ? '#22C55E' : '#EF4444'}
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {data.volume !== undefined && (
        <div className="mt-2 text-xs text-text-muted">
          成交量: {(data.volume / 10000).toFixed(2)}万
        </div>
      )}
    </div>
  );
};

export default StockCardBlock;
