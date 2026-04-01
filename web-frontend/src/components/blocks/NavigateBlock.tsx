import React from 'react';
import { Navigation } from 'lucide-react';

interface NavigateBlockProps {
  target: string;
  params?: Record<string, unknown>;
  label?: string;
}

const NavigateBlock: React.FC<NavigateBlockProps> = ({ target, label }) => {
  return (
    <div className="flex items-center gap-2 my-2 p-2 bg-bg-tertiary/50 rounded-lg border border-bg-tertiary">
      <Navigation className="w-4 h-4 text-primary-light" />
      <span className="text-sm text-text-secondary">
        {label || `跳转到 ${target}`}
      </span>
    </div>
  );
};

export default NavigateBlock;
