import React from 'react';
import DashboardLayout from '@/features/admin_moderation/components/DashboardLayout';
import { GlobalRevenueDashboard } from '@/features/admin_moderation/components/GlobalRevenueDashboard';
import { AccessControlMatrix } from '@/features/admin_moderation/components/AccessControlMatrix';
import { useAuth } from '@/app/providers/AuthProvider';
import { Card, CardContent } from '@/shared/ui/card';
import { Badge } from '@/shared/ui/badge';
import { ShieldCheck, Info } from 'lucide-react';

export const InventoryDashboard: React.FC = () => {
  const { role } = useAuth();
  const isSuperAdmin = role === 'SUPER_ADMIN';

  return (
    <DashboardLayout activeTab="Products">
      <div className="space-y-6 max-w-7xl mx-auto">
        {/* Super Admin Global Revenue & System Telemetry */}
        {isSuperAdmin ? (
          <GlobalRevenueDashboard />
        ) : (
          <Card className="rounded-2xl border-purple-200 bg-purple-50/50 shadow-2xs">
            <CardContent className="p-4 flex items-center justify-between">
              <div className="flex items-center space-x-3">
                <div className="p-2.5 bg-purple-100 text-purple-700 rounded-xl">
                  <ShieldCheck size={20} />
                </div>
                <div>
                  <div className="flex items-center space-x-2">
                    <h3 className="font-bold text-sm text-neutral-900 font-serif">
                      Moderator Atelier Workspace
                    </h3>
                    <Badge variant="outline" className="bg-purple-100 text-purple-800 border-purple-200 text-[10px] uppercase font-mono">
                      Moderation Clearance
                    </Badge>
                  </div>
                  <p className="text-xs text-neutral-600 mt-0.5">
                    You have permission to inspect abaya inventory, edit stock counts, and verify customer orders.
                  </p>
                </div>
              </div>
              <div className="hidden sm:flex items-center text-[11px] text-neutral-500 font-medium">
                <Info size={13} className="mr-1 text-purple-600" />
                <span>Product creation & telemetry metrics are reserved for Super Admin</span>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Inventory & Stock Matrix */}
        <AccessControlMatrix />
      </div>
    </DashboardLayout>
  );
};

export default InventoryDashboard;
