import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Home from '@/pages/storefront/Home';
import ProductDetail from '@/pages/storefront/ProductDetail';
import Checkout from '@/pages/storefront/Checkout';
import Login from '@/pages/auth/Login';
import InventoryDashboard from '@/pages/dashboard/InventoryDashboard';
import ModeratorPanel from '@/pages/dashboard/ModeratorPanel';
import OrdersPage from '@/pages/dashboard/OrdersPage';
import CustomerOrderPage from '@/pages/storefront/CustomerOrderPage';

export const AppRouter: React.FC = () => {
  return (
    <BrowserRouter>
      <Routes>
        {/* Storefront Routes */}
        <Route path="/" element={<Home />} />
        <Route path="/product/:id" element={<ProductDetail />} />
        <Route path="/checkout" element={<Checkout />} />
        <Route path="/orders/:id" element={<CustomerOrderPage />} />
        <Route path="/order-status/:id" element={<CustomerOrderPage />} />
        <Route path="/order-status" element={<CustomerOrderPage />} />

        {/* Auth Routes */}
        <Route path="/login" element={<Login />} />

        {/* Unified Admin & Moderator Dashboard Routes */}
        <Route path="/orders" element={<OrdersPage />} />
        <Route path="/dashboard/orders" element={<OrdersPage />} />
        <Route path="/admin" element={<InventoryDashboard />} />
        <Route path="/moderator" element={<ModeratorPanel />} />
        <Route path="/dashboard/moderator" element={<ModeratorPanel />} />
        <Route path="/dashboard" element={<InventoryDashboard />} />
        <Route path="/dashboard/*" element={<InventoryDashboard />} />

        {/* Fallback */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
};

export default AppRouter;
