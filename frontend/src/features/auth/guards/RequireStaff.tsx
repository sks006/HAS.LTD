import React, { type ReactNode } from 'react';
import { RequireStaff as StaffGuard } from './RequireAuth';

export const RequireStaff: React.FC<{ children: ReactNode; fallback?: ReactNode }> = ({ children, fallback }) => (
  <StaffGuard fallback={fallback}>{children}</StaffGuard>
);

export default RequireStaff;
