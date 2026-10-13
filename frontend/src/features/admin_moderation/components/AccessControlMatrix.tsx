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
  X,
  Loader2,
} from 'lucide-react';
import Button from '@/shared/ui/Button';
import type { InventoryItemDto } from '@/shared/types/contracts';
import { can, Role } from '@/shared/types/roles';
import { useRootStore } from '@/slicers/root_store';
import { useProducts, useUpdateProduct, useDeleteProduct } from '@/features/catalog/hooks/useProducts';
import { AddProductModal } from './AddProductModal';

const roleLabel: Record<Role, string> = {
  SUPER_ADMIN: 'SUPER_ADMIN',
  MODERATOR: 'MODERATOR',
  CUSTOMER: 'CUSTOMER',
};

export function AccessControlMatrix() {
  useProducts(); // Fetch live products from backend database
  const deleteMutation = useDeleteProduct();
  const removeInventoryItem = useRootStore((s) => s.removeInventoryItem);
  const removeProduct = useRootStore((s) => s.removeProduct);

  const user = useRootStore((s) => s.user);
  const setRole = useRootStore((s) => s.setRole);
  const inventory = useRootStore((s) => s.inventory);
  const [search, setSearch] = useState('');
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<InventoryItemDto | null>(null);
  const [actionNotice, setActionNotice] = useState<string | null>(null);

  const role = user?.role ?? 'CUSTOMER';
  const canRead = can(role, 'inventory:read');
  const canCreate = can(role, 'product:create');
  const canUpdate = can(role, 'product:update');
  const canDelete = can(role, 'product:delete') || can(role, 'inventory:delete');
  const canViewMetrics = can(role, 'system:metrics_view');

  const handleDeleteProduct = async (item: InventoryItemDto) => {
    if (!window.confirm(`Are you sure you want to delete "${item.name}" from backend database?`)) return;

    try {
      const targetId = item.id || item.sku;
      if (targetId) {
        await deleteMutation.mutateAsync(targetId);
      }
      if (item.id) removeProduct(item.id);
      if (item.sku) removeInventoryItem(item.sku);
      if (item.id) removeInventoryItem(item.id);

      setActionNotice(`Successfully deleted "${item.name}" from database.`);
      setTimeout(() => setActionNotice(null), 4000);
    } catch (err: any) {
      console.warn('Backend delete fallback:', err);
      if (item.id) removeProduct(item.id);
      if (item.sku) removeInventoryItem(item.sku);
      if (item.id) removeInventoryItem(item.id);

      setActionNotice(`Deleted "${item.name}" locally.`);
      setTimeout(() => setActionNotice(null), 4000);
    }
  };

  const filtered = inventory.filter(
    (i) => i.name.toLowerCase().includes(search.toLowerCase()) || i.sku.toLowerCase().includes(search.toLowerCase())
  );
  const totalStock = inventory.reduce((s, i) => s + i.stock, 0);
  const lowStock = inventory.filter((i) => i.status === 'Low stock').length;
  const outOfStock = inventory.filter((i) => i.status === 'Out of stock').length;
  const inStock = inventory.filter((i) => i.status === 'In stock').length;

  return (
    <div className="mx-auto max-w-[1440px] space-y-6 text-navy">
      <AddProductModal isOpen={isAddModalOpen} onClose={() => setIsAddModalOpen(false)} />
      {editingItem && (
        <EditProductModal item={editingItem} onClose={() => setEditingItem(null)} />
      )}

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
              Database Active
            </span>
          </div>
          <p className="mt-2 max-w-lg text-sm leading-6 text-slateblue">
            Manage stock levels, SKUs, images, and incoming inventory in PostgreSQL database.
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

      {actionNotice && (
        <div className="p-4 bg-emerald-50 border border-emerald-300 rounded-2xl flex items-center space-x-3 text-emerald-800 text-xs font-semibold shadow-xs animate-in fade-in">
          <CheckCircle2 size={18} className="text-emerald-600 shrink-0" />
          <span>{actionNotice}</span>
        </div>
      )}

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
                  <Button
                    variant="primary"
                    size="sm"
                    className="rounded-xl"
                    onClick={() => setIsAddModalOpen(true)}
                  >
                    <Plus size={13} className="mr-1 inline" />
                    Add Product to Backend DB
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
            <InventoryTable
              items={filtered}
              canUpdate={canUpdate}
              canDelete={canDelete}
              onEdit={(item) => setEditingItem(item)}
              onDelete={handleDeleteProduct}
            />
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
  onEdit,
  onDelete,
}: {
  items: InventoryItemDto[];
  canUpdate: boolean;
  canDelete: boolean;
  onEdit: (item: InventoryItemDto) => void;
  onDelete: (item: InventoryItemDto) => void;
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
            <th className="px-4 py-3 font-semibold">SKU / ID</th>
            <th className="px-4 py-3 font-semibold">Incoming</th>
            <th className="px-4 py-3 font-semibold">Stock</th>
            <th className="px-4 py-3 font-semibold">Status</th>
            <th className="px-4 py-3 font-semibold">Price</th>
            <th className="px-4 py-3 font-semibold">Action</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-lightgray">
          {items.map((item) => (
            <tr key={item.sku || item.id} className="text-xs hover:bg-cream/40 transition-colors">
              <td className="px-4 py-3 font-semibold text-navy flex items-center gap-2">
                {item.image ? (
                  <img src={item.image} alt={item.name} className="w-8 h-8 rounded-lg object-cover border border-gray-200 shadow-xs shrink-0" />
                ) : (
                  <div className="w-8 h-8 rounded-lg bg-cream border border-gray-200 flex items-center justify-center text-[10px] text-navy font-bold shrink-0">
                    AN
                  </div>
                )}
                <span className="truncate max-w-[160px]">{item.name}</span>
              </td>
              <td className="px-4 py-3 text-slateblue">{item.category}</td>
              <td className="px-4 py-3 text-slateblue font-mono text-[11px] truncate max-w-[120px]">{item.sku}</td>
              <td className="px-4 py-3 text-navy font-semibold">{item.incoming}</td>
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
                    <button
                      onClick={() => onEdit(item)}
                      title="Edit Product"
                      className="p-1 text-slateblue hover:text-navy hover:bg-gray-100 rounded-lg transition-colors"
                    >
                      <Pencil size={14} />
                    </button>
                  )}
                  {canDelete && (
                    <button
                      onClick={() => onDelete(item)}
                      title="Delete Product"
                      className="p-1 text-slateblue hover:text-rose-600 hover:bg-rose-50 rounded-lg transition-colors"
                    >
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

function EditProductModal({
  item,
  onClose,
}: {
  item: InventoryItemDto;
  onClose: () => void;
}) {
  const updateMutation = useUpdateProduct();
  const updateInventoryItem = useRootStore((s) => s.updateInventoryItem);

  const [name, setName] = useState(item.name);
  const [price, setPrice] = useState(item.price);
  const [category, setCategory] = useState(item.category);
  const [stock, setStock] = useState(item.stock);
  const [incoming, setIncoming] = useState(item.incoming);
  const [image, setImage] = useState(item.image || '');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const stockNum = Number(stock);
    const incNum = Number(incoming);
    const status = stockNum === 0 ? 'Out of stock' : stockNum < 20 ? 'Low stock' : 'In stock';
    const key = item.sku || item.id;

    try {
      if (item.id) {
        await updateMutation.mutateAsync({
          id: item.id,
          payload: {
            name,
            price: Number(price),
            fabric_type: category,
            stock: stockNum,
            incoming: incNum,
            images: image ? [image] : [],
          },
        });
      }
      updateInventoryItem(key, {
        name,
        price: Number(price),
        category,
        stock: stockNum,
        incoming: incNum,
        status,
        image,
      });
      onClose();
    } catch (err) {
      console.warn('Backend update notice:', err);
      updateInventoryItem(key, {
        name,
        price: Number(price),
        category,
        stock: stockNum,
        incoming: incNum,
        status,
        image,
      });
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="fixed inset-0 bg-black/50 backdrop-blur-xs" onClick={onClose} />
      <div className="relative bg-white rounded-2xl shadow-xl w-full max-w-md overflow-hidden border border-gray-100 z-10 animate-in fade-in duration-200">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
          <h2 className="text-base font-bold text-gray-900">Edit Product & Inventory</h2>
          <button onClick={onClose} className="p-1 rounded-lg text-gray-400 hover:text-gray-600">
            <X className="w-5 h-5" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4 text-xs">
          <div>
            <label className="block text-gray-700 font-semibold mb-1">Product Name</label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500/20 text-xs"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-gray-700 font-semibold mb-1">Category</label>
              <input
                type="text"
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl text-xs"
              />
            </div>

            <div>
              <label className="block text-gray-700 font-semibold mb-1">Price ($)</label>
              <input
                type="number"
                min="0"
                value={price}
                onChange={(e) => setPrice(Number(e.target.value))}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl text-xs"
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-gray-700 font-semibold mb-1">Stock Quantity</label>
              <input
                type="number"
                min="0"
                value={stock}
                onChange={(e) => setStock(Number(e.target.value))}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl text-xs font-bold text-navy"
              />
            </div>

            <div>
              <label className="block text-gray-700 font-semibold mb-1">Incoming Stock</label>
              <input
                type="number"
                min="0"
                value={incoming}
                onChange={(e) => setIncoming(Number(e.target.value))}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl text-xs font-bold text-navy"
              />
            </div>
          </div>

          <div>
            <label className="block text-gray-700 font-semibold mb-1">Product Image (URL / Data URI)</label>
            <input
              type="text"
              value={image}
              onChange={(e) => setImage(e.target.value)}
              placeholder="https://images.unsplash.com/..."
              className="w-full px-3 py-2 border border-gray-200 rounded-xl text-xs"
            />
            {image && (
              <div className="mt-2 flex items-center gap-2">
                <img src={image} alt="Preview" className="w-10 h-10 rounded-lg object-cover border border-gray-200" />
                <span className="text-[10px] text-gray-500">Image Preview</span>
              </div>
            )}
          </div>

          <div className="pt-4 flex items-center justify-end space-x-3 border-t border-gray-100">
            <button type="button" onClick={onClose} className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-xl">
              Cancel
            </button>
            <button
              type="submit"
              disabled={updateMutation.isPending}
              className="px-4 py-2 bg-navy text-white rounded-xl font-semibold shadow-xs flex items-center space-x-1.5"
            >
              {updateMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
              <span>Save Changes</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default AccessControlMatrix;
