import React from 'react';
import Table from '@/shared/ui/Table';
import Button from '@/shared/ui/Button';
import { ProductDto } from '@/shared/types/contracts';

interface ReviewProps {
  products?: ProductDto[];
  onToggleStatus?: (productId: string) => void;
}

export const ProductReviewTable: React.FC<ReviewProps> = ({ products = [], onToggleStatus }) => {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-bold text-navy">Product Moderation Review</h3>
        <span className="text-xs bg-navy/10 border border-navy/20 text-navy font-mono px-3 py-1 rounded-full font-semibold">
          Catalog Guard
        </span>
      </div>

      <Table headers={['Product ID', 'Name', 'Category', 'Price', 'Actions']}>
        {products.length === 0 ? (
          <tr>
            <td colSpan={5} className="px-6 py-8 text-center text-slateblue text-sm">
              No products available for review.
            </td>
          </tr>
        ) : (
          products.map((p) => (
            <tr key={p.id} className="hover:bg-cream/40 border-b border-lightgray transition-colors">
              <td className="px-6 py-4 font-mono text-xs text-navy">{p.id}</td>
              <td className="px-6 py-4 text-navy font-semibold">{p.name}</td>
              <td className="px-6 py-4 text-slateblue text-xs">{p.category}</td>
              <td className="px-6 py-4 text-navy">
                {p.currency} {p.price}
              </td>
              <td className="px-6 py-4">
                <Button size="sm" variant="outline" onClick={() => onToggleStatus?.(p.id)}>
                  Toggle Status
                </Button>
              </td>
            </tr>
          ))
        )}
      </Table>
    </div>
  );
};

export default ProductReviewTable;
