import React from 'react';

interface TableBlockProps {
  headers: string[];
  rows: string[][];
  title?: string;
}

const TableBlock: React.FC<TableBlockProps> = ({ headers, rows, title }) => {
  return (
    <div className="my-2 overflow-x-auto">
      {title && <h4 className="text-sm font-semibold text-text-primary mb-2">{title}</h4>}
      <table className="min-w-full border-collapse rounded-lg overflow-hidden">
        <thead>
          <tr>
            {headers.map((h, i) => (
              <th key={i} className="bg-bg-tertiary px-3 py-2 text-left text-xs font-semibold text-text-secondary uppercase tracking-wider">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, ri) => (
            <tr key={ri} className={ri % 2 === 0 ? 'bg-bg-secondary/50' : 'bg-bg-primary/50'}>
              {row.map((cell, ci) => (
                <td key={ci} className="px-3 py-2 text-sm text-text-primary whitespace-nowrap">
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default TableBlock;
