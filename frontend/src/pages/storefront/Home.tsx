import React, { useState } from 'react';
import { ProductGrid } from '@/features/catalog/components/ProductGrid';
import { FilterSidebar } from '@/features/catalog/components/FilterSidebar';
import { CartDrawer } from '@/features/checkout/components/CartDrawer';
import { useRootStore } from '@/slicers/root_store';
import { useNavigate } from 'react-router-dom';
import Button from '@/shared/ui/Button';

export const Home: React.FC = () => {
  const navigate = useNavigate();
  const [cartOpen, setCartOpen] = useState(false);
  const items = useRootStore((s) => s.items);

  const cartCount = items.reduce((sum, item) => sum + item.quantity, 0);

  return (
    <div className="min-h-screen bg-white text-navy flex flex-col font-sans">
      {/* Header Bar */}
      <header className="border-b border-lightgray bg-white sticky top-0 z-40 shadow-sm">
        <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center space-x-3 cursor-pointer" onClick={() => navigate('/')}>
            <div className="w-10 h-10 rounded-2xl bg-navy flex items-center justify-center font-serif text-white text-xl font-black shadow-sm">
              AN
            </div>
            <div>
              <span className="text-xl font-extrabold tracking-tight text-navy block font-serif leading-none">
                Ajrah Noor
              </span>
              <span className="text-[9px] uppercase tracking-[0.25em] text-taupe font-bold block mt-1">
                Luxury Abaya & Couture
              </span>
            </div>
          </div>

          <div className="flex items-center space-x-4">
            <Button variant="outline" size="sm" onClick={() => navigate('/login')} className="rounded-xl">
              Portal Access
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => setCartOpen(true)}
              className="rounded-xl relative"
            >
              Bag ({cartCount})
            </Button>
          </div>
        </div>
      </header>

      {/* Hero Banner */}
      <section className="bg-cream/40 border-b border-lightgray/60 py-12 px-6">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-6">
          <div className="space-y-3 text-center md:text-left">
            <span className="text-xs font-bold uppercase tracking-[0.3em] text-taupe">
              Haute Couture Edit
            </span>
            <h1 className="text-3xl sm:text-4xl font-extrabold text-navy font-serif tracking-tight">
              Elegance Defined. Timeless Luxury Abayas.
            </h1>
            <p className="text-sm text-slateblue max-w-xl">
              Discover the new Ajrah Noor collection featuring silk organza, velvet trims, and hand-stitched gold embroidery.
            </p>
          </div>
          <Button variant="primary" size="lg" className="rounded-2xl shrink-0" onClick={() => setCartOpen(true)}>
            Explore Collection
          </Button>
        </div>
      </section>

      {/* Main Content Layout */}
      <div className="max-w-7xl mx-auto px-6 py-10 flex-1 flex flex-col md:flex-row gap-8 w-full bg-white">
        <FilterSidebar />
        <main className="flex-1 space-y-6">
          <div className="flex justify-between items-center border-b border-lightgray pb-4">
            <h2 className="text-2xl font-bold text-navy font-serif">Curated Atelier Collection</h2>
            <span className="text-xs text-slateblue font-medium">Fast Shipping Across GCC</span>
          </div>
          <ProductGrid />
        </main>
      </div>

      {/* Footer */}
      <footer className="border-t border-lightgray bg-white py-8 px-6 mt-12">
        <div className="max-w-7xl mx-auto flex flex-col sm:flex-row justify-between items-center gap-4 text-xs text-slateblue">
          <p>© {new Date().getFullYear()} Ajrah Noor Couture. All rights reserved.</p>
          <div className="flex space-x-6">
            <span className="hover:text-navy cursor-pointer" onClick={() => navigate('/dashboard/admin')}>
              Admin Panel
            </span>
            <span className="hover:text-navy cursor-pointer" onClick={() => navigate('/dashboard/moderator')}>
              Moderator Audit
            </span>
          </div>
        </div>
      </footer>

      <CartDrawer open={cartOpen} onClose={() => setCartOpen(false)} />
    </div>
  );
};

export default Home;
