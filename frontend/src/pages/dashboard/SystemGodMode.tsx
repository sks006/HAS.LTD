import React from 'react';
import { RequireAdmin } from '@/features/auth/guards/RequireAdmin';
import { GlobalRevenueDashboard } from '@/features/admin_moderation/components/GlobalRevenueDashboard';
import { AccessControlMatrix } from '@/features/admin_moderation/components/AccessControlMatrix';
import { useNavigate } from 'react-router-dom';
import Button from '@/shared/ui/Button';

import DashboardLayout from '@/features/admin_moderation/components/DashboardLayout';

export const SystemGodModeContent: React.FC = () => {
  return (
    <DashboardLayout activeTab="Products">
      <div className="space-y-8 max-w-7xl mx-auto">
        <div className="flex items-center justify-between border-b border-gray-200 pb-4">
          <div>
            <span className="text-xs uppercase font-mono text-gray-400 font-bold">God Mode Domain</span>
            <h1 className="text-2xl font-extrabold text-gray-900 mt-0.5">System Operational Admin</h1>
          </div>
        </div>

        <GlobalRevenueDashboard />
        <AccessControlMatrix />
      </div>
    </DashboardLayout>
  );
};

export const SystemGodMode: React.FC = () => (
  <RequireAdmin>
    <SystemGodModeContent />
  </RequireAdmin>
);

export default SystemGodMode;
