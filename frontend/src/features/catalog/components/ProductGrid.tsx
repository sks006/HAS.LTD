import React from 'react';
import { useRootStore } from '@/slicers/root_store';
import { ProductDto } from '@/shared/types/contracts';
import Button from '@/shared/ui/Button';

export const ProductGrid: React.FC = () => {
  const products = useRootStore((s) => s.products);
  const activeFilter = useRootStore((s) => s.activeFilter);
  const searchQuery = useRootStore((s) => s.searchQuery);
  const addToCart = useRootStore((s) => s.addToCart);

  const filteredProducts = products.filter((product) => {
    const matchesFilter =
      activeFilter === 'All Abayas' || activeFilter === 'All pieces' || product.category === activeFilter;
    const matchesSearch = product.name
      .toLowerCase()
      .includes(searchQuery.toLowerCase());
    return matchesFilter && matchesSearch;
  });

  if (filteredProducts.length === 0) {
    return (
      <div className="text-center py-16 text-slateblue bg-white border border-lightgray rounded-2xl p-8 shadow-sm">
        <p className="font-semibold text-navy">No pieces found in this category.</p>
        <p className="text-xs text-slateblue mt-1">Try selecting another collection or clearing your search.</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {filteredProducts.map((product) => (
        <ProductCardItem
          key={product.id}
          product={product}
          onAddToCart={() =>
            addToCart({
              productId: product.id,
              name: product.name,
              quantity: 1,
              price: product.price,
            })
          }
        />
      ))}
    </div>
  );
};

const ProductCardItem: React.FC<{ product: ProductDto; onAddToCart: () => void }> = ({
  product,
  onAddToCart,
}) => {
  return (
    <div className="bg-white border border-lightgray rounded-2xl overflow-hidden flex flex-col justify-between p-4 space-y-4 hover:shadow-lg transition-all duration-300 group">
      <div className="aspect-[0.85/1] w-full bg-cream rounded-xl overflow-hidden relative">
        <img
          src={product.image}
          alt={product.alt}
          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
        />
        {product.sale && (
          <span className="absolute top-3 left-3 bg-taupe text-white text-[10px] uppercase font-bold px-2.5 py-1 rounded-full tracking-wider shadow-sm">
            Couture Sale
          </span>
        )}
      </div>

      <div className="space-y-1">
        <span className="text-[11px] text-slateblue uppercase font-bold tracking-wider">
          {product.category}
        </span>
        <h3 className="font-bold text-navy text-base leading-snug group-hover:text-taupe transition-colors">
          {product.name}
        </h3>
        <p className="text-navy font-black text-lg pt-1">
          {product.currency} {product.price.toLocaleString()}
        </p>
      </div>

      <Button variant="primary" size="sm" onClick={onAddToCart} className="w-full rounded-xl">
        Add to Bag
      </Button>
    </div>
  );
};

export default ProductGrid;
