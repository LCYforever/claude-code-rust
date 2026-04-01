import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface TextBlockProps {
  content: string;
}

const TextBlock: React.FC<TextBlockProps> = ({ content }) => {
  return (
    <div className="prose prose-invert prose-sm max-w-none">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          p: ({ children }) => <p className="text-text-primary leading-relaxed mb-2 last:mb-0">{children}</p>,
          h1: ({ children }) => <h1 className="text-xl font-bold text-text-primary mt-4 mb-2">{children}</h1>,
          h2: ({ children }) => <h2 className="text-lg font-semibold text-text-primary mt-3 mb-2">{children}</h2>,
          h3: ({ children }) => <h3 className="text-base font-semibold text-text-primary mt-2 mb-1">{children}</h3>,
          code: ({ className, children, ...props }) => {
            const isInline = !className;
            if (isInline) {
              return (
                <code className="bg-bg-tertiary text-primary-light px-1.5 py-0.5 rounded text-sm font-mono" {...props}>
                  {children}
                </code>
              );
            }
            return (
              <code className={`block bg-bg-primary p-3 rounded-lg text-sm font-mono overflow-x-auto ${className}`} {...props}>
                {children}
              </code>
            );
          },
          pre: ({ children }) => <pre className="bg-bg-primary rounded-lg overflow-x-auto my-2">{children}</pre>,
          ul: ({ children }) => <ul className="list-disc list-inside space-y-1 text-text-secondary">{children}</ul>,
          ol: ({ children }) => <ol className="list-decimal list-inside space-y-1 text-text-secondary">{children}</ol>,
          li: ({ children }) => <li className="text-text-secondary">{children}</li>,
          a: ({ href, children }) => (
            <a href={href} className="text-primary-light hover:text-primary underline transition-colors" target="_blank" rel="noopener noreferrer">
              {children}
            </a>
          ),
          blockquote: ({ children }) => (
            <blockquote className="border-l-3 border-primary pl-3 my-2 text-text-muted italic">{children}</blockquote>
          ),
          table: ({ children }) => (
            <div className="overflow-x-auto my-2">
              <table className="min-w-full border-collapse">{children}</table>
            </div>
          ),
          th: ({ children }) => <th className="bg-bg-tertiary px-3 py-2 text-left text-sm font-semibold text-text-primary border border-bg-tertiary">{children}</th>,
          td: ({ children }) => <td className="px-3 py-2 text-sm text-text-secondary border border-bg-tertiary">{children}</td>,
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

export default TextBlock;
