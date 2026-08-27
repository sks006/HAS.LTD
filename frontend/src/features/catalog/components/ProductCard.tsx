import React from 'react';
import { ProductDto } from '../../../shared/types/contracts';
import { Button } from '../../../shared/ui/Button';

interface ProductCardProps {
  product: ProductDto;
  onAddToCart: (product: ProductDto) => void;
}

export function ProductCard({ product, onAddToCart }: ProductCardProps) {
  return (
    <div className="group relative bg-slate-900/40 hover:bg-slate-900/80 border border-slate-800/80 hover:border-indigo-500/50 rounded-xl overflow-hidden shadow-xl hover:shadow-indigo-500/5 transition-all duration-500 flex flex-col h-full">
      {/* Product Image placeholder with modern visual pattern */}
      <div className="aspect-video w-full bg-gradient-to-br from-indigo-950/40 via-purple-950/20 to-slate-950/40 flex items-center justify-center border-b border-slate-800/40 relative overflow-hidden">
        <span className="text-slate-600 font-mono tracking-wider text-xs">HAS.LTD PRODUCT</span>
        <div className="absolute inset-0 bg-gradient-to-t from-slate-950/60 to-transparent" />
      </div>

      <div className="p-5 flex-1 flex flex-col justify-between">
        <div>
          <h4 className="text-white font-bold tracking-tight text-lg group-hover:text-indigo-400 transition-colors duration-300">
            {product.name}
          </h4>
          <p className="text-slate-400 font-mono text-sm mt-1">
            ${(product.price_cents / 100).toFixed(2)}
          </p>
        </div>
        
        <div className="mt-5">
          <Button 
            variant="primary" 
            className="w-full text-sm py-2 px-3 opacity-90 group-hover:opacity-100 transition-all duration-300"
            onClick={() => onAddToCart(product)}
          >
            Add to Cart
          </Button>
        </div>
      </div>
    </div>
  );
}
