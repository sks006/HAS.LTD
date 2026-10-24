import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { CheckoutPayload, CheckoutResponse } from './cart_fetch';

// Types for payloads & items
export interface CartItem {
  productId: string;
  name: string;
  quantity: number;
  price: number;
}

export interface SyncCartPayload {
  items: { variant_id: string; quantity_change: number }[];
}

export interface SubmitOrderPayload {
  checkout: CheckoutPayload;
  idempotencyKey: string;
}

// Async thunks
export const syncCart = createAsyncThunk<void, SyncCartPayload>(
  'cart/sync',
  async ({ items }, { rejectWithValue }) => {
    try {
      await apiFetch<void>(ENDPOINTS.CART_SYNC, {
        method: 'POST',
        body: items,
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const submitOrder = createAsyncThunk<CheckoutResponse, SubmitOrderPayload>(
  'cart/submitOrder',
  async ({ checkout, idempotencyKey }, { rejectWithValue }) => {
    try {
      return await apiFetch<CheckoutResponse>(ENDPOINTS.CHECKOUT, {
        method: 'POST',
        headers: { 'x-idempotency-key': idempotencyKey },
        body: checkout,
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

// Slice state
export interface CartState {
  items: CartItem[];
  lastOrder: CheckoutResponse | null;
  loading: boolean;
  error: string | null;
}

const initialState: CartState = {
  items: [],
  lastOrder: null,
  loading: false,
  error: null,
};

const cartSlice = createSlice({
  name: 'cart',
  initialState,
  reducers: {
    addToCart: (state, action: PayloadAction<CartItem>) => {
      const existing = state.items.find((i) => i.productId === action.payload.productId);
      if (existing) {
        existing.quantity += action.payload.quantity;
      } else {
        state.items.push(action.payload);
      }
    },
    removeFromCart: (state, action: PayloadAction<string>) => {
      state.items = state.items.filter((i) => i.productId !== action.payload);
    },
    updateQuantity: (state, action: PayloadAction<{ productId: string; quantity: number }>) => {
      const item = state.items.find((i) => i.productId === action.payload.productId);
      if (item) {
        item.quantity = Math.max(1, action.payload.quantity);
      }
    },
    setItemQuantity: (state, action: PayloadAction<{ productId: string; quantity: number }>) => {
      const item = state.items.find((i) => i.productId === action.payload.productId);
      if (item) {
        item.quantity = Math.max(1, action.payload.quantity);
      }
    },
    clearCart: (state) => {
      state.items = [];
    },
  },
  extraReducers: (builder) => {
    builder
      // Sync Cart
      .addCase(syncCart.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(syncCart.fulfilled, (state) => {
        state.loading = false;
      })
      .addCase(syncCart.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ? String(action.payload) : 'Failed to sync cart';
      })
      // Submit Order
      .addCase(submitOrder.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(submitOrder.fulfilled, (state, action) => {
        state.loading = false;
        state.lastOrder = action.payload;
        state.items = [];
      })
      .addCase(submitOrder.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ? String(action.payload) : 'Failed to submit order';
      });
  },
});

export const { addToCart, removeFromCart, updateQuantity, setItemQuantity, clearCart } = cartSlice.actions;

export default cartSlice.reducer;
