import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { useProducts } from '../../features/catalog/hooks/useProducts';
import { ProductCard } from '../../features/catalog/components/ProductCard';
import { CartSummary } from '../../features/cart_checkout/components/CartSummary';
import { useCheckout } from '../../features/cart_checkout/hooks/useCheckout';
import { useAuthStore } from '../../features/auth/store/authSlice';
import { ProductDto } from '../../shared/types/contracts';

export function StorefrontPage() {
  const { data: products, isLoading, error } = useProducts();
  const [cart, setCart] = useState<{ product: ProductDto; quantity: number }[]>([]);
  const checkoutMutation = useCheckout();
  const { token, clearAuth, role } = useAuthStore();

  const handleAddToCart = (product: ProductDto) => {
    setCart((prev) => {
      const existing = prev.find((item) => item.product.id === product.id);
      if (existing) {
        return prev.map((item) =>
          item.product.id === product.id ? { ...item, quantity: item.quantity + 1 } : item
        );
      }
      return [...prev, { product, quantity: 1 }];
    });
  };

  const handleRemoveFromCart = (id: string) => {
    setCart((prev) => prev.filter((item) => item.product.id !== id));
  };

  const handleCheckout = () => {
    const items = cart.map((item) => ({
      variant_id: item.product.id,
      quantity: item.quantity,
    }));
    
    checkoutMutation.mutate(
      {
        currency: 'USD',
        shipping_address: {
          recipient_name: 'Jane Doe',
          phone: '555-0199',
          street_line1: '123 Main St',
          city: 'Metropolis',
          postal_code: '10001',
          country: 'US',
        },
        items,
      },
      {
        onSuccess: (data) => {
          alert(`Checkout successful! ${data.message}`);
          setCart([]);
        },
        onError: (err: any) => {
          alert(`Checkout failed: ${err.response?.data?.message || err.message}`);
        },
      }
    );
  };

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col">
      {/* Premium Header */}
      <header className="border-b border-slate-900 bg-slate-950/80 backdrop-blur-md sticky top-0 z-50 px-6 py-4 flex justify-between items-center">
        <div className="flex items-center space-x-3">
          <span className="font-extrabold text-xl tracking-wider bg-clip-text bg-gradient-to-r from-indigo-400 to-purple-400">
            HAS.LTD
          </span>
          <span className="bg-slate-900 border border-slate-800 text-[10px] text-slate-400 px-2 py-0.5 rounded font-mono uppercase">
            Storefront
          </span>
        </div>

        <nav className="flex items-center space-x-6">
          {token ? (
            <>
              {(role === 'SUPER_ADMIN' || role === 'MODERATOR') && (
                <Link to="/dashboard" className="text-slate-300 hover:text-white font-medium text-sm transition-colors">
                  Dashboard
                </Link>
              )}
              <button 
                onClick={clearAuth} 
                className="text-slate-400 hover:text-white font-medium text-sm transition-colors"
              >
                Logout
              </button>
            </>
          ) : (
            <Link to="/login" className="text-indigo-400 hover:text-indigo-300 font-semibold text-sm transition-colors">
              Sign In
            </Link>
          )}
        </nav>
      </header>

      {/* Main Grid Layout */}
      <main className="flex-1 max-w-7xl w-full mx-auto px-6 py-10 grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Products Section */}
        <section className="lg:col-span-3 space-y-6">
          <h2 className="text-2xl font-bold tracking-tight text-white">Our Catalog</h2>
          
          {isLoading && (
            <div className="flex justify-center items-center py-20">
              <div className="animate-spin rounded-full h-10 w-10 border-t-2 border-indigo-500" />
            </div>
          )}

          {error && (
            <div className="bg-red-950/20 border border-red-900/50 text-red-400 p-4 rounded-xl text-sm">
              Failed to load products. Please check if backend is running.
            </div>
          )}

          {products && (
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
              {products.map((product) => (
                <ProductCard 
                  key={product.id} 
                  product={product} 
                  onAddToCart={handleAddToCart} 
                />
              ))}
            </div>
          )}
        </section>

        {/* Sidebar Cart Section */}
        <aside className="lg:col-span-1">
          <div className="sticky top-24">
            <CartSummary 
              items={cart} 
              onRemove={handleRemoveFromCart} 
              onCheckout={handleCheckout} 
              isSubmitting={checkoutMutation.isPending}
            />
          </div>
        </aside>
      </main>
    </div>
  );
}
