import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { InventoryItemDto } from '@/shared/types/contracts';

// Types for payloads
export interface UpdateInventoryPayload {
  id: string;
  patch: Partial<InventoryItemDto>;
}

// Async thunks
export const fetchInventory = createAsyncThunk<InventoryItemDto[]>(
  'inventory/fetch',
  async (_, { rejectWithValue }) => {
    try {
      return await apiFetch<InventoryItemDto[]>(ENDPOINTS.INVENTORY);
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const updateProductInventory = createAsyncThunk<InventoryItemDto, UpdateInventoryPayload>(
  'inventory/updateProduct',
  async ({ id, patch }, { rejectWithValue }) => {
    try {
      return await apiFetch<InventoryItemDto>(`${ENDPOINTS.ADMIN_INVENTORY}/${id}`, {
        method: 'PUT',
        body: patch,
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const deleteInventoryProduct = createAsyncThunk<void, string>(
  'inventory/deleteProduct',
  async (id, { rejectWithValue }) => {
    try {
      await apiFetch<void>(`${ENDPOINTS.ADMIN_INVENTORY}/${id}`, {
        method: 'DELETE',
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

// Slice state
export interface InventoryState {
  inventory: InventoryItemDto[];
  occVersion: number;
  loading: boolean;
  error: string | null;
}

const initialState: InventoryState = {
  inventory: [],
  occVersion: 1,
  loading: false,
  error: null,
};

const inventorySlice = createSlice({
  name: 'inventory',
  initialState,
  reducers: {
    setInventory: (state, action: PayloadAction<InventoryItemDto[]>) => {
      state.inventory = action.payload;
      state.occVersion += 1;
    },
    addInventoryItem: (state, action: PayloadAction<Omit<InventoryItemDto, 'id'>>) => {
      state.inventory.unshift({ id: `inv-${Date.now()}`, ...action.payload });
      state.occVersion += 1;
    },
    updateInventoryItem: (
      state,
      action: PayloadAction<{ key: string; patch: Partial<InventoryItemDto> }>
    ) => {
      const { key, patch } = action.payload;
      const index = state.inventory.findIndex((item) => item.sku === key || item.id === key);
      if (index !== -1) {
        state.inventory[index] = { ...state.inventory[index], ...patch };
        state.occVersion += 1;
      }
    },
    removeInventoryItem: (state, action: PayloadAction<string>) => {
      state.inventory = state.inventory.filter((item) => item.sku !== action.payload && item.id !== action.payload);
      state.occVersion += 1;
    },
  },
  extraReducers: (builder) => {
    builder
      // Fetch Inventory
      .addCase(fetchInventory.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchInventory.fulfilled, (state, action) => {
        state.loading = false;
        state.inventory = action.payload;
        state.occVersion += 1;
      })
      .addCase(fetchInventory.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ? String(action.payload) : 'Failed to fetch inventory';
      })
      // Update Inventory
      .addCase(updateProductInventory.fulfilled, (state, action) => {
        const index = state.inventory.findIndex((item) => item.id === action.payload.id);
        if (index !== -1) {
          state.inventory[index] = action.payload;
        } else {
          state.inventory.push(action.payload);
        }
        state.occVersion += 1;
      })
      // Delete Inventory
      .addCase(deleteInventoryProduct.fulfilled, (state, action) => {
        state.inventory = state.inventory.filter((item) => item.id !== action.meta.arg);
        state.occVersion += 1;
      });
  },
});

export const {
  setInventory,
  addInventoryItem,
  updateInventoryItem,
  removeInventoryItem,
} = inventorySlice.actions;

export default inventorySlice.reducer;
