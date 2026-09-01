import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import ThreeDViewerFallback from '@/features/catalog/components/3DViewerFallback';
import Button from '@/shared/ui/Button';
import { useRootStore } from '@/slicers/root_store';

export const ProductDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const addToCart = useRootStore((s) => s.addToCart);

  const dummyProduct = {
    id: id || 'prod-1',
    name: 'Silk Organza Abaya',
    category: 'Abayas',
    price: 1850,
    currency: 'SAR',
    image: 'https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?auto=format&fit=crop&q=80&w=800',
    alt: 'Silk Organza Abaya',
    sale: true,
    rating: 4.9,
    reviews: 28,
  };

  return (
    <div className="min-h-screen bg-white text-navy p-6 md:p-12 font-sans">
      <div className="max-w-5xl mx-auto space-y-8">
        <button
          onClick={() => navigate('/')}
          className="text-sm text-slateblue hover:text-navy transition-colors flex items-center space-x-2 font-medium"
        >
          <span>←</span> <span>Back to Catalog</span>
        </button>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-start">
          <ThreeDViewerFallback productName={dummyProduct.name} />

          <div className="space-y-6">
            <div>
              <span className="text-xs uppercase tracking-widest text-slateblue font-bold">
                {dummyProduct.category}
              </span>
              <h1 className="text-3xl font-extrabold text-navy mt-1">{dummyProduct.name}</h1>
              <p className="text-2xl font-bold text-navy mt-3">
                {dummyProduct.currency} {dummyProduct.price}
              </p>
            </div>

            <div className="p-4 bg-cream/40 border border-lightgray rounded-xl space-y-2 text-xs text-slateblue">
              <div className="flex justify-between">
                <span>SKU:</span>
                <span className="font-mono text-navy">{dummyProduct.id}</span>
              </div>
            </div>

            <Button
              variant="primary"
              size="lg"
              className="w-full"
              onClick={() => {
                addToCart({
                  productId: dummyProduct.id,
                  name: dummyProduct.name,
                  quantity: 1,
                  price: dummyProduct.price,
                });
                navigate('/');
              }}
            >
              Add to Cart
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ProductDetail;
