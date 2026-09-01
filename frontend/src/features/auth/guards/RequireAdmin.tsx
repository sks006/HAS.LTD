import React, { type ReactNode } from 'react';
import { RequireAdmin as AdminGuard } from './RequireAuth';

export const RequireAdmin: React.FC<{ children: ReactNode; fallback?: ReactNode }> = ({ children, fallback }) => (
  <AdminGuard fallback={fallback}>{children}</AdminGuard>
);

export default RequireAdmin;
