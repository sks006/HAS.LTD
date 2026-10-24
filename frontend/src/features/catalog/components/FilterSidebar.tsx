import React from 'react';
import { useRootStore } from '@/slicers/root_store';
import { Card, CardHeader, CardTitle, CardContent } from '@/shared/ui/card';
import { Input } from '@/shared/ui/input';
import { Badge } from '@/shared/ui/badge';
import { Button } from '@/shared/ui/button';
import { Separator } from '@/shared/ui/separator';
import { Search, Sparkles, Filter, X } from 'lucide-react';

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
  const products = useRootStore((s) => s.products);

  const getCategoryCount = (cat: string) => {
    if (cat === 'All Abayas' || cat === 'All pieces') return products.length;
    return products.filter((p) => p.category === cat).length;
  };

  return (
    <aside className="w-full md:w-64 space-y-4">
      <Card className="border-neutral-200 shadow-2xs bg-white">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="font-serif text-sm font-bold text-neutral-900 flex items-center space-x-1.5">
              <Filter size={14} className="text-amber-800" />
              <span>Catalog Filters</span>
            </CardTitle>
            {(activeFilter !== 'All Abayas' || searchQuery) && (
              <button
                type="button"
                onClick={() => {
                  setActiveFilter('All Abayas');
                  setSearchQuery('');
                }}
                className="text-[10px] text-amber-800 hover:text-neutral-900 font-mono flex items-center space-x-0.5"
                title="Reset filters"
              >
                <X size={12} />
                <span>Reset</span>
              </button>
            )}
          </div>
        </CardHeader>

        <CardContent className="space-y-5">
          {/* Search Box */}
          <div className="space-y-1.5">
            <label className="text-[10px] font-bold uppercase tracking-wider text-neutral-500 block">
              Search by Keyword
            </label>
            <div className="relative">
              <Search className="absolute left-3 top-2.5 h-3.5 w-3.5 text-neutral-400" />
              <Input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Silk, velvet, gold..."
                className="pl-8 text-xs h-9 bg-neutral-50/70 border-neutral-200"
              />
            </div>
          </div>

          <Separator />

          {/* Categories */}
          <div className="space-y-1.5">
            <label className="text-[10px] font-bold uppercase tracking-wider text-neutral-500 block">
              Collections & Fabrics
            </label>
            <div className="space-y-1">
              {categories.map((cat) => {
                const count = getCategoryCount(cat);
                const isSelected = activeFilter === cat || (cat === 'All Abayas' && activeFilter === 'All pieces');

                return (
                  <button
                    key={cat}
                    type="button"
                    onClick={() => setActiveFilter(cat)}
                    className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs transition-all cursor-pointer ${
                      isSelected
                        ? 'bg-neutral-900 text-white font-semibold shadow-xs'
                        : 'text-neutral-600 hover:text-neutral-900 hover:bg-neutral-100/70 font-medium'
                    }`}
                  >
                    <span>{cat}</span>
                    <Badge
                      variant={isSelected ? 'outline' : 'secondary'}
                      className={`text-[9px] px-1.5 py-0 font-mono ${
                        isSelected ? 'border-neutral-600 text-neutral-200' : 'text-neutral-500'
                      }`}
                    >
                      {count}
                    </Badge>
                  </button>
                );
              })}
            </div>
          </div>

          <Separator />

          {/* Atelier Brand Callout */}
          <div className="p-3 bg-neutral-50 border border-neutral-200/80 rounded-xl space-y-1 text-[11px]">
            <div className="flex items-center space-x-1.5 text-amber-800 font-bold">
              <Sparkles size={12} />
              <span>Ajrah Noor Atelier</span>
            </div>
            <p className="text-neutral-500 text-[10px] leading-relaxed">
              Every garment is custom-tailored with complimentary GCC express delivery.
            </p>
          </div>
        </CardContent>
      </Card>
    </aside>
  );
};

export default FilterSidebar;
