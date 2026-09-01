import React from 'react';
import { RequireModerator } from '@/features/auth/guards/RequireModerator';
import { OrderAuditQueue } from '@/features/admin_moderation/components/OrderAuditQueue';
import { ProductReviewTable } from '@/features/admin_moderation/components/ProductReviewTable';
import { useRootStore } from '@/slicers/root_store';
import { useNavigate } from 'react-router-dom';
import Button from '@/shared/ui/Button';

export const ModeratorPanelContent: React.FC = () => {
  const navigate = useNavigate();
  const products = useRootStore((s) => s.products);

  const dummyOrders = [
    {
      id: 'ord-8831',
      items: [{ productId: 'prod-1', name: 'Silk Organza Abaya', quantity: 1, price: 1850 }],
      total: 1850,
      currency: 'SAR',
      status: 'pending' as const,
      createdAt: new Date().toISOString(),
    },
  ];

  return (
    <div className="min-h-screen bg-white text-navy p-6 md:p-12 space-y-8 font-sans">
      <div className="flex items-center justify-between border-b border-lightgray pb-6">
        <div>
          <span className="text-xs uppercase font-mono text-taupe font-bold">Moderation Zone</span>
          <h1 className="text-3xl font-extrabold text-navy mt-1">Moderator Audit Console</h1>
        </div>
        <Button variant="outline" size="sm" onClick={() => navigate('/')}>
          Storefront
        </Button>
      </div>

      <div className="space-y-12">
        <OrderAuditQueue orders={dummyOrders} />
        <ProductReviewTable products={products} />
      </div>
    </div>
  );
};

export const ModeratorPanel: React.FC = () => (
  <RequireModerator>
    <ModeratorPanelContent />
  </RequireModerator>
);

export default ModeratorPanel;
