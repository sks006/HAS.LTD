import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Home from '@/pages/storefront/Home';
import ProductDetail from '@/pages/storefront/ProductDetail';
import Checkout from '@/pages/storefront/Checkout';
import Login from '@/pages/auth/Login';
import ModeratorPanel from '@/pages/dashboard/ModeratorPanel';
import SystemGodMode from '@/pages/dashboard/SystemGodMode';

export const AppRouter: React.FC = () => {
  return (
    <BrowserRouter>
      <Routes>
        {/* Storefront Routes */}
        <Route path="/" element={<Home />} />
        <Route path="/product/:id" element={<ProductDetail />} />
        <Route path="/checkout" element={<Checkout />} />

        {/* Auth Routes */}
        <Route path="/login" element={<Login />} />

        {/* Protected Dashboard Zones */}
        <Route path="/dashboard/moderator" element={<ModeratorPanel />} />
        <Route path="/dashboard/admin" element={<SystemGodMode />} />

        {/* Fallback */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
};

export default AppRouter;
