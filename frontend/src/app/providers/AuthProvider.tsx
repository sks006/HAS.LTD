import React, { createContext, useContext, useEffect, useState } from 'react';

interface AuthContextType {
  token: string | null;
  role: 'SUPER_ADMIN' | 'MODERATOR' | 'CUSTOMER' | null;
  login: (token: string, role: 'SUPER_ADMIN' | 'MODERATOR' | 'CUSTOMER') => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
  const [role, setRole] = useState<'SUPER_ADMIN' | 'MODERATOR' | 'CUSTOMER' | null>(
    localStorage.getItem('role') as any
  );

  const login = (newToken: string, newRole: 'SUPER_ADMIN' | 'MODERATOR' | 'CUSTOMER') => {
    localStorage.setItem('token', newToken);
    localStorage.setItem('role', newRole);
    setToken(newToken);
    setRole(newRole);
  };

  const logout = () => {
    localStorage.removeItem('token');
    localStorage.removeItem('role');
    setToken(null);
    setRole(null);
  };

  return (
    <AuthContext.Provider value={{ token, role, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
