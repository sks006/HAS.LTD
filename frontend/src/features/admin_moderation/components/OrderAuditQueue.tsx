import React from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/shared/ui/card';
import { Badge } from '@/shared/ui/badge';
import { Button } from '@/shared/ui/button';
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from '@/shared/ui/table';
import { OrderDto } from '@/shared/types/contracts';
import { ShieldCheck, Check, X, Clock } from 'lucide-react';

interface OrderQueueProps {
  orders?: OrderDto[];
  onReviewOrder?: (id: string, action: 'APPROVE' | 'REJECT') => void;
}

export const OrderAuditQueue: React.FC<OrderQueueProps> = ({ orders = [], onReviewOrder }) => {
  return (
    <Card className="border-neutral-200/80 shadow-2xs bg-white">
      <CardHeader className="pb-3 border-b border-neutral-100 flex flex-row items-center justify-between">
        <div>
          <CardTitle className="font-serif text-base font-bold text-neutral-900 flex items-center space-x-2">
            <ShieldCheck size={18} className="text-amber-800" />
            <span>Pending Order Audit Queue</span>
          </CardTitle>
          <p className="text-xs text-neutral-500 mt-0.5">
            Moderator inspection and manual transaction validation queue
          </p>
        </div>
        <Badge variant="outline" className="border-amber-300 text-amber-800 font-mono text-[10px]">
          Live Review Mode
        </Badge>
      </CardHeader>

      <CardContent className="p-0">
        <Table>
          <TableHeader className="bg-neutral-50/70">
            <TableRow>
              <TableHead className="text-xs font-semibold text-neutral-500">Order ID</TableHead>
              <TableHead className="text-xs font-semibold text-neutral-500">Lifecycle State</TableHead>
              <TableHead className="text-xs font-semibold text-neutral-500">Settlement Total</TableHead>
              <TableHead className="text-xs font-semibold text-neutral-500 text-right">Moderator Review</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {orders.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="h-32 text-center text-xs text-neutral-400">
                  <Clock size={20} className="mx-auto mb-1 text-neutral-300" />
                  No orders currently queued for manual audit review.
                </TableCell>
              </TableRow>
            ) : (
              orders.map((o) => (
                <TableRow key={o.id} className="hover:bg-neutral-50/80 transition-colors">
                  <TableCell className="font-mono text-xs font-semibold text-neutral-800">
                    {o.id}
                  </TableCell>
                  <TableCell>
                    <Badge variant="secondary" className="font-mono text-[10px] uppercase">
                      {o.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="font-mono text-xs font-bold text-neutral-900">
                    {o.currency} {o.total}
                  </TableCell>
                  <TableCell className="text-right space-x-2">
                    <Button
                      size="sm"
                      variant="luxury"
                      className="h-7 text-xs font-mono uppercase"
                      onClick={() => onReviewOrder?.(o.id, 'APPROVE')}
                    >
                      <Check size={12} className="mr-1" />
                      <span>Approve</span>
                    </Button>
                    <Button
                      size="sm"
                      variant="destructive"
                      className="h-7 text-xs font-mono uppercase"
                      onClick={() => onReviewOrder?.(o.id, 'REJECT')}
                    >
                      <X size={12} className="mr-1" />
                      <span>Reject</span>
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
};

export default OrderAuditQueue;
