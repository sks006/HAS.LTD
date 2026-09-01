import type { StateCreator } from 'zustand';
import type { InventoryItemDto } from '@/shared/types/contracts';

export interface InventorySlice {
  inventory: InventoryItemDto[];
  occVersion: number;
  setInventory: (inventory: InventoryItemDto[]) => void;
  updateInventoryItem: (sku: string, patch: Partial<InventoryItemDto>) => void;
  removeInventoryItem: (sku: string) => void;
}

const initialInventory: InventoryItemDto[] = [
  {
    id: 'inv-1',
    name: 'Ajrah Noor Silk Organza Abaya',
    category: 'Velvet & Silk',
    sku: 'AN-SLK-001',
    incoming: 20,
    stock: 45,
    price: 1850,
    currency: 'SAR',
    status: 'In stock',
  },
  {
    id: 'inv-2',
    name: 'Noor Gold Embroidered Royal Abaya',
    category: 'Gold Embroidery',
    sku: 'AN-GLD-002',
    incoming: 5,
    stock: 4,
    price: 2400,
    currency: 'SAR',
    status: 'Low stock',
  },
  {
    id: 'inv-3',
    name: 'Ajrah Atelier Chiffon Layered Abaya',
    category: 'Casual Luxe',
    sku: 'AN-CHF-003',
    incoming: 0,
    stock: 0,
    price: 1450,
    currency: 'SAR',
    status: 'Out of stock',
  },
  {
    id: 'inv-4',
    name: 'Midnight Velvet Crystal Abaya',
    category: 'Bridal Edition',
    sku: 'AN-BRD-004',
    incoming: 12,
    stock: 18,
    price: 3200,
    currency: 'SAR',
    status: 'In stock',
  },
];

export const createInventorySlice: StateCreator<InventorySlice, [], [], InventorySlice> = (set) => ({
  inventory: initialInventory,
  occVersion: 1,
  setInventory: (inventory) => set((state) => ({ inventory, occVersion: state.occVersion + 1 })),
  updateInventoryItem: (sku, patch) =>
    set((state) => ({
      inventory: state.inventory.map((item) => (item.sku === sku ? { ...item, ...patch } : item)),
      occVersion: state.occVersion + 1,
    })),
  removeInventoryItem: (sku) =>
    set((state) => ({
      inventory: state.inventory.filter((item) => item.sku !== sku),
      occVersion: state.occVersion + 1,
    })),
});
