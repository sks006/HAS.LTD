import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { OrderRecord, OrderState } from '@/shared/types/contracts';

// Payload types
export interface AddOrderPayload {
  customerName: string;
  phone?: string;
  shippingAddress?: {
    recipient_name: string;
    phone: string;
    street_line1: string;
    city: string;
    state?: string;
    postal_code: string;
    country: string;
    delivery_instructions?: string;
  };
  productName: string;
  sku: string;
  paymentMethod: 'Cash on Delivery' | 'Paid via PayPal' | 'Credit Card' | 'Apple Pay';
  amount: number;
  currency?: string;
  status: OrderState;
}

export interface UpdateOrderStatusPayload {
  id: string;
  status: OrderState;
}

// Initial state starts empty - live customer orders from website & backend API populate this
const INITIAL_ORDERS: OrderRecord[] = [];

// Async thunks
export const fetchOrders = createAsyncThunk<OrderRecord[]>(
  'orders/fetchOrders',
  async (_, { rejectWithValue }) => {
    try {
      return await apiFetch<OrderRecord[]>(ENDPOINTS.ORDERS);
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const createNewOrder = createAsyncThunk<OrderRecord, AddOrderPayload>(
  'orders/createNewOrder',
  async (payload, { rejectWithValue }) => {
    try {
      return await apiFetch<OrderRecord>(ENDPOINTS.ORDERS, {
        method: 'POST',
        body: payload,
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const updateOrderStatusThunk = createAsyncThunk<
  { id: string; status: OrderState },
  { id: string; status: OrderState; version?: number }
>(
  'orders/updateOrderStatusThunk',
  async ({ id, status, version = 1 }, { rejectWithValue }) => {
    try {
      await apiFetch(`${ENDPOINTS.ORDERS}/${id}/status`, {
        method: 'PUT',
        body: { state: status, version },
      });
      return { id, status };
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const deleteOrderThunk = createAsyncThunk<string, string>(
  'orders/deleteOrderThunk',
  async (id, { rejectWithValue }) => {
    try {
      await apiFetch(`${ENDPOINTS.ORDERS}/${id}`, {
        method: 'DELETE',
      });
      return id;
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export interface OrdersState {
  orders: OrderRecord[];
  activeFilter: 'All Order' | 'Active' | 'Delivered' | 'Returned';
  searchQuery: string;
  sortBy: 'Latest' | 'Amount (High-Low)' | 'Amount (Low-High)' | 'Customer Name';
  loading: boolean;
  error: string | null;
}

const initialState: OrdersState = {
  orders: INITIAL_ORDERS,
  activeFilter: 'Active', // Active is selected by default as in screenshot
  searchQuery: '',
  sortBy: 'Latest',
  loading: false,
  error: null,
};

const ordersSlice = createSlice({
  name: 'orders',
  initialState,
  reducers: {
    setOrders: (state, action: PayloadAction<OrderRecord[]>) => {
      state.orders = action.payload;
    },
    setActiveOrderFilter: (
      state,
      action: PayloadAction<'All Order' | 'Active' | 'Delivered' | 'Returned'>
    ) => {
      state.activeFilter = action.payload;
    },
    setOrderSearchQuery: (state, action: PayloadAction<string>) => {
      state.searchQuery = action.payload;
    },
    setOrderSortBy: (
      state,
      action: PayloadAction<'Latest' | 'Amount (High-Low)' | 'Amount (Low-High)' | 'Customer Name'>
    ) => {
      state.sortBy = action.payload;
    },
    addOrder: (state, action: PayloadAction<AddOrderPayload & { id?: string }>) => {
      const orderId = action.payload.id || `ord-${Math.floor(10000 + Math.random() * 90000)}`;
      const newRecord: OrderRecord = {
        id: orderId,
        order_number: `ID: ${orderId.slice(0, 8)}`,
        customer_name: action.payload.customerName,
        product_name: action.payload.productName,
        sku: action.payload.sku || 'SK-45',
        payment_method: action.payload.paymentMethod,
        amount: action.payload.amount,
        currency: action.payload.currency || '$',
        status: action.payload.status,
        created_at: new Date().toISOString(),
        version: 1,
        idempotency_key: `idemp-${orderId}-${Date.now()}`,
        shipping_address: action.payload.shippingAddress || {
          recipient_name: action.payload.customerName,
          phone: action.payload.phone || '',
          street_line1: '',
          city: '',
          postal_code: '',
          country: 'Saudi Arabia',
        },
      };
  
      state.orders.unshift(newRecord);
    },
    updateOrderStatus: (state, action: PayloadAction<UpdateOrderStatusPayload>) => {
      const order = state.orders.find((o) => o.id === action.payload.id);
      if (order) {
        order.status = action.payload.status;
        order.updated_at = new Date().toISOString();
        if (order.version !== undefined) {
          order.version += 1;
        }
      }
    },
    removeOrder: (state, action: PayloadAction<string>) => {
      state.orders = state.orders.filter((o) => o.id !== action.payload);
    },
    removeMultipleOrders: (state, action: PayloadAction<string[]>) => {
      state.orders = state.orders.filter((o) => !action.payload.includes(o.id));
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchOrders.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchOrders.fulfilled, (state, action) => {
        state.loading = false;
        if (Array.isArray(action.payload) && action.payload.length > 0) {
          const apiIds = new Set(action.payload.map((o) => o.id));
          const localOnly = state.orders.filter((o) => !apiIds.has(o.id));
          state.orders = [...action.payload, ...localOnly];
        }
      })
      .addCase(fetchOrders.rejected, (state, action) => {
        state.loading = false;
        // Keep initial mock orders on API failure
      })
      .addCase(createNewOrder.fulfilled, (state, action) => {
        state.orders.unshift(action.payload);
      })
      .addCase(updateOrderStatusThunk.fulfilled, (state, action) => {
        const index = state.orders.findIndex((o) => o.id === action.payload.id);
        if (index !== -1) {
          state.orders[index].status = action.payload.status;
        }
      })
      .addCase(deleteOrderThunk.fulfilled, (state, action) => {
        state.orders = state.orders.filter((o) => o.id !== action.payload);
      });
  },
});

export const {
  setOrders,
  setActiveOrderFilter,
  setOrderSearchQuery,
  setOrderSortBy,
  addOrder,
  updateOrderStatus,
  removeOrder,
  removeMultipleOrders,
} = ordersSlice.actions;

export default ordersSlice.reducer;
