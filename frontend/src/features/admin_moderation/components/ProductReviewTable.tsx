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
import { ProductDto } from '@/shared/types/contracts';
import { Eye, Package } from 'lucide-react';

interface ReviewProps {
  products?: ProductDto[];
  onToggleStatus?: (productId: string) => void;
}

export const ProductReviewTable: React.FC<ReviewProps> = ({ products = [], onToggleStatus }) => {
  return (
    <Card className="border-neutral-200/80 shadow-2xs bg-white">
      <CardHeader className="pb-3 border-b border-neutral-100 flex flex-row items-center justify-between">
        <div>
          <CardTitle className="font-serif text-base font-bold text-neutral-900 flex items-center space-x-2">
            <Package size={18} className="text-amber-800" />
            <span>Product Catalog Moderation</span>
          </CardTitle>
          <p className="text-xs text-neutral-500 mt-0.5">
            Active listing controls & verified inventory verification
          </p>
        </div>
        <Badge variant="outline" className="border-neutral-300 text-neutral-600 font-mono text-[10px]">
          Catalog Guard
        </Badge>
      </CardHeader>

      <CardContent className="p-0">
        <Table>
          <TableHeader className="bg-neutral-50/70">
            <TableRow>
              <TableHead className="text-xs font-semibold text-neutral-500">Product Identifier</TableHead>
              <TableHead className="text-xs font-semibold text-neutral-500">Name</TableHead>
              <TableHead className="text-xs font-semibold text-neutral-500">Fabric & Category</TableHead>
              <TableHead className="text-xs font-semibold text-neutral-500">Retail Price</TableHead>
              <TableHead className="text-xs font-semibold text-neutral-500 text-right">Moderation Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {products.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="h-32 text-center text-xs text-neutral-400">
                  No catalog products available for review.
                </TableCell>
              </TableRow>
            ) : (
              products.map((p) => (
                <TableRow key={p.id} className="hover:bg-neutral-50/80 transition-colors">
                  <TableCell className="font-mono text-xs font-semibold text-neutral-800">
                    {p.id}
                  </TableCell>
                  <TableCell className="font-semibold text-xs text-neutral-900">
                    {p.name}
                  </TableCell>
                  <TableCell className="text-xs text-neutral-500">
                    {p.category}
                  </TableCell>
                  <TableCell className="font-mono text-xs font-bold text-neutral-900">
                    {p.currency} {p.price.toLocaleString()}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button
                      size="sm"
                      variant="outline"
                      className="h-7 text-xs font-mono"
                      onClick={() => onToggleStatus?.(p.id)}
                    >
                      <Eye size={12} className="mr-1" />
                      <span>Toggle Visibility</span>
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

export default ProductReviewTable;
