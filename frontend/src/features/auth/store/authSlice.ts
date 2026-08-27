import { create } from 'zustand';

interface AuthState {
  token: string | null;
  role: 'SUPER_ADMIN' | 'MODERATOR' | 'CUSTOMER' | null;
  setAuth: (token: string | null, role: 'SUPER_ADMIN' | 'MODERATOR' | 'CUSTOMER' | null) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  token: localStorage.getItem('token'),
  role: localStorage.getItem('role') as any,
  setAuth: (token, role) => {
    if (token) localStorage.setItem('token', token);
    else localStorage.removeItem('token');
    
    if (role) localStorage.setItem('role', role);
    else localStorage.removeItem('role');
    
    set({ token, role });
  },
  clearAuth: () => {
    localStorage.removeItem('token');
    localStorage.removeItem('role');
    set({ token: null, role: null });
  },
}));
