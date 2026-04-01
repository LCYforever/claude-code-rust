import React from 'react';

interface ChartBlockProps {
  chart_type: string;
  data: unknown;
  title?: string;
}

const ChartBlock: React.FC<ChartBlockProps> = ({ title }) => {
  return (
    <div className="my-2 p-4 bg-bg-tertiary/50 rounded-lg border border-bg-tertiary">
      {title && <h4 className="text-sm font-semibold text-text-primary mb-2">{title}</h4>}
      <div className="h-48 flex items-center justify-center text-text-muted text-sm">
        图表渲染区域 (通用 recharts 图表)
      </div>
    </div>
  );
};

export default ChartBlock;
