import React from 'react';
import { RichTextNode } from '../../types';

interface RichTextBlockProps {
  nodes: RichTextNode[];
}

const RichTextBlock: React.FC<RichTextBlockProps> = ({ nodes }) => {
  return (
    <div className="text-sm text-text-primary leading-relaxed">
      {nodes.map((node, idx) => {
        switch (node.type) {
          case 'link':
            return (
              <a
                key={idx}
                href={node.url}
                className="text-primary-light hover:text-primary underline transition-colors cursor-pointer"
                target="_blank"
                rel="noopener noreferrer"
              >
                {node.content}
              </a>
            );
          case 'bold':
            return <strong key={idx} className="font-semibold">{node.content}</strong>;
          case 'italic':
            return <em key={idx}>{node.content}</em>;
          case 'code':
            return (
              <code key={idx} className="bg-bg-tertiary text-primary-light px-1 py-0.5 rounded text-xs font-mono">
                {node.content}
              </code>
            );
          default:
            return <span key={idx}>{node.content}</span>;
        }
      })}
    </div>
  );
};

export default RichTextBlock;
