import React from 'react';
import { RequireAdmin } from '@/features/auth/guards/RequireAdmin';
import { GlobalRevenueDashboard } from '@/features/admin_moderation/components/GlobalRevenueDashboard';
import { AccessControlMatrix } from '@/features/admin_moderation/components/AccessControlMatrix';
import { useNavigate } from 'react-router-dom';
import Button from '@/shared/ui/Button';

export const SystemGodModeContent: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-white text-navy p-6 md:p-12 space-y-8 font-sans">
      <div className="flex items-center justify-between border-b border-lightgray pb-6">
        <div>
          <span className="text-xs uppercase font-mono text-taupe font-bold">God Mode Domain</span>
          <h1 className="text-3xl font-extrabold text-navy mt-1">System Operational Admin</h1>
        </div>
        <Button variant="outline" size="sm" onClick={() => navigate('/')}>
          Storefront
        </Button>
      </div>

      <GlobalRevenueDashboard />
      <AccessControlMatrix />
    </div>
  );
};

export const SystemGodMode: React.FC = () => (
  <RequireAdmin>
    <SystemGodModeContent />
  </RequireAdmin>
);

export default SystemGodMode;
