import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Home from '@/pages/storefront/Home';
import ProductDetail from '@/pages/storefront/ProductDetail';
import Checkout from '@/pages/storefront/Checkout';
import Login from '@/pages/auth/Login';
import Register from '@/pages/auth/Register';
import InventoryDashboard from '@/pages/dashboard/InventoryDashboard';
import ModeratorPanel from '@/pages/dashboard/ModeratorPanel';
import OrdersPage from '@/pages/dashboard/OrdersPage';
import CustomerOrderPage from '@/pages/storefront/CustomerOrderPage';
import { RequireStaff } from '@/features/auth/guards/RequireStaff';

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
        <Route path="/register" element={<Register />} />

        {/* Protected Staff Portal Dashboard Routes (SUPER_ADMIN and MODERATOR) */}
        <Route
          path="/orders"
          element={
            <RequireStaff>
              <OrdersPage />
            </RequireStaff>
          }
        />
        <Route
          path="/dashboard/orders"
          element={
            <RequireStaff>
              <OrdersPage />
            </RequireStaff>
          }
        />
        <Route
          path="/admin"
          element={
            <RequireStaff>
              <InventoryDashboard />
            </RequireStaff>
          }
        />
        <Route
          path="/moderator"
          element={
            <RequireStaff>
              <ModeratorPanel />
            </RequireStaff>
          }
        />
        <Route
          path="/dashboard/moderator"
          element={
            <RequireStaff>
              <ModeratorPanel />
            </RequireStaff>
          }
        />
        <Route
          path="/dashboard"
          element={
            <RequireStaff>
              <InventoryDashboard />
            </RequireStaff>
          }
        />
        <Route
          path="/dashboard/*"
          element={
            <RequireStaff>
              <InventoryDashboard />
            </RequireStaff>
          }
        />

        {/* Fallback */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
};

export default AppRouter;
