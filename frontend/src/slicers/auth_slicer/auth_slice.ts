import type { StateCreator } from 'zustand';
import type { AuthUser } from '@/shared/types/contracts';
import type { Role } from '@/shared/types/roles';

export interface AuthSlice {
  user: AuthUser | null;
  token: string | null;
  setAuth: (user: AuthUser | null, token: string | null) => void;
  setRole: (role: Role) => void;
  logout: () => void;
}

export const createAuthSlice: StateCreator<AuthSlice, [], [], AuthSlice> = (set) => ({
  user: { id: 'demo-admin', email: 'admin@lamina.sa', role: 'SUPER_ADMIN' },
  token: null,
  setAuth: (user, token) => set({ user, token }),
  setRole: (role) =>
    set((state) => ({
      user: state.user
        ? { ...state.user, role }
        : { id: 'demo-user', email: 'user@lamina.sa', role },
    })),
  logout: () => set({ user: null, token: null }),
});
