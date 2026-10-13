'use client';

import React from 'react';
import { Shield, UserCheck, Users } from 'lucide-react';
import { useRootStore } from '@/slicers/root_store';

export function GlobalRevenueDashboard() {
  const inventory = useRootStore((s) => s.inventory);
  const totalValue = inventory.reduce((s, i) => s + i.price * i.stock, 0);

  return (
    <div className="grid gap-4 sm:grid-cols-3">
      <div className="p-6 bg-white border border-lightgray rounded-2xl flex items-center gap-4 shadow-sm">
        <div className="p-3 bg-navy/10 text-navy rounded-xl">
          <Shield size={20} />
        </div>
        <div>
          <p className="text-xs font-semibold text-slateblue uppercase tracking-wider">System Revenue</p>
          <h4 className="text-2xl font-extrabold text-navy mt-1">
            SAR {totalValue.toLocaleString()}
          </h4>
        </div>
      </div>

      <div className="p-6 bg-white border border-lightgray rounded-2xl flex items-center gap-4 shadow-sm">
        <div className="p-3 bg-emerald-50 text-emerald-700 rounded-xl">
          <UserCheck size={20} />
        </div>
        <div>
          <p className="text-xs font-semibold text-slateblue uppercase tracking-wider">Active Operators</p>
          <h4 className="text-2xl font-extrabold text-emerald-700 mt-1">3 Operators</h4>
        </div>
      </div>

      <div className="p-6 bg-white border border-lightgray rounded-2xl flex items-center gap-4 shadow-sm">
        <div className="p-3 bg-taupe/10 text-taupe rounded-xl">
          <Users size={20} />
        </div>
        <div>
          <p className="text-xs font-semibold text-slateblue uppercase tracking-wider">Registered Users</p>
          <h4 className="text-2xl font-extrabold text-navy mt-1">1,284 Users</h4>
        </div>
      </div>
    </div>
  );
}

export default GlobalRevenueDashboard;
