import React, { useState } from 'react';
import { MessageBlock } from '../../types';
import { Smartphone, CheckCircle2, XCircle, Loader2, AlertCircle } from 'lucide-react';
import { sendNativeCallback } from '../../api/client';

type NativeCallBlock = Extract<MessageBlock, { type: 'native_call' }>;

interface NativeCallBlockProps {
  block: NativeCallBlock;
  sessionId?: string;
}

const NativeCallBlock: React.FC<NativeCallBlockProps> = ({ block, sessionId }) => {
  const [status, setStatus] = useState<string>(block.status || 'pending');
  const [result, setResult] = useState<unknown>(block.result);

  const handleExecute = async () => {
    if (!window.NativeBridge) {
      setStatus('failed');
      setResult({ error: 'NativeBridge not available' });
      return;
    }

    setStatus('executing');
    try {
      const res = await window.NativeBridge.call(block.method, block.params);
      setStatus('success');
      setResult(res);

      if (block.callback_id) {
        await sendNativeCallback(block.callback_id, sessionId || '', res, true);
      }
    } catch (err) {
      setStatus('failed');
      setResult({ error: String(err) });

      if (block.callback_id) {
        await sendNativeCallback(block.callback_id, sessionId || '', { error: String(err) }, false);
      }
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'executing':
        return <Loader2 className="w-4 h-4 text-primary-light animate-spin" />;
      case 'success':
        return <CheckCircle2 className="w-4 h-4 text-success" />;
      case 'failed':
        return <XCircle className="w-4 h-4 text-danger" />;
      default:
        return <AlertCircle className="w-4 h-4 text-warning" />;
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'executing': return '正在调用...';
      case 'success': return '调用成功';
      case 'failed': return '调用失败';
      default: return '等待执行';
    }
  };

  return (
    <div className="my-2 p-3 bg-bg-tertiary/50 rounded-lg border border-bg-tertiary">
      <div className="flex items-center gap-2 mb-2">
        <Smartphone className="w-4 h-4 text-primary-light" />
        <span className="text-sm font-medium text-text-primary">
          {block.label || `调用原生接口: ${block.method}`}
        </span>
      </div>

      <div className="flex items-center gap-2 text-xs text-text-muted mb-2">
        {getStatusIcon()}
        <span>{getStatusText()}</span>
      </div>

      {block.params && (
        <div className="text-xs text-text-muted bg-bg-primary rounded p-2 mb-2 font-mono">
          {JSON.stringify(block.params, null, 2)}
        </div>
      )}

      {!block.auto_execute && status === 'pending' && (
        <button
          onClick={handleExecute}
          className="px-3 py-1.5 text-sm bg-primary hover:bg-primary-dark text-white rounded-md transition-colors cursor-pointer"
        >
          确认调用
        </button>
      )}

      {result !== undefined && result !== null && (
        <div className="mt-2 text-xs text-text-muted bg-bg-primary rounded p-2 font-mono">
          结果: {typeof result === 'string' ? result : JSON.stringify(result, null, 2)}
        </div>
      )}
    </div>
  );
};

export default NativeCallBlock;
