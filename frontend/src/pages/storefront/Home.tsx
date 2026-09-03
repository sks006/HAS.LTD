import React, { useState } from 'react';
import { ProductGrid } from '@/features/catalog/components/ProductGrid';
import { FilterSidebar } from '@/features/catalog/components/FilterSidebar';
import { CartDrawer } from '@/features/checkout/components/CartDrawer';
import { useRootStore } from '@/slicers/root_store';
import { useProducts } from '@/features/catalog/hooks/useProducts';
import { useNavigate } from 'react-router-dom';
import {
  Search,
  ChevronDown,
  User,
  ShoppingBag,
  ArrowRight,
  Sparkles,
  Shield,
  LayoutDashboard,
} from 'lucide-react';

export const Home: React.FC = () => {
  useProducts(); // Initialize product data from Axum backend
  const navigate = useNavigate();
  const [cartOpen, setCartOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  
  const items = useRootStore((s) => s.items);
  const searchQuery = useRootStore((s) => s.searchQuery);
  const setSearchQuery = useRootStore((s) => s.setSearchQuery);

  const cartCount = items.reduce((sum, item) => sum + item.quantity, 0);

  return (
    <div className="min-h-screen bg-white text-neutral-900 flex flex-col font-sans select-none">
      {/* Top Header Bar matching User Screenshot #2 */}
      <header className="border-b border-neutral-200/80 bg-white sticky top-0 z-40">
        {/* Upper Utility Header */}
        <div className="max-w-7xl mx-auto px-6 py-3.5 flex items-center justify-between">
          {/* Left: Search & Currency */}
          <div className="flex items-center space-x-6">
            <div className="relative">
              <button
                onClick={() => setSearchOpen(!searchOpen)}
                className="flex items-center space-x-1.5 text-xs font-semibold text-neutral-600 hover:text-neutral-900 transition-colors cursor-pointer"
              >
                <Search size={14} className="text-neutral-500" />
                <span>Search</span>
              </button>

              {searchOpen && (
                <div className="absolute left-0 top-8 w-64 bg-white p-2 border border-neutral-200 rounded-xl shadow-lg z-50 animate-in fade-in">
                  <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Search abayas, fabrics..."
                    className="w-full px-3 py-1.5 text-xs bg-neutral-50 border border-neutral-200 rounded-lg focus:outline-none focus:border-neutral-900"
                    autoFocus
                  />
                </div>
              )}
            </div>

            <div className="flex items-center space-x-1 text-xs font-semibold text-neutral-600 cursor-pointer">
              <span>SAR ر.س</span>
              <ChevronDown size={12} className="text-neutral-400" />
            </div>
          </div>

          {/* Center Brand Title */}
          <div
            onClick={() => navigate('/')}
            className="cursor-pointer text-center group"
          >
            <h1 className="font-serif text-2xl md:text-3xl font-extrabold tracking-tight text-neutral-900 group-hover:text-amber-800 transition-colors">
              Ajrah Noor
            </h1>
          </div>

          {/* Right Controls: User Profile + Portal Access Button + Bag */}
          <div className="flex items-center space-x-3">
            {/* PROMINENT PORTAL ACCESS BUTTON */}
            <button
              onClick={() => navigate('/dashboard/orders')}
              className="bg-[#1c1917] hover:bg-neutral-800 text-white text-xs font-bold px-3.5 py-1.5 rounded-full flex items-center space-x-1.5 shadow-xs transition-all cursor-pointer border border-neutral-700"
            >
              <LayoutDashboard size={13} className="text-amber-300 shrink-0" />
              <span className="uppercase tracking-wider text-[11px]">Portal Access</span>
            </button>

            {/* User Icon */}
            <button
              onClick={() => navigate('/login')}
              title="Staff & Customer Login"
              className="p-1.5 rounded-full text-neutral-600 hover:text-neutral-900 hover:bg-neutral-100 transition-colors cursor-pointer"
            >
              <User size={18} />
            </button>

            {/* Bag Icon */}
            <button
              onClick={() => setCartOpen(true)}
              className="relative p-1.5 text-neutral-800 hover:text-neutral-900 transition-colors cursor-pointer"
              title="View Bag"
            >
              <ShoppingBag size={20} />
              {cartCount > 0 && (
                <span className="absolute -top-1 -right-1 bg-amber-600 text-white text-[10px] font-bold w-4 h-4 rounded-full flex items-center justify-center">
                  {cartCount}
                </span>
              )}
            </button>
          </div>
        </div>

        {/* Center Main Category Navigation */}
        <nav className="border-t border-neutral-100 py-2.5">
          <div className="max-w-7xl mx-auto px-6 flex justify-center space-x-8 text-[11px] font-bold uppercase tracking-[0.2em] text-neutral-600">
            <span className="hover:text-neutral-900 cursor-pointer transition-colors">
              NEW IN
            </span>
            <span className="hover:text-neutral-900 cursor-pointer transition-colors text-neutral-900 border-b border-neutral-900 pb-0.5">
              ABAYAS
            </span>
            <span className="hover:text-neutral-900 cursor-pointer transition-colors">
              OUR STORY
            </span>
            <span className="hover:text-neutral-900 cursor-pointer transition-colors">
              JOURNAL
            </span>
          </div>
        </nav>
      </header>

      {/* Hero Section matching Screenshot #2 */}
      <section className="bg-white border-b border-neutral-100 py-12 px-6">
        <div className="max-w-7xl mx-auto grid grid-cols-1 md:grid-cols-2 gap-12 items-center">
          <div className="space-y-6">
            <div className="flex items-center space-x-2 text-[10px] uppercase font-bold tracking-[0.25em] text-amber-700">
              <Sparkles size={13} />
              <span>THE NEW MODEST WARDROBE</span>
            </div>

            <div className="space-y-3">
              <span className="font-serif italic text-xl text-neutral-600 block">
                Quietly distinctive.
              </span>
              <h2 className="text-3xl sm:text-4xl font-serif text-neutral-900 font-normal leading-tight">
                Thoughtful abayas for the way you move through the world. Made to be lived in, remembered, and worn your way.
              </h2>
            </div>

            <button
              onClick={() => {
                const el = document.getElementById('collection-grid');
                el?.scrollIntoView({ behavior: 'smooth' });
              }}
              className="bg-[#1c1917] hover:bg-neutral-800 text-white text-xs font-bold tracking-widest uppercase px-6 py-3.5 rounded-none inline-flex items-center space-x-3 transition-all cursor-pointer"
            >
              <span>EXPLORE THE COLLECTION</span>
              <ArrowRight size={14} />
            </button>
          </div>

          <div className="relative aspect-[1.1/1] bg-neutral-100 rounded-2xl overflow-hidden shadow-sm">
            <img
              src="https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?auto=format&fit=crop&q=80&w=1200"
              alt="Ajrah Noor Hero Abaya"
              className="w-full h-full object-cover"
            />
            <div className="absolute top-4 right-4 bg-white/90 backdrop-blur-xs p-3 border border-amber-200 rounded-xl text-center shadow-xs">
              <span className="block font-serif text-xs font-bold text-neutral-900">A/N</span>
              <span className="text-[9px] uppercase font-mono tracking-widest text-neutral-500 block">VOL. ONE</span>
            </div>
          </div>
        </div>
      </section>

      {/* Main Catalog Grid */}
      <div id="collection-grid" className="max-w-7xl mx-auto px-6 py-12 flex-1 flex flex-col md:flex-row gap-8 w-full bg-white">
        <FilterSidebar />
        <main className="flex-1 space-y-6">
          <div className="flex justify-between items-end border-b border-neutral-200 pb-4">
            <div>
              <span className="text-[10px] uppercase font-bold tracking-[0.25em] text-neutral-400">
                HAUTE COUTURE EDIT
              </span>
              <h2 className="text-2xl font-serif text-neutral-900 font-normal">
                Curated Abaya Collection
              </h2>
            </div>
            <span className="text-xs text-neutral-500 font-medium">
              Worldwide Express Shipping
            </span>
          </div>
          <ProductGrid />
        </main>
      </div>

      {/* Footer matching Screenshot #3 */}
      <footer className="bg-[#1c1917] text-white py-12 px-6 mt-16">
        <div className="max-w-7xl mx-auto space-y-12">
          {/* Newsletter Box */}
          <div className="flex flex-col md:flex-row items-center justify-between gap-6 border-b border-neutral-800 pb-12">
            <div className="space-y-1 text-center md:text-left">
              <span className="text-[10px] uppercase font-bold tracking-[0.3em] text-amber-400">
                STAY CLOSE
              </span>
              <h3 className="font-serif text-2xl text-white italic">
                A little more Ajrah Noor, in your inbox.
              </h3>
            </div>

            <div className="flex items-center bg-neutral-900 border border-neutral-700 rounded-xl p-1.5 w-full max-w-md">
              <input
                type="email"
                placeholder="you@example.com"
                className="bg-transparent text-white px-3 py-1.5 text-xs flex-1 focus:outline-none placeholder-neutral-500"
              />
              <button className="bg-amber-600 hover:bg-amber-500 text-white p-2 rounded-lg cursor-pointer">
                <ArrowRight size={14} />
              </button>
            </div>
          </div>

          {/* Footer Nav Bar matching Screenshot #3 */}
          <div className="flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-neutral-400 font-medium">
            <div className="font-serif text-xl font-bold text-white">
              Ajrah Noor
            </div>

            <div className="flex items-center space-x-6">
              <span className="hover:text-white cursor-pointer transition-colors">Shop</span>
              <span className="hover:text-white cursor-pointer transition-colors">About</span>
              <span className="hover:text-white cursor-pointer transition-colors">Message us</span>
              
              {/* STAFF SIGN IN / PORTAL ACCESS FOOTER LINK */}
              <button
                onClick={() => navigate('/dashboard/orders')}
                className="hover:text-amber-400 text-amber-300 font-bold cursor-pointer transition-colors flex items-center space-x-1"
              >
                <Shield size={13} />
                <span>Portal Access (Staff Sign In)</span>
              </button>
            </div>

            <div>
              © {new Date().getFullYear()} Ajrah Noor. All rights reserved.
            </div>
          </div>
        </div>
      </footer>

      <CartDrawer open={cartOpen} onClose={() => setCartOpen(false)} />
    </div>
  );
};

export default Home;
