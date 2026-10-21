import React, { createContext, useContext, useEffect } from 'react';
import { useRootStore } from '@/slicers/root_store';
import type { Role } from '@/shared/types/roles';

interface AuthContextType {
  token: string | null;
  role: Role | null;
  isAuthenticated: boolean;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType>({
  token: null,
  role: null,
  isAuthenticated: false,
  logout: () => {},
});

export const useAuth = () => useContext(AuthContext);

interface AuthProviderProps {
  children: React.ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const token = useRootStore((s) => s.token);
  const role = useRootStore((s) => s.user?.role ?? null);
  const logout = useRootStore((s) => s.logout);

  useEffect(() => {
    // Optionally validate token or listen for auth events here
  }, [token]);

  const value: AuthContextType = {
    token,
    role,
    isAuthenticated: Boolean(token),
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export default AuthProvider;
