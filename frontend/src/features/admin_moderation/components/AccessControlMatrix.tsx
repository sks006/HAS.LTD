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
  Loader2,
} from 'lucide-react';
import { Button } from '@/shared/ui/button';
import { Badge } from '@/shared/ui/badge';
import { Input } from '@/shared/ui/input';
import { Card, CardContent } from '@/shared/ui/card';
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
  TableCell,
} from '@/shared/ui/table';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/shared/ui/dialog';
import { Label } from '@/shared/ui/label';
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
  const productsQuery = useProducts();
  const deleteMutation = useDeleteProduct();

  const user = useRootStore((s) => s.user);
  const inventory = useRootStore((s) => s.inventory);
  const [search, setSearch] = useState('');
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<InventoryItemDto | null>(null);
  const [actionNotice, setActionNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const role = user?.role ?? 'CUSTOMER';
  const canRead = can(role, 'inventory:read');
  const canCreate = can(role, 'product:create');
  const canUpdate = role === 'SUPER_ADMIN';
  const canDelete = role === 'SUPER_ADMIN';
  const canViewMetrics = can(role, 'system:metrics_view');

  const handleDeleteProduct = async (item: InventoryItemDto) => {
    if (!window.confirm(`Are you sure you want to delete "${item.name}" from backend database?`)) return;

    setActionError(null);
    try {
      const targetId = item.id || item.sku;
      if (targetId) {
        await deleteMutation.mutateAsync(targetId);
      }

      setActionNotice(`Successfully deleted "${item.name}" from database.`);
      setTimeout(() => setActionNotice(null), 4000);
    } catch (requestError) {
      setActionError(requestError instanceof Error ? requestError.message : 'Unable to delete the product.');
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
    <div className="mx-auto max-w-[1440px] space-y-6 text-neutral-900 font-sans">
      <AddProductModal isOpen={isAddModalOpen} onClose={() => setIsAddModalOpen(false)} />
      {editingItem && (
        <EditProductModal item={editingItem} onClose={() => setEditingItem(null)} />
      )}

      <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.3em] text-neutral-400">
            Ajrah Noor Operations Workspace
          </p>
          <div className="flex items-center gap-3">
            <h2 className="font-serif text-3xl font-extrabold text-neutral-900 sm:text-4xl">
              Couture Inventory & RBAC Matrix
            </h2>
            <Badge
              variant={productsQuery.isError ? 'destructive' : 'secondary'}
              className="font-mono text-[10px] uppercase"
            >
              {productsQuery.isError ? 'Database Unavailable' : productsQuery.isPending ? 'Connecting' : 'Database Connected'}
            </Badge>
          </div>
          <p className="mt-2 max-w-lg text-sm leading-6 text-neutral-500">
            Manage stock levels, SKUs, images, and incoming inventory in PostgreSQL database.
          </p>
        </div>
        <Badge variant="outline" className="px-3.5 py-1.5 text-xs font-bold bg-neutral-900 text-white border-neutral-900">
          Signed in as {roleLabel[role]}
        </Badge>
      </div>

      {productsQuery.isError && (
        <div role="alert" className="rounded-2xl border border-rose-200 bg-rose-50 p-4 text-xs text-rose-700">
          Unable to load products: {productsQuery.error instanceof Error ? productsQuery.error.message : 'Backend request failed.'}
        </div>
      )}
      {actionNotice && (
        <div className="p-4 bg-emerald-50 border border-emerald-300 rounded-2xl flex items-center space-x-3 text-emerald-800 text-xs font-semibold shadow-xs animate-in fade-in">
          <CheckCircle2 size={18} className="text-emerald-600 shrink-0" />
          <span>{actionNotice}</span>
        </div>
      )}
      {actionError && (
        <div role="alert" className="rounded-2xl border border-rose-200 bg-rose-50 p-4 text-xs text-rose-700">
          {actionError}
        </div>
      )}

      {canRead ? (
        <>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {canViewMetrics && (
              <Card className="rounded-2xl border-neutral-200 shadow-xs">
                <CardContent className="p-4 flex items-center gap-3">
                  <div className="p-2.5 bg-neutral-100 text-neutral-800 rounded-xl">
                    <Box size={17} />
                  </div>
                  <div>
                    <p className="text-xs text-neutral-500">Inventory Valuation</p>
                    <strong className="text-neutral-900 font-bold font-mono">SAR 184,500.00</strong>
                  </div>
                  <ArrowUpRight className="ml-auto text-neutral-400" size={16} />
                </CardContent>
              </Card>
            )}
            <Card className="rounded-2xl border-neutral-200 shadow-xs">
              <CardContent className="p-4 flex items-center gap-3">
                <div className="p-2.5 bg-emerald-50 text-emerald-700 rounded-xl">
                  <CheckCircle2 size={17} />
                </div>
                <div>
                  <p className="text-xs text-neutral-500">Tracked Pieces</p>
                  <strong className="text-neutral-900 font-bold">{inventory.length} Abaya SKUs</strong>
                </div>
              </CardContent>
            </Card>
            <Card className="rounded-2xl border-neutral-200 shadow-xs">
              <CardContent className="p-4 flex items-center gap-3">
                <div className="p-2.5 bg-amber-50 text-amber-700 rounded-xl">
                  <CircleAlert size={17} />
                </div>
                <div>
                  <p className="text-xs text-neutral-500">Low Stock Warning</p>
                  <strong className="text-neutral-900 font-bold">{lowStock} SKUs</strong>
                </div>
              </CardContent>
            </Card>
            <Card className="rounded-2xl border-neutral-200 shadow-xs">
              <CardContent className="p-4 flex items-center gap-3">
                <div className="p-2.5 bg-rose-50 text-rose-700 rounded-xl">
                  <CircleAlert size={17} />
                </div>
                <div>
                  <p className="text-xs text-neutral-500">Sold Out</p>
                  <strong className="text-neutral-900 font-bold">{outOfStock} SKUs</strong>
                </div>
              </CardContent>
            </Card>
          </div>

          <Card className="overflow-hidden rounded-2xl border-neutral-200 shadow-xs">
            <div className="flex flex-col gap-3 border-b border-neutral-200 p-4 sm:flex-row sm:items-center sm:justify-between bg-white">
              <div className="relative flex-1 max-w-sm">
                <Search size={15} className="text-neutral-400 absolute left-3 top-1/2 -translate-y-1/2" />
                <Input
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  className="pl-9 h-9 text-xs bg-neutral-50 border-neutral-200"
                  placeholder="Search Abaya or SKU..."
                />
              </div>
              <div className="flex items-center gap-2">
                {canCreate && (
                  <Button
                    variant="primary"
                    size="sm"
                    className="rounded-xl h-9"
                    onClick={() => setIsAddModalOpen(true)}
                  >
                    <Plus size={14} className="mr-1.5" />
                    <span>Add Product to Backend DB</span>
                  </Button>
                )}
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-4 border-b border-neutral-200 px-4 py-2.5 text-[10px] text-neutral-500 bg-neutral-50/50">
              <span className="font-semibold text-neutral-900">{totalStock} total units</span>
              <span>In Stock: {inStock}</span>
              <span>Low Stock: {lowStock}</span>
              <span>Out of Stock: {outOfStock}</span>
              <span className="ml-auto hidden sm:block">
                Role: <strong className="text-neutral-900">{roleLabel[role]}</strong>
              </span>
            </div>
            <InventoryTable
              items={filtered}
              canUpdate={canUpdate}
              canDelete={canDelete}
              onEdit={(item) => setEditingItem(item)}
              onDelete={handleDeleteProduct}
            />
          </Card>
        </>
      ) : (
        <Card className="flex flex-col items-center justify-center rounded-2xl border-neutral-200 p-16 text-center shadow-xs">
          <div className="mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-rose-50 text-rose-600">
            <Lock size={24} />
          </div>
          <h3 className="text-2xl font-bold text-neutral-900 font-serif">Access Restricted</h3>
          <p className="mt-2 max-w-sm text-sm text-neutral-500">
            The <strong className="font-semibold text-neutral-900">CUSTOMER</strong> role does not have administrative access to the Ajrah Noor inventory matrix.
          </p>
        </Card>
      )}
      <div className="flex items-center gap-2 text-xs text-neutral-500">
        <ShieldCheck size={14} className="text-neutral-900" />
        <span>
          Ajrah Noor Security Context: <strong className="text-neutral-900">{roleLabel[role]}</strong>
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
    return <p className="py-12 text-center text-sm text-neutral-500">No inventory matches your search.</p>;

  return (
    <Table>
      <TableHeader>
        <TableRow className="bg-neutral-50/70 border-b border-neutral-200 text-[10px] uppercase font-bold tracking-wider text-neutral-500">
          <TableHead className="py-3 px-4">Abaya Name</TableHead>
          <TableHead className="py-3 px-4">Category</TableHead>
          <TableHead className="py-3 px-4">SKU / ID</TableHead>
          <TableHead className="py-3 px-4">Incoming</TableHead>
          <TableHead className="py-3 px-4">Stock</TableHead>
          <TableHead className="py-3 px-4">Status</TableHead>
          <TableHead className="py-3 px-4">Price</TableHead>
          <TableHead className="py-3 px-4 text-right">Action</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody className="text-xs">
        {items.map((item) => (
          <TableRow key={item.sku || item.id} className="hover:bg-neutral-50/60 transition-colors">
            <TableCell className="py-3 px-4 font-semibold text-neutral-900">
              <div className="flex items-center gap-2.5">
                {item.image ? (
                  <img src={item.image} alt={item.name} className="w-8 h-8 rounded-lg object-cover border border-neutral-200 shadow-2xs shrink-0" />
                ) : (
                  <div className="w-8 h-8 rounded-lg bg-neutral-100 border border-neutral-200 flex items-center justify-center text-[10px] text-neutral-800 font-bold shrink-0">
                    AN
                  </div>
                )}
                <span className="truncate max-w-[180px]">{item.name}</span>
              </div>
            </TableCell>
            <TableCell className="py-3 px-4 text-neutral-600">{item.category}</TableCell>
            <TableCell className="py-3 px-4 text-neutral-500 font-mono text-[11px] truncate max-w-[120px]">{item.sku}</TableCell>
            <TableCell className="py-3 px-4 text-neutral-900 font-semibold">{item.incoming}</TableCell>
            <TableCell className="py-3 px-4 font-bold text-neutral-900">{item.stock}</TableCell>
            <TableCell className="py-3 px-4">
              <Badge
                variant={
                  item.status === 'In stock'
                    ? 'secondary'
                    : item.status === 'Low stock'
                    ? 'outline'
                    : 'destructive'
                }
                className="text-[10px] font-semibold"
              >
                {item.status}
              </Badge>
            </TableCell>
            <TableCell className="py-3 px-4 font-bold text-neutral-900 font-mono">
              {item.currency} {item.price.toLocaleString()}
            </TableCell>
            <TableCell className="py-3 px-4 text-right">
              <div className="flex items-center justify-end gap-1.5">
                {canUpdate && (
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => onEdit(item)}
                    title="Edit Product"
                    className="h-8 w-8 text-neutral-500 hover:text-neutral-900"
                  >
                    <Pencil size={14} />
                  </Button>
                )}
                {canDelete && (
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => onDelete(item)}
                    title="Delete Product"
                    className="h-8 w-8 text-neutral-500 hover:text-rose-600 hover:bg-rose-50"
                  >
                    <Trash2 size={14} />
                  </Button>
                )}
                {!canUpdate && !canDelete && <span className="text-[10px] text-neutral-400">Read-only</span>}
              </div>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
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

  const [name, setName] = useState(item.name);
  const [price, setPrice] = useState(item.price);
  const [category, setCategory] = useState(item.category);
  const [stock, setStock] = useState(item.stock);
  const [incoming, setIncoming] = useState(item.incoming);
  const [image, setImage] = useState(item.image || '');
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const stockNum = Number(stock);
    const incNum = Number(incoming);
    setError(null);

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
      onClose();
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : 'Unable to update the product.');
    }
  };

  return (
    <Dialog open={true} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-md bg-white border-neutral-200 rounded-2xl">
        <DialogHeader>
          <DialogTitle className="text-base font-bold text-neutral-900 font-serif">
            Edit Product & Inventory
          </DialogTitle>
          <DialogDescription className="text-xs text-neutral-500">
            Update pricing, stock counts, and atelier imagery in PostgreSQL.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 text-xs pt-2">
          {error && <p role="alert" className="rounded-xl bg-rose-50 p-3 text-rose-700">{error}</p>}
          <div className="space-y-1.5">
            <Label htmlFor="edit-name" className="text-neutral-700 font-semibold text-xs">Product Name</Label>
            <Input
              id="edit-name"
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="text-xs h-9 bg-neutral-50"
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label htmlFor="edit-cat" className="text-neutral-700 font-semibold text-xs">Category</Label>
              <Input
                id="edit-cat"
                type="text"
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                className="text-xs h-9 bg-neutral-50"
              />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="edit-price" className="text-neutral-700 font-semibold text-xs">Price ($)</Label>
              <Input
                id="edit-price"
                type="number"
                min="0"
                value={price}
                onChange={(e) => setPrice(Number(e.target.value))}
                className="text-xs h-9 bg-neutral-50 font-mono"
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label htmlFor="edit-stock" className="text-neutral-700 font-semibold text-xs">Stock Quantity</Label>
              <Input
                id="edit-stock"
                type="number"
                min="0"
                value={stock}
                onChange={(e) => setStock(Number(e.target.value))}
                className="text-xs h-9 bg-neutral-50 font-mono font-bold"
              />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="edit-inc" className="text-neutral-700 font-semibold text-xs">Incoming Stock</Label>
              <Input
                id="edit-inc"
                type="number"
                min="0"
                value={incoming}
                onChange={(e) => setIncoming(Number(e.target.value))}
                className="text-xs h-9 bg-neutral-50 font-mono font-bold"
              />
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="edit-img" className="text-neutral-700 font-semibold text-xs">Product Image (URL / Data URI)</Label>
            <Input
              id="edit-img"
              type="text"
              value={image}
              onChange={(e) => setImage(e.target.value)}
              placeholder="https://images.unsplash.com/..."
              className="text-xs h-9 bg-neutral-50"
            />
            {image && (
              <div className="mt-2 flex items-center gap-2">
                <img src={image} alt="Preview" className="w-10 h-10 rounded-lg object-cover border border-neutral-200" />
                <span className="text-[10px] text-neutral-500">Image Preview</span>
              </div>
            )}
          </div>

          <DialogFooter className="pt-3 border-t border-neutral-100 flex items-center justify-end space-x-2">
            <Button type="button" variant="outline" size="sm" onClick={onClose} className="rounded-xl h-9">
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              size="sm"
              disabled={updateMutation.isPending}
              className="rounded-xl h-9"
            >
              {updateMutation.isPending && <Loader2 className="w-3.5 h-3.5 mr-1.5 animate-spin" />}
              <span>Save Changes</span>
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

export default AccessControlMatrix;
