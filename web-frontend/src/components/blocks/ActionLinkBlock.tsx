import React from 'react';
import { ActionPayload } from '../../types';
import { ExternalLink } from 'lucide-react';

interface ActionLinkBlockProps {
  label: string;
  icon?: string;
  action: ActionPayload;
}

const ActionLinkBlock: React.FC<ActionLinkBlockProps> = ({ label, action }) => {
  const handleClick = () => {
    if (action.type === 'url' && action.url) {
      window.open(action.url, '_blank', 'noopener,noreferrer');
    }
  };

  return (
    <button
      onClick={handleClick}
      className="inline-flex items-center gap-1.5 text-sm text-primary-light hover:text-primary transition-colors cursor-pointer my-1"
    >
      <ExternalLink className="w-3.5 h-3.5" />
      <span className="underline underline-offset-2">{label}</span>
    </button>
  );
};

export default ActionLinkBlock;
