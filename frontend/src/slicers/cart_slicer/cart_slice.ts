import type { StateCreator } from 'zustand';

export interface CartItem {
  productId: string;
  name: string;
  quantity: number;
  price: number;
}

export interface CartSlice {
  items: CartItem[];
  addToCart: (item: CartItem) => void;
  removeFromCart: (productId: string) => void;
  updateQuantity: (productId: string, quantity: number) => void;
  clearCart: () => void;
}

export const createCartSlice: StateCreator<CartSlice, [], [], CartSlice> = (set) => ({
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
});
