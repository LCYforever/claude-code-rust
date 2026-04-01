import React from 'react';
import { MessageBlock } from '../types';
import TextBlock from './blocks/TextBlock';
import RichTextBlock from './blocks/RichTextBlock';
import TaskProgressBlock from './blocks/TaskProgressBlock';
import StockCardBlock from './blocks/StockCardBlock';
import TableBlock from './blocks/TableBlock';
import ChartBlock from './blocks/ChartBlock';
import ActionLinkBlock from './blocks/ActionLinkBlock';
import ImageBlock from './blocks/ImageBlock';
import QuickRepliesBlock from './blocks/QuickRepliesBlock';
import NavigateBlock from './blocks/NavigateBlock';
import NativeCallBlock from './blocks/NativeCallBlock';
import DelegationBlock from './blocks/DelegationBlock';

interface BlockRendererProps {
  block: MessageBlock;
  onQuickReply?: (value: string) => void;
}

const BlockRenderer: React.FC<BlockRendererProps> = ({ block, onQuickReply }) => {
  switch (block.type) {
    case 'text':
      return <TextBlock content={block.content} />;
    case 'rich_text':
      return <RichTextBlock nodes={block.nodes} />;
    case 'task_progress':
      return <TaskProgressBlock steps={block.steps} collapsed={block.collapsed} />;
    case 'stock_card':
      return <StockCardBlock data={block.data} />;
    case 'table':
      return <TableBlock headers={block.headers} rows={block.rows} title={block.title} />;
    case 'chart':
      return <ChartBlock chart_type={block.chart_type} data={block.data} title={block.title} />;
    case 'action_link':
      return <ActionLinkBlock label={block.label} icon={block.icon} action={block.action} />;
    case 'image':
      return <ImageBlock url={block.url} alt={block.alt} />;
    case 'quick_replies':
      return <QuickRepliesBlock options={block.options} onSelect={onQuickReply} />;
    case 'navigate':
      return <NavigateBlock target={block.target} params={block.params} label={block.label} />;
    case 'native_call':
      return <NativeCallBlock block={block} />;
    case 'delegation':
      return (
        <DelegationBlock
          targetAgentId={block.target_agent_id}
          targetAgentName={block.target_agent_name}
          task={block.task}
          status={block.status}
        />
      );
    default:
      return null;
  }
};

export default BlockRenderer;
