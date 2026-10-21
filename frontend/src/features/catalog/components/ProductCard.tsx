'use client';

import { useMemo } from 'react';
import { Heart, Minus, Plus, ShoppingBag } from 'lucide-react';
import { Button } from '@/shared/ui/Button';
import type { ProductDto } from '@/shared/types/contracts';
import { useStore } from '@/shared/store/store';
import { renderStars, filters } from './Stars';

export function ProductGrid() {
  const products = useStore((s) => s.products);
  const activeFilter = useStore((s) => s.activeFilter);
  const setActiveFilter = useStore((s) => s.setActiveFilter);
  const addToCart = useStore((s) => s.addToCart);
  const [wishlist, setWishlist] = useWishlist();

  const visibleProducts = useMemo(
    () => activeFilter === 'All pieces' ? products : products.filter((p) => p.category === activeFilter),
    [products, activeFilter],
  );

  const toggleWishlist = (id: string) => {
    setWishlist((current: string[]) => current.includes(id) ? current.filter((item) => item !== id) : [...current, id]);
  };

  return (
    <>
      <div className="mb-8 flex flex-col gap-5 border-b border-[#e8e8e8] pb-6 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.3em] text-[#8e8e8e]">The latest edit</p>
          <h1 className="font-serif text-[30px] tracking-[-0.03em] sm:text-[38px]">Quietly distinctive.</h1>
        </div>
        <div className="flex flex-wrap gap-2" id="collections">
          {filters.map((filter) => (
            <button key={filter} type="button" onClick={() => setActiveFilter(filter)} className={`filter-pill ${activeFilter === filter ? 'selected' : ''}`}>{filter}</button>
          ))}
        </div>
      </div>

      <div id="collection" className="grid grid-cols-1 gap-x-6 gap-y-10 sm:grid-cols-2 lg:grid-cols-3">
        {visibleProducts.map((product) => (
          <ProductCard key={product.id} product={product} isWishlisted={wishlist.includes(product.id)} onWishlist={() => toggleWishlist(product.id)} onAddToCart={() => addToCart({ productId: product.id, name: product.name, quantity: 1, price: product.price })} />
        ))}
      </div>

      {visibleProducts.length === 0 && <p className="py-16 text-center text-sm text-[#777]">No pieces in this edit yet.</p>}
    </>
  );
}

function ProductCard({ product, isWishlisted, onWishlist, onAddToCart }: { product: ProductDto; isWishlisted: boolean; onWishlist: () => void; onAddToCart: () => void }) {
  const [qty, setQty] = useQty(product.id);
  const addToCartStore = useStore((s) => s.addToCart);

  return (
    <article className="group">
      <div className="product-image-wrap">
        {product.sale && <span className="sale-badge">Sale</span>}
        <img src={product.image} alt={product.alt} className="product-image" />
        <button type="button" aria-label={isWishlisted ? `Remove ${product.name} from wishlist` : `Add ${product.name} to wishlist`} onClick={onWishlist} className={`wishlist-button ${isWishlisted ? 'liked' : ''}`}><Heart size={18} fill={isWishlisted ? 'currentColor' : 'none'} strokeWidth={1.6} /></button>
      </div>
      <div className="mt-4 flex items-start justify-between gap-3 px-1">
        <div>
          <h2 className="text-[14px] font-medium tracking-[-0.01em]">{product.name}</h2>
          <p className="mt-1 text-[11px] text-[#858585]">{product.category}</p>
        </div>
        <span className="text-[13px] font-medium">{product.currency} {product.price}</span>
      </div>
      <div className="mt-2 flex items-center gap-1.5 px-1">
        <div className="flex items-center gap-0.5">{renderStars(product.rating)}</div>
        <span className="text-[11px] text-[#858585]"><span className="font-medium text-[#444]">{product.rating.toFixed(1)}</span> ({product.reviews})</span>
      </div>
      <div className="mt-4 flex items-center gap-3 px-1">
        <div className="flex h-10 items-center rounded-full border border-[#d5d5d5]">
          <button type="button" aria-label={`Decrease quantity of ${product.name}`} onClick={() => setQty((q) => Math.max(1, q - 1))} className="flex h-10 w-10 items-center justify-center rounded-full text-[#333] transition-colors hover:bg-[#f5f5f5] disabled:opacity-40" disabled={qty <= 1}><Minus size={15} strokeWidth={2} /></button>
          <span className="w-7 text-center text-sm font-medium tabular-nums">{qty}</span>
          <button type="button" aria-label={`Increase quantity of ${product.name}`} onClick={() => setQty((q) => q + 1)} className="flex h-10 w-10 items-center justify-center rounded-full text-[#333] transition-colors hover:bg-[#f5f5f5]"><Plus size={15} strokeWidth={2} /></button>
        </div>
        <Button onClick={() => { addToCartStore({ productId: product.id, name: product.name, quantity: qty, price: product.price }); }} className="h-10 flex-1 rounded-full bg-black text-xs font-semibold text-white transition-colors hover:bg-[#242424]"><ShoppingBag size={14} strokeWidth={2} className="mr-2" />Add to cart</Button>
      </div>
      <div className="mt-2.5 flex gap-2 px-1">
        <Button onClick={() => { addToCartStore({ productId: product.id, name: product.name, quantity: qty, price: product.price }); }} className="h-10 flex-1 rounded-full bg-[#1a1a1a] text-xs font-semibold text-white transition-colors hover:bg-black">Order now</Button>
        <Button variant="outline" className="h-10 flex-1 rounded-full border-[#d5d5d5] bg-white text-xs font-medium text-[#333] transition-colors hover:bg-[#f7f7f7]">Message</Button>
      </div>
    </article>
  );
}

import { useState } from 'react';
function useQty(productId: string) {
  const [quantities, setQuantities] = useState<Record<string, number>>({});
  const qty = quantities[productId] ?? 1;
  const setQty = (updater: (q: number) => number) => setQuantities((c) => ({ ...c, [productId]: updater(c[productId] ?? 1) }));
  return [qty, setQty] as const;
}

function useWishlist() {
  const [wishlist, setWishlist] = useState<string[]>([]);
  return [wishlist, setWishlist] as const;
}
