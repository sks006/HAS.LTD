import { useStore } from '@/shared/store/store';
import type { Role } from '@/shared/types/roles';
import type { AuthUser } from '@/shared/types/contracts';

export const authSlice = {
  get user(): AuthUser | null { return useStore.getState().user; },
  get token(): string | null { return useStore.getState().token; },
  setAuth: (user: AuthUser | null, token: string | null) => useStore.getState().setAuth(user, token),
  setRole: (role: Role) => useStore.getState().setRole(role),
  logout: () => useStore.getState().logout(),
};
