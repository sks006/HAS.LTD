import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { AuthUser } from '@/shared/types/contracts';
import type { Role } from '@/shared/types/roles';

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
      return await apiFetch<AuthResponse>(ENDPOINTS.LOGIN, {
        method: 'POST',
        body: { email, password },
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const registerUser = createAsyncThunk<AuthResponse, RegisterPayload>(
  'auth/register',
  async ({ email, password }, { rejectWithValue }) => {
    try {
      return await apiFetch<AuthResponse>(ENDPOINTS.REGISTER, {
        method: 'POST',
        body: { email, password },
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const refreshToken = createAsyncThunk<{ token: string }, string>(
  'auth/refreshToken',
  async (token, { rejectWithValue }) => {
    try {
      return await apiFetch<{ token: string }>(ENDPOINTS.REFRESH, {
        method: 'POST',
        token,
      });
    } catch (error) {
      return rejectWithValue(error);
    }
  }
);

export const logoutUser = createAsyncThunk<void, string>(
  'auth/logout',
  async (token, { rejectWithValue }) => {
    try {
      await apiFetch<void>(ENDPOINTS.LOGOUT, {
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
  user: { id: 'demo-admin', email: 'admin@lamina.sa', role: 'SUPER_ADMIN' },
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
    setRole: (state, action: PayloadAction<Role>) => {
      if (state.user) {
        state.user.role = action.payload;
      } else {
        state.user = { id: 'demo-user', email: 'user@lamina.sa', role: action.payload };
      }
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
      .addCase(registerUser.fulfilled, (state, action) => {
        state.loading = false;
        state.user = action.payload.user;
        state.token = action.payload.token;
      })
      .addCase(registerUser.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload ? String(action.payload) : 'Registration failed';
      })
      // Refresh
      .addCase(refreshToken.fulfilled, (state, action) => {
        state.token = action.payload.token;
      })
      // Logout
      .addCase(logoutUser.fulfilled, (state) => {
        state.user = null;
        state.token = null;
      });
  },
});

export const { setAuth, setRole, logout } = authSlice.actions;

export default authSlice.reducer;
