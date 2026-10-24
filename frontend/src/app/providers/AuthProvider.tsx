import React, { createContext, useContext, useEffect } from 'react';
import { useRootStore } from '@/slicers/root_store';
import {
  fetchCurrentUser,
  logoutFetch,
  normalizeBackendUser,
  refreshTokenFetch,
} from '@/slicers/auth_slicer/auth_fetch';
import { ApiError, setApiToken } from '@/shared/api/client';
import type { Role } from '@/shared/types/roles';

interface AuthContextType {
  token: string | null;
  role: Role | null;
  user: import('@/shared/types/contracts').AuthUser | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType>({
  token: null,
  role: null,
  user: null,
  isAuthenticated: false,
  isLoading: true,
  logout: () => {},
});

export const useAuth = () => useContext(AuthContext);

interface AuthProviderProps {
  children: React.ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const token = useRootStore((s) => s.token);
  const role = useRootStore((s) => s.user?.role ?? null);
  const clearAuth = useRootStore((s) => s.logout);
  const setAuth = useRootStore((s) => s.setAuth);
  const [isLoading, setIsLoading] = React.useState(true);

  useEffect(() => {
    setApiToken(token);
    let active = true;
    if (token) {
      setIsLoading(false);
      return () => { active = false; };
    }

    setIsLoading(true);
    void refreshTokenFetch()
      .then(async ({ access_token }) => {
        const user = await fetchCurrentUser(access_token);
        if (active) setAuth(normalizeBackendUser(user), access_token);
      })
      .catch((error: unknown) => {
        if (!(error instanceof ApiError && error.status === 401)) {
          console.error('Unable to restore the backend session:', error);
        }
      })
      .finally(() => {
        if (active) setIsLoading(false);
      });

    return () => {
      active = false;
    };
  }, [token, setAuth]);

  const logout = async () => {
    if (!token) {
      setApiToken(null);
      clearAuth();
      return;
    }
    try {
      await logoutFetch(token);
    } catch (error) {
      console.error('Backend logout failed:', error);
    } finally {
      setApiToken(null);
      clearAuth();
    }
  };

  const user = useRootStore((s) => s.user);

  const value: AuthContextType = {
    token,
    role,
    user,
    isAuthenticated: Boolean(token),
    isLoading,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export default AuthProvider;
