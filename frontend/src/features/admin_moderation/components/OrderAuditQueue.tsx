import React from 'react';
import Table from '@/shared/ui/Table';
import Button from '@/shared/ui/Button';
import { OrderDto } from '@/shared/types/contracts';

interface OrderQueueProps {
  orders?: OrderDto[];
  onReviewOrder?: (id: string, action: 'APPROVE' | 'REJECT') => void;
}

export const OrderAuditQueue: React.FC<OrderQueueProps> = ({ orders = [], onReviewOrder }) => {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-bold text-navy">Pending Order Audit Queue</h3>
        <span className="text-xs bg-amber-50 border border-amber-200 text-amber-800 font-mono px-3 py-1 rounded-full font-semibold">
          Moderator Audit Mode
        </span>
      </div>

      <Table headers={['Order ID', 'Status', 'Total', 'Action']}>
        {orders.length === 0 ? (
          <tr>
            <td colSpan={4} className="px-6 py-8 text-center text-slateblue text-sm">
              No orders pending moderation audit.
            </td>
          </tr>
        ) : (
          orders.map((o) => (
            <tr key={o.id} className="hover:bg-cream/40 border-b border-lightgray transition-colors">
              <td className="px-6 py-4 font-mono text-xs text-navy">{o.id}</td>
              <td className="px-6 py-4">
                <span className="px-2.5 py-1 bg-amber-50 text-amber-800 text-xs font-bold rounded">
                  {o.status}
                </span>
              </td>
              <td className="px-6 py-4 text-navy font-semibold">
                {o.currency} {o.total}
              </td>
              <td className="px-6 py-4 flex space-x-2">
                <Button size="sm" variant="primary" onClick={() => onReviewOrder?.(o.id, 'APPROVE')}>
                  Approve
                </Button>
                <Button size="sm" variant="danger" onClick={() => onReviewOrder?.(o.id, 'REJECT')}>
                  Reject
                </Button>
              </td>
            </tr>
          ))
        )}
      </Table>
    </div>
  );
};

export default OrderAuditQueue;
