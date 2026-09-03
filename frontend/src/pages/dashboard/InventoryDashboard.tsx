import React from 'react';
import DashboardLayout from '@/features/admin_moderation/components/DashboardLayout';
import { GlobalRevenueDashboard } from '@/features/admin_moderation/components/GlobalRevenueDashboard';
import { AccessControlMatrix } from '@/features/admin_moderation/components/AccessControlMatrix';

export const InventoryDashboard: React.FC = () => {
  return (
    <DashboardLayout>
      <div className="space-y-6 max-w-7xl mx-auto">
        <GlobalRevenueDashboard />
        <AccessControlMatrix />
      </div>
    </DashboardLayout>
  );
};

export default InventoryDashboard;
