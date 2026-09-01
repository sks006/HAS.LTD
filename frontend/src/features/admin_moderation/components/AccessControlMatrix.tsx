'use client';

import React, { useState } from 'react';
import {
  ArrowUpRight,
  Box,
  CheckCircle2,
  CircleAlert,
  Lock,
  Pencil,
  Plus,
  Search,
  ShieldCheck,
  Trash2,
} from 'lucide-react';
import Button from '@/shared/ui/Button';
import type { InventoryItemDto } from '@/shared/types/contracts';
import { can, Role } from '@/shared/types/roles';
import { useRootStore } from '@/slicers/root_store';

const roleLabel: Record<Role, string> = {
  SUPER_ADMIN: 'SUPER_ADMIN',
  MODERATOR: 'MODERATOR',
  CUSTOMER: 'CUSTOMER',
};

export function AccessControlMatrix() {
  const user = useRootStore((s) => s.user);
  const setRole = useRootStore((s) => s.setRole);
  const inventory = useRootStore((s) => s.inventory);
  const [search, setSearch] = useState('');

  const role = user?.role ?? 'CUSTOMER';
  const canRead = can(role, 'inventory:read');
  const canCreate = can(role, 'product:create');
  const canUpdate = can(role, 'product:update');
  const canDelete = can(role, 'product:delete') || can(role, 'inventory:delete');
  const canViewMetrics = can(role, 'system:metrics_view');

  const filtered = inventory.filter(
    (i) => i.name.toLowerCase().includes(search.toLowerCase()) || i.sku.toLowerCase().includes(search.toLowerCase())
  );
  const totalStock = inventory.reduce((s, i) => s + i.stock, 0);
  const lowStock = inventory.filter((i) => i.status === 'Low stock').length;
  const outOfStock = inventory.filter((i) => i.status === 'Out of stock').length;
  const inStock = inventory.filter((i) => i.status === 'In stock').length;

  return (
    <div className="mx-auto max-w-[1440px] space-y-6 text-navy">
      <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.3em] text-taupe">
            Ajrah Noor Operations Workspace
          </p>
          <div className="flex items-center gap-3">
            <h2 className="font-serif text-3xl font-extrabold text-navy sm:text-4xl">
              Couture Inventory & RBAC Matrix
            </h2>
            <span className="rounded-full bg-emerald-500/10 border border-emerald-500/30 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide text-emerald-700">
              Active OCC
            </span>
          </div>
          <p className="mt-2 max-w-lg text-sm leading-6 text-slateblue">
            Manage stock levels, SKUs, and RBAC permissions across all Ajrah Noor boutiques.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {(['CUSTOMER', 'MODERATOR', 'SUPER_ADMIN'] as Role[]).map((r) => (
            <button
              key={r}
              type="button"
              onClick={() => setRole(r)}
              className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all ${
                role === r
                  ? 'bg-navy text-white shadow-sm'
                  : 'bg-white border border-lightgray text-slateblue hover:text-navy'
              }`}
            >
              {r === 'CUSTOMER' ? 'Customer' : r === 'MODERATOR' ? 'Moderator' : 'Admin'}
            </button>
          ))}
        </div>
      </div>

      {canRead ? (
        <>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {canViewMetrics && (
              <div className="p-4 bg-white border border-lightgray rounded-2xl flex items-center gap-3 shadow-sm">
                <div className="p-2.5 bg-cream text-navy rounded-xl">
                  <Box size={17} />
                </div>
                <div>
                  <p className="text-xs text-slateblue">Inventory Valuation</p>
                  <strong className="text-navy font-bold">SAR 184,500.00</strong>
                </div>
                <ArrowUpRight className="ml-auto text-slateblue" size={16} />
              </div>
            )}
            <div className="p-4 bg-white border border-lightgray rounded-2xl flex items-center gap-3 shadow-sm">
              <div className="p-2.5 bg-emerald-50 text-emerald-700 rounded-xl">
                <CheckCircle2 size={17} />
              </div>
              <div>
                <p className="text-xs text-slateblue">Tracked Pieces</p>
                <strong className="text-navy font-bold">{inventory.length} Abaya SKUs</strong>
              </div>
            </div>
            <div className="p-4 bg-white border border-lightgray rounded-2xl flex items-center gap-3 shadow-sm">
              <div className="p-2.5 bg-amber-50 text-amber-700 rounded-xl">
                <CircleAlert size={17} />
              </div>
              <div>
                <p className="text-xs text-slateblue">Low Stock Warning</p>
                <strong className="text-navy font-bold">{lowStock} SKUs</strong>
              </div>
            </div>
            <div className="p-4 bg-white border border-lightgray rounded-2xl flex items-center gap-3 shadow-sm">
              <div className="p-2.5 bg-rose-50 text-rose-700 rounded-xl">
                <CircleAlert size={17} />
              </div>
              <div>
                <p className="text-xs text-slateblue">Sold Out</p>
                <strong className="text-navy font-bold">{outOfStock} SKUs</strong>
              </div>
            </div>
          </div>

          <div className="overflow-hidden rounded-2xl border border-lightgray bg-white shadow-sm">
            <div className="flex flex-col gap-3 border-b border-lightgray p-4 sm:flex-row sm:items-center sm:justify-between">
              <div className="flex h-10 max-w-sm items-center gap-2 rounded-xl border border-lightgray bg-white px-3 text-slateblue">
                <Search size={15} />
                <input
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  className="w-full bg-transparent text-xs text-navy outline-none placeholder:text-slateblue/60"
                  placeholder="Search Abaya or SKU..."
                />
              </div>
              <div className="flex items-center gap-2">
                {canCreate && (
                  <Button variant="primary" size="sm" className="rounded-xl">
                    <Plus size={13} className="mr-1 inline" />
                    New Abaya SKU
                  </Button>
                )}
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-4 border-b border-lightgray px-4 py-3 text-[10px] text-slateblue bg-cream/30">
              <span className="font-semibold text-navy">{totalStock} total units</span>
              <span>In Stock: {inStock}</span>
              <span>Low Stock: {lowStock}</span>
              <span>Out of Stock: {outOfStock}</span>
              <span className="ml-auto hidden text-slateblue sm:block">
                Role: <strong className="text-navy">{roleLabel[role]}</strong>
              </span>
            </div>
            <InventoryTable items={filtered} canUpdate={canUpdate} canDelete={canDelete} />
          </div>
        </>
      ) : (
        <div className="flex flex-col items-center justify-center rounded-2xl border border-lightgray bg-white px-6 py-16 text-center shadow-sm">
          <div className="mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-rose-50 text-rose-600">
            <Lock size={24} />
          </div>
          <h3 className="text-2xl font-bold text-navy">Access Restricted</h3>
          <p className="mt-2 max-w-sm text-sm text-slateblue">
            The <strong className="font-semibold text-navy">CUSTOMER</strong> role does not have administrative access to the Ajrah Noor inventory matrix.
          </p>
        </div>
      )}
      <div className="flex items-center gap-2 text-xs text-slateblue">
        <ShieldCheck size={14} className="text-navy" />
        <span>
          Ajrah Noor Security Context: <strong className="text-navy">{roleLabel[role]}</strong>
        </span>
      </div>
    </div>
  );
}

function InventoryTable({
  items,
  canUpdate,
  canDelete,
}: {
  items: InventoryItemDto[];
  canUpdate: boolean;
  canDelete: boolean;
}) {
  if (items.length === 0)
    return <p className="py-12 text-center text-sm text-slateblue">No inventory matches your search.</p>;
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[760px] text-left border-collapse">
        <thead>
          <tr className="border-b border-lightgray text-[10px] uppercase tracking-[0.12em] text-slateblue bg-cream/20">
            <th className="px-4 py-3 font-semibold">Abaya Name</th>
            <th className="px-4 py-3 font-semibold">Category</th>
            <th className="px-4 py-3 font-semibold">SKU</th>
            <th className="px-4 py-3 font-semibold">Incoming</th>
            <th className="px-4 py-3 font-semibold">Stock</th>
            <th className="px-4 py-3 font-semibold">Status</th>
            <th className="px-4 py-3 font-semibold">Price</th>
            <th className="px-4 py-3 font-semibold">Action</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-lightgray">
          {items.map((item) => (
            <tr key={item.sku} className="text-xs hover:bg-cream/40 transition-colors">
              <td className="px-4 py-3 font-semibold text-navy">{item.name}</td>
              <td className="px-4 py-3 text-slateblue">{item.category}</td>
              <td className="px-4 py-3 text-slateblue font-mono">{item.sku}</td>
              <td className="px-4 py-3 text-navy">{item.incoming}</td>
              <td className="px-4 py-3 font-bold text-navy">{item.stock}</td>
              <td className="px-4 py-3">
                <span
                  className={`px-2 py-0.5 rounded-full text-[10px] font-bold ${
                    item.status === 'In stock'
                      ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                      : item.status === 'Low stock'
                      ? 'bg-amber-50 text-amber-700 border border-amber-200'
                      : 'bg-rose-50 text-rose-700 border border-rose-200'
                  }`}
                >
                  {item.status}
                </span>
              </td>
              <td className="px-4 py-3 font-semibold text-navy">
                {item.currency} {item.price.toLocaleString()}
              </td>
              <td className="px-4 py-3">
                <div className="flex items-center gap-2">
                  {canUpdate && (
                    <button className="text-slateblue hover:text-navy">
                      <Pencil size={14} />
                    </button>
                  )}
                  {canDelete && (
                    <button className="text-slateblue hover:text-rose-600">
                      <Trash2 size={14} />
                    </button>
                  )}
                  {!canUpdate && !canDelete && <span className="text-[10px] text-slateblue/60">Read-only</span>}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default AccessControlMatrix;
