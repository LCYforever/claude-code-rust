import React, { useState } from 'react';
import { TaskStep } from '../../types';
import { ChevronDown, ChevronRight, CheckCircle2, Circle, Loader2 } from 'lucide-react';

interface TaskProgressBlockProps {
  steps: TaskStep[];
  collapsed: boolean;
}

const TaskProgressBlock: React.FC<TaskProgressBlockProps> = ({ steps, collapsed: initialCollapsed }) => {
  const [isCollapsed, setIsCollapsed] = useState(initialCollapsed);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle2 className="w-4 h-4 text-success" />;
      case 'running':
        return <Loader2 className="w-4 h-4 text-primary-light animate-spin" />;
      default:
        return <Circle className="w-4 h-4 text-text-muted" />;
    }
  };

  return (
    <div className="bg-bg-tertiary/50 rounded-lg border border-bg-tertiary my-2">
      <button
        onClick={() => setIsCollapsed(!isCollapsed)}
        className="flex items-center gap-2 w-full px-3 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
      >
        {isCollapsed ? <ChevronRight className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
        <span className="font-medium">任务进度 ({steps.filter(s => s.status === 'completed').length}/{steps.length})</span>
      </button>
      {!isCollapsed && (
        <div className="px-3 pb-3 space-y-2">
          {steps.map((step, idx) => (
            <div key={idx} className="flex items-start gap-2">
              {getStatusIcon(step.status)}
              <div>
                <span className={`text-sm ${step.status === 'completed' ? 'text-text-secondary line-through' : 'text-text-primary'}`}>
                  {step.label}
                </span>
                {step.detail && <p className="text-xs text-text-muted mt-0.5">{step.detail}</p>}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default TaskProgressBlock;
