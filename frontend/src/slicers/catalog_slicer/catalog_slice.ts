import type { StateCreator } from 'zustand';
import type { ProductDto } from '@/shared/types/contracts';

export interface CatalogSlice {
  products: ProductDto[];
  activeFilter: string;
  searchQuery: string;
  selectedAsset2D3D: string | null;
  setProducts: (products: ProductDto[]) => void;
  setActiveFilter: (filter: string) => void;
  setSearchQuery: (query: string) => void;
  setSelectedAsset2D3D: (asset: string | null) => void;
}

const initialProducts: ProductDto[] = [
  {
    id: 'prod-1',
    name: 'Ajrah Noor Silk Organza Abaya',
    category: 'Velvet & Silk',
    price: 1850,
    currency: 'SAR',
    image: 'https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?auto=format&fit=crop&q=80&w=800',
    alt: 'Ajrah Noor Silk Organza Abaya',
    sale: true,
    rating: 4.9,
    reviews: 28,
  },
  {
    id: 'prod-2',
    name: 'Noor Gold Embroidered Royal Abaya',
    category: 'Gold Embroidery',
    price: 2400,
    currency: 'SAR',
    image: 'https://images.unsplash.com/photo-1515886657613-9f3515b0c78f?auto=format&fit=crop&q=80&w=800',
    alt: 'Noor Gold Embroidered Royal Abaya',
    sale: false,
    rating: 5.0,
    reviews: 42,
  },
  {
    id: 'prod-3',
    name: 'Ajrah Atelier Chiffon Layered Abaya',
    category: 'Casual Luxe',
    price: 1450,
    currency: 'SAR',
    image: 'https://images.unsplash.com/photo-1496747611176-843222e1e57c?auto=format&fit=crop&q=80&w=800',
    alt: 'Ajrah Atelier Chiffon Layered Abaya',
    sale: false,
    rating: 4.8,
    reviews: 19,
  },
  {
    id: 'prod-4',
    name: 'Midnight Velvet Crystal Abaya',
    category: 'Bridal Edition',
    price: 3200,
    currency: 'SAR',
    image: 'https://images.unsplash.com/photo-1539571696357-5a69c17a67c6?auto=format&fit=crop&q=80&w=800',
    alt: 'Midnight Velvet Crystal Abaya',
    sale: true,
    rating: 5.0,
    reviews: 64,
  },
  {
    id: 'prod-5',
    name: 'Ajrah Satin Overlay Kimono Abaya',
    category: 'Velvet & Silk',
    price: 1950,
    currency: 'SAR',
    image: 'https://images.unsplash.com/photo-1509631179647-0177331693ae?auto=format&fit=crop&q=80&w=800',
    alt: 'Ajrah Satin Overlay Kimono Abaya',
    sale: false,
    rating: 4.9,
    reviews: 31,
  },
];

export const createCatalogSlice: StateCreator<CatalogSlice, [], [], CatalogSlice> = (set) => ({
  products: initialProducts,
  activeFilter: 'All Abayas',
  searchQuery: '',
  selectedAsset2D3D: null,
  setProducts: (products) => set({ products }),
  setActiveFilter: (activeFilter) => set({ activeFilter }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setSelectedAsset2D3D: (selectedAsset2D3D) => set({ selectedAsset2D3D }),
});
