import React from 'react';
import { DelegationStatus } from '../../types';
import { Bot, CheckCircle2, XCircle, Loader2, Clock } from 'lucide-react';

interface DelegationBlockProps {
  targetAgentId: string;
  targetAgentName: string;
  task: string;
  status: DelegationStatus;
}

const DelegationBlock: React.FC<DelegationBlockProps> = ({ targetAgentName, task, status }) => {
  const getStatusConfig = () => {
    switch (status) {
      case 'pending':
        return { icon: <Clock className="w-4 h-4" />, color: 'text-warning', bg: 'border-warning/20', label: '等待执行' };
      case 'running':
        return { icon: <Loader2 className="w-4 h-4 animate-spin" />, color: 'text-primary-light', bg: 'border-primary/20', label: '执行中' };
      case 'completed':
        return { icon: <CheckCircle2 className="w-4 h-4" />, color: 'text-success', bg: 'border-success/20', label: '已完成' };
      case 'failed':
        return { icon: <XCircle className="w-4 h-4" />, color: 'text-danger', bg: 'border-danger/20', label: '失败' };
    }
  };

  const config = getStatusConfig();

  return (
    <div className={`my-2 p-3 rounded-lg border ${config.bg} bg-bg-tertiary/30`}>
      <div className="flex items-center gap-2 mb-1">
        <Bot className={`w-4 h-4 ${config.color}`} />
        {status !== 'completed' && (
          <span className="text-sm font-medium text-text-primary">
            正在委托给 <span className="text-primary-light">[{targetAgentName}]</span> 处理
          </span>
        )}
        <span className={`flex items-center gap-1 text-xs ${config.color}`}>
          {config.icon}
          {config.label}
        </span>
      </div>
      {status !== 'completed' && (
        <p className="text-xs text-text-muted ml-6">{task}</p>
      )}
    </div>
  );
};

export default DelegationBlock;
