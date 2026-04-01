import React from 'react';
import { Brain, Star, ListTodo, Sparkles, TrendingUp, Code } from 'lucide-react';

interface QuickActionBarProps {
  onAction: (text: string) => void;
}

const quickActions = [
  { label: '深度思考', icon: Brain, prompt: '请深入分析以下问题：' },
  { label: '任务助手', icon: ListTodo, prompt: '帮我制定一个详细的任务计划：' },
  { label: '智能投研', icon: TrendingUp, prompt: '请分析最近的市场走势和投资机会' },
  { label: '代码助手', icon: Code, prompt: '帮我编写以下功能的代码：' },
  { label: '灵感创意', icon: Sparkles, prompt: '请给我一些关于以下主题的创意建议：' },
  { label: '收藏问题', icon: Star, prompt: '' },
];

const QuickActionBar: React.FC<QuickActionBarProps> = ({ onAction }) => {
  return (
    <div className="flex gap-2 overflow-x-auto scrollbar-hide px-4 py-2">
      {quickActions.map((action, idx) => (
        <button
          key={idx}
          onClick={() => action.prompt && onAction(action.prompt)}
          className="flex-shrink-0 flex items-center gap-1.5 px-3 py-1.5 text-xs bg-bg-tertiary/50 text-text-secondary rounded-full border border-bg-tertiary hover:border-primary hover:text-primary-light transition-all cursor-pointer whitespace-nowrap"
        >
          <action.icon className="w-3.5 h-3.5" />
          {action.label}
        </button>
      ))}
    </div>
  );
};

export default QuickActionBar;
