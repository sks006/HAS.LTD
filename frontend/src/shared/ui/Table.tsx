import React from 'react';

export interface TableProps {
  headers: string[];
  children: React.ReactNode;
}

export const Table: React.FC<TableProps> = ({ headers, children }) => {
  return (
    <div className="overflow-x-auto rounded-xl border border-slate-800 bg-slate-900 shadow-xl">
      <table className="w-full text-left text-sm text-slate-200">
        <thead className="bg-slate-950/60 text-xs uppercase tracking-wider text-slate-400 border-b border-slate-800">
          <tr>
            {headers.map((h, i) => (
              <th key={i} className="px-6 py-4 font-semibold">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-800/60">{children}</tbody>
      </table>
    </div>
  );
};

export default Table;
