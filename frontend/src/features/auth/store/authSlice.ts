import { useStore } from '@/shared/store/store';
import type { AuthUser } from '@/shared/types/contracts';

export const authSlice = {
  get user(): AuthUser | null { return useStore.getState().user; },
  get token(): string | null { return useStore.getState().token; },
  setAuth: (user: AuthUser | null, token: string | null) => useStore.getState().setAuth(user, token),
  logout: () => useStore.getState().logout(),
};
