import React from 'react';
import { useRootStore } from '@/slicers/root_store';

const categories = [
  'All Abayas',
  'Velvet & Silk',
  'Gold Embroidery',
  'Casual Luxe',
  'Bridal Edition',
];

export const FilterSidebar: React.FC = () => {
  const activeFilter = useRootStore((s) => s.activeFilter);
  const setActiveFilter = useRootStore((s) => s.setActiveFilter);
  const searchQuery = useRootStore((s) => s.searchQuery);
  const setSearchQuery = useRootStore((s) => s.setSearchQuery);

  return (
    <aside className="w-full md:w-64 space-y-6 bg-white border border-lightgray p-6 rounded-2xl h-fit shadow-sm">
      <div>
        <h3 className="text-xs font-bold uppercase tracking-widest text-slateblue mb-3">
          Search Collections
        </h3>
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search Ajrah Noor..."
          className="w-full px-3.5 py-2.5 bg-white border border-lightgray rounded-xl text-navy placeholder-slateblue/50 focus:outline-none focus:border-navy text-xs font-medium"
        />
      </div>

      <div>
        <h3 className="text-xs font-bold uppercase tracking-widest text-slateblue mb-3">
          Abaya Categories
        </h3>
        <div className="space-y-1">
          {categories.map((cat) => (
            <button
              key={cat}
              onClick={() => setActiveFilter(cat)}
              className={`w-full text-left px-3.5 py-2.5 rounded-xl text-xs transition-all ${
                activeFilter === cat
                  ? 'bg-navy text-white font-bold shadow-sm'
                  : 'text-slateblue hover:text-navy hover:bg-cream/60 font-medium'
              }`}
            >
              {cat}
            </button>
          ))}
        </div>
      </div>

      <div className="pt-4 border-t border-lightgray">
        <div className="p-3.5 bg-cream/40 rounded-xl border border-lightgray/60 space-y-1 text-[11px]">
          <p className="font-bold text-navy">Ajrah Noor Bespoke Atelier</p>
          <p className="text-slateblue leading-relaxed">
            Custom sizing & haute couture tailors available for every order.
          </p>
        </div>
      </div>
    </aside>
  );
};

export default FilterSidebar;
