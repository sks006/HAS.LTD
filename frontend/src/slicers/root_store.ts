import { configureStore } from '@reduxjs/toolkit';
import { create } from 'zustand';
import authReducer, { setAuth, setRole, logout, AuthState } from './auth_slicer/auth_slice';
import productsReducer, {
  setProducts,
  removeProduct,
  setActiveFilter,
  setSearchQuery,
  setSelectedAsset2D3D,
  ProductsState,
} from './catalog_slicer/catalog_slice';
import cartReducer, {
  addToCart,
  removeFromCart,
  setItemQuantity,
  clearCart,
  CartItem,
  CartState,
} from './cart_slicer/cart_slice';
import inventoryReducer, {
  setInventory,
  addInventoryItem,
  updateInventoryItem,
  removeInventoryItem,
  InventoryState,
} from './inventory_slicer/inventory_slice';
import ordersReducer, { OrdersState } from './order_slicer/order_slice';

import type { AuthUser, ProductDto, InventoryItemDto } from '@/shared/types/contracts';
import type { Role } from '@/shared/types/roles';

// 1. Redux Toolkit Store Configuration
export const store = configureStore({
  reducer: {
    auth: authReducer,
    products: productsReducer,
    cart: cartReducer,
    inventory: inventoryReducer,
    orders: ordersReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

// 2. Zustand Store Interface for Backward Compatibility with existing hooks
export interface LegacyAuthSlice {
  user: AuthUser | null;
  token: string | null;
  setAuth: (user: AuthUser | null, token: string | null) => void;
  setRole: (role: Role) => void;
  logout: () => void;
}

export interface LegacyCatalogSlice {
  products: ProductDto[];
  activeFilter: string;
  searchQuery: string;
  selectedAsset2D3D: string | null;
  setProducts: (products: ProductDto[]) => void;
  removeProduct: (id: string) => void;
  setActiveFilter: (filter: string) => void;
  setSearchQuery: (query: string) => void;
  setSelectedAsset2D3D: (asset: string | null) => void;
}

export interface LegacyCartSlice {
  items: CartItem[];
  addToCart: (item: CartItem) => void;
  removeFromCart: (productId: string) => void;
  updateQuantity: (productId: string, quantity: number) => void;
  clearCart: () => void;
}

export interface LegacyInventorySlice {
  inventory: InventoryItemDto[];
  occVersion: number;
  setInventory: (inventory: InventoryItemDto[]) => void;
  addInventoryItem: (item: Omit<InventoryItemDto, 'id'>) => void;
  updateInventoryItem: (sku: string, patch: Partial<InventoryItemDto>) => void;
  removeInventoryItem: (key: string) => void;
}

export type RootStore = LegacyAuthSlice & LegacyCatalogSlice & LegacyCartSlice & LegacyInventorySlice;

export const useRootStore = create<RootStore>()((set) => ({
  // Auth
  user: { id: 'demo-admin', email: 'admin@lamina.sa', role: 'SUPER_ADMIN' },
  token: null,
  setAuth: (user, token) => set({ user, token }),
  setRole: (role) =>
    set((state) => ({
      user: state.user
        ? { ...state.user, role }
        : { id: 'demo-user', email: 'user@lamina.sa', role },
    })),
  logout: () => set({ user: null, token: null }),

  // Catalog
  products: [],
  activeFilter: 'All Abayas',
  searchQuery: '',
  selectedAsset2D3D: null,
  setProducts: (products) => set({ products }),
  removeProduct: (id) => set((state) => ({ products: state.products.filter((p) => p.id !== id) })),
  setActiveFilter: (activeFilter) => set({ activeFilter }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setSelectedAsset2D3D: (selectedAsset2D3D) => set({ selectedAsset2D3D }),

  // Cart
  items: [],
  addToCart: (item) =>
    set((state) => {
      const existing = state.items.find((i) => i.productId === item.productId);
      if (existing) {
        return {
          items: state.items.map((i) =>
            i.productId === item.productId ? { ...i, quantity: i.quantity + item.quantity } : i
          ),
        };
      }
      return { items: [...state.items, item] };
    }),
  removeFromCart: (productId) =>
    set((state) => ({ items: state.items.filter((i) => i.productId !== productId) })),
  updateQuantity: (productId, quantity) =>
    set((state) => ({
      items: state.items.map((i) =>
        i.productId === productId ? { ...i, quantity: Math.max(1, quantity) } : i
      ),
    })),
  clearCart: () => set({ items: [] }),

  // Inventory
  inventory: [],
  occVersion: 1,
  setInventory: (inventory) => set((state) => ({ inventory, occVersion: state.occVersion + 1 })),
  addInventoryItem: (item) =>
    set((state) => ({
      inventory: [{ id: `inv-${Date.now()}`, ...item }, ...state.inventory],
      occVersion: state.occVersion + 1,
    })),
  updateInventoryItem: (key, patch) =>
    set((state) => ({
      inventory: state.inventory.map((item) => (item.sku === key || item.id === key ? { ...item, ...patch } : item)),
      occVersion: state.occVersion + 1,
    })),
  removeInventoryItem: (key) =>
    set((state) => ({
      inventory: state.inventory.filter((item) => item.sku !== key && item.id !== key),
      occVersion: state.occVersion + 1,
    })),
}));

export const useStore = useRootStore;
