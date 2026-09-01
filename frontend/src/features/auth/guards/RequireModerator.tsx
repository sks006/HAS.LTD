import React, { type ReactNode } from 'react';
import { RequireModerator as ModGuard } from './RequireAuth';

export const RequireModerator: React.FC<{ children: ReactNode; fallback?: ReactNode }> = ({ children, fallback }) => (
  <ModGuard fallback={fallback}>{children}</ModGuard>
);

export default RequireModerator;
