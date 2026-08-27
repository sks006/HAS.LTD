import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { StorefrontPage } from '../pages/storefront/StorefrontPage';
import { DashboardPage } from '../pages/dashboard/DashboardPage';
import { LoginPage } from '../pages/auth/LoginPage';
import { useAuthStore } from '../features/auth/store/authSlice';

export function AppRouter() {
  const token = useAuthStore((state) => state.token);
  const role = useAuthStore((state) => state.role);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<StorefrontPage />} />
        <Route 
          path="/login" 
          element={token ? <Navigate to="/" replace /> : <LoginPage />} 
        />
        <Route 
          path="/dashboard" 
          element={
            token && (role === 'SUPER_ADMIN' || role === 'MODERATOR') ? (
              <DashboardPage />
            ) : (
              <Navigate to="/login" replace />
            )
          } 
        />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
