import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { AuthUser } from '@/shared/types/contracts';
import {
  loginFetch,
  normalizeBackendUser,
  registerFetch,
  type RegisterResponse,
} from './auth_fetch';

// Types for payloads
export interface LoginPayload {
  email: string;
  password?: string;
}

export interface RegisterPayload {
  email: string;
  password?: string;
}

export interface AuthResponse {
  user: AuthUser;
  token: string;
}

// Async thunks
export const loginUser = createAsyncThunk<AuthResponse, LoginPayload>(
  'auth/login',
  async ({ email, password }, { rejectWithValue }) => {
    try {
      const response = await loginFetch(email, password || '');
      if (!response.access_token || !response.user) {
        throw new Error('Additional MFA verification is required. Use the sign-in form to continue.');
      }
      return { user: normalizeBackendUser(response.user), token: response.access_token };
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const registerUser = createAsyncThunk<RegisterResponse, RegisterPayload>(
  'auth/register',
  async ({ email, password }, { rejectWithValue }) => {
    try {
      return await registerFetch(email, password || '');
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const refreshToken = createAsyncThunk<{ access_token: string }, void>(
  'auth/refreshToken',
  async (_, { rejectWithValue }) => {
    try {
      return await apiFetch<{ access_token: string }>(ENDPOINTS.REFRESH, { method: 'POST' });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const logoutUser = createAsyncThunk<void, string>(
  'auth/logout',
  async (token, { rejectWithValue }) => {
    try {
      await apiFetch<{ success: boolean }>(ENDPOINTS.LOGOUT, {
        method: 'POST',
        token,
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

// Slice state
export interface AuthState {
  user: AuthUser | null;
  token: string | null;
  loading: boolean;
  error: string | null;
}

const initialState: AuthState = {
  user: null,
  token: null,
  loading: false,
  error: null,
};

const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    setAuth: (
      state,
      action: PayloadAction<{ user: AuthUser | null; token: string | null } | { user: AuthUser | null; token?: string | null }>
    ) => {
      state.user = action.payload.user;
      state.token = action.payload.token ?? null;
    },
    logout: (state) => {
      state.user = null;
      state.token = null;
      state.loading = false;
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      // Login
      .addCase(loginUser.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(loginUser.fulfilled, (state, action) => {
        state.loading = false;
        state.user = action.payload.user;
        state.token = action.payload.token;
      })
      .addCase(loginUser.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ? String(action.payload) : 'Login failed';
      })
      // Register
      .addCase(registerUser.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(registerUser.fulfilled, (state) => {
        state.loading = false;
        state.error = null;
      })
      .addCase(registerUser.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ? String(action.payload) : 'Registration failed';
      })
      // Refresh
      .addCase(refreshToken.fulfilled, (state, action) => {
        state.token = action.payload.access_token;
      })
      // Logout
      .addCase(logoutUser.fulfilled, (state) => {
        state.user = null;
        state.token = null;
      });
  },
});

export const { setAuth, logout } = authSlice.actions;

export default authSlice.reducer;
