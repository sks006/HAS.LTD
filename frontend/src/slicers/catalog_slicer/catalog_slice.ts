import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import type { ProductDto } from '@/shared/types/contracts';
import {
  fetchCatalogProducts as fetchCatalogProductsData,
  fetchProductById as fetchProductByIdData,
  createProductFetch as createProductData,
  updateProductFetch as updateProductData,
  deleteProductFetch as deleteProductData,
} from './catalog_fetch';

// Types for payloads
export interface CreateProductPayload {
  name: string;
  price: number;
  description?: string;
  fabric_type?: string;
  season?: string;
  images?: string[];
  stock?: number;
  incoming?: number;
}

export interface UpdateProductPayload {
  name?: string;
  price?: number;
  description?: string;
  fabric_type?: string;
  season?: string;
  images?: string[];
  stock?: number;
  incoming?: number;
}

// Async thunks
export const fetchCatalogProducts = createAsyncThunk<ProductDto[]>(
  'products/fetchCatalog',
  async (_, { rejectWithValue }) => {
    try {
      return await fetchCatalogProductsData();
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const fetchProductById = createAsyncThunk<ProductDto, string>(
  'products/fetchById',
  async (id, { rejectWithValue }) => {
    try {
      return await fetchProductByIdData(id);
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const createProduct = createAsyncThunk<ProductDto, CreateProductPayload>(
  'products/create',
  async (payload, { rejectWithValue }) => {
    try {
      return await createProductData(payload);
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const updateProduct = createAsyncThunk<ProductDto, { id: string; payload: UpdateProductPayload }>(
  'products/update',
  async ({ id, payload }, { rejectWithValue }) => {
    try {
      return await updateProductData(id, payload);
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const deleteProduct = createAsyncThunk<void, string>(
  'products/delete',
  async (id, { rejectWithValue }) => {
    try {
      await deleteProductData(id);
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

// Slice state
export interface ProductsState {
  items: ProductDto[];
  selectedProduct: ProductDto | null;
  activeFilter: string;
  searchQuery: string;
  selectedAsset2D3D: string | null;
  loading: boolean;
  error: string | null;
}

const initialState: ProductsState = {
  items: [],
  selectedProduct: null,
  activeFilter: 'All Abayas',
  searchQuery: '',
  selectedAsset2D3D: null,
  loading: false,
  error: null,
};

const productsSlice = createSlice({
  name: 'products',
  initialState,
  reducers: {
    setActiveFilter: (state, action: PayloadAction<string>) => {
      state.activeFilter = action.payload;
    },
    setSearchQuery: (state, action: PayloadAction<string>) => {
      state.searchQuery = action.payload;
    },
    setSelectedAsset2D3D: (state, action: PayloadAction<string | null>) => {
      state.selectedAsset2D3D = action.payload;
    },
    setProducts: (state, action: PayloadAction<ProductDto[]>) => {
      state.items = action.payload;
    },
    removeProduct: (state, action: PayloadAction<string>) => {
      state.items = state.items.filter((p) => p.id !== action.payload);
    },
  },
  extraReducers: (builder) => {
    builder
      // Fetch catalog
      .addCase(fetchCatalogProducts.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchCatalogProducts.fulfilled, (state, action) => {
        state.loading = false;
        state.items = action.payload;
      })
      .addCase(fetchCatalogProducts.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ? String(action.payload) : 'Failed to fetch products';
      })
      // Fetch single product
      .addCase(fetchProductById.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchProductById.fulfilled, (state, action) => {
        state.loading = false;
        state.selectedProduct = action.payload;
      })
      .addCase(fetchProductById.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ? String(action.payload) : 'Failed to fetch product';
      })
      // Create product
      .addCase(createProduct.fulfilled, (state, action) => {
        state.items.push(action.payload);
      })
      // Update product
      .addCase(updateProduct.fulfilled, (state, action) => {
        const index = state.items.findIndex((p) => p.id === action.payload.id);
        if (index !== -1) {
          state.items[index] = action.payload;
        }
      })
      // Delete product
      .addCase(deleteProduct.fulfilled, (state, action) => {
        state.items = state.items.filter((p) => p.id !== action.meta.arg);
      });
  },
});

export const {
  setActiveFilter,
  setSearchQuery,
  setSelectedAsset2D3D,
  setProducts,
  removeProduct,
} = productsSlice.actions;

export default productsSlice.reducer;
