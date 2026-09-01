import { create } from 'zustand';
import { AuthSlice, createAuthSlice } from './auth_slicer/auth_slice';
import { CatalogSlice, createCatalogSlice } from './catalog_slicer/catalog_slice';
import { InventorySlice, createInventorySlice } from './inventory_slicer/inventory_slice';
import { CartSlice, createCartSlice } from './cart_slicer/cart_slice';

export type RootStore = AuthSlice & CatalogSlice & InventorySlice & CartSlice;

export const useRootStore = create<RootStore>()((...a) => ({
  ...createAuthSlice(...a),
  ...createCatalogSlice(...a),
  ...createInventorySlice(...a),
  ...createCartSlice(...a),
}));

// Backwards-compatibility alias for components using useStore
export const useStore = useRootStore;
