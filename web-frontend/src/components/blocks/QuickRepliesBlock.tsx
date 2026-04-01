import React from 'react';
import { QuickReply } from '../../types';

interface QuickRepliesBlockProps {
  options: QuickReply[];
  onSelect?: (value: string) => void;
}

const QuickRepliesBlock: React.FC<QuickRepliesBlockProps> = ({ options, onSelect }) => {
  return (
    <div className="flex flex-wrap gap-2 my-2">
      {options.map((option, idx) => (
        <button
          key={idx}
          onClick={() => onSelect?.(option.value)}
          className="px-3 py-1.5 text-sm bg-bg-tertiary text-text-secondary rounded-full border border-bg-tertiary hover:border-primary hover:text-primary-light transition-all cursor-pointer"
        >
          {option.icon && <span className="mr-1">{option.icon}</span>}
          {option.label}
        </button>
      ))}
    </div>
  );
};

export default QuickRepliesBlock;
