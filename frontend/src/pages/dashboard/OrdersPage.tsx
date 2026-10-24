import React, { useState, useMemo, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import DashboardLayout from '@/features/admin_moderation/components/DashboardLayout';
import {
  RootState,
  AppDispatch,
} from '@/slicers/root_store';
import {
  fetchOrders,
  updateOrderStatusThunk,
  deleteOrderThunk,
  setActiveOrderFilter,
  setOrderSearchQuery,
  setOrderSortBy,
  addOrder,
  updateOrderStatus,
  removeOrder,
  removeMultipleOrders,
} from '@/slicers/order_slicer/order_slice';
import type { OrderRecord, OrderState } from '@/shared/types/contracts';
import { useAuth } from '@/app/providers/AuthProvider';
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
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
} from '@/shared/ui/dropdown-menu';
import { Alert, AlertDescription } from '@/shared/ui/alert';
import { Label } from '@/shared/ui/label';
import {
  Copy,
  CheckCircle2,
  Filter,
  CheckCircle,
  RotateCcw,
  SlidersHorizontal,
  ChevronDown,
  MoreVertical,
  ChevronLeft,
  ChevronRight,
  Search,
  Eye,
  Trash2,
  Package,
  MapPin,
  Shield,
  CreditCard,
  User,
} from 'lucide-react';

export const OrdersPage: React.FC = () => {
  const { role } = useAuth();
  const isSuperAdmin = role === 'SUPER_ADMIN';
  const dispatch = useDispatch<AppDispatch>();
  const { orders, activeFilter, searchQuery, sortBy, loading, error } = useSelector(
    (state: RootState) => state.orders
  );

  useEffect(() => {
    dispatch(fetchOrders());
  }, [dispatch]);

  // Local UI state
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // Modal states
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [inspectOrder, setInspectOrder] = useState<OrderRecord | null>(null);

  // Pagination state
  const [currentPage, setCurrentPage] = useState(1);
  const pageSize = 10;

  // Add Order Form state
  const [newOrderForm, setNewOrderForm] = useState({
    customerName: '',
    productName: '',
    sku: 'SK-45',
    paymentMethod: 'Cash on Delivery' as 'Cash on Delivery' | 'Paid via PayPal' | 'Credit Card' | 'Apple Pay',
    amount: '',
    status: 'Pending' as OrderState,
  });

  // Filter & Search Logic
  const filteredOrders = useMemo(() => {
    return orders.filter((order) => {
      // 1. Filter by Tab Status
      if (activeFilter === 'Active') {
        if (order.status !== 'Pending' && order.status !== 'Processing' && order.status !== 'Paid' && order.status !== 'Reserved') {
          return false;
        }
      } else if (activeFilter === 'Delivered') {
        if (order.status !== 'Delivered' && order.status !== 'Shipped') return false;
      } else if (activeFilter === 'Returned') {
        if (order.status !== 'Returned' && order.status !== 'Cancelled') return false;
      }

      // 2. Filter by Search Query
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesName = order.customer_name.toLowerCase().includes(q);
        const matchesProduct = order.product_name.toLowerCase().includes(q);
        const matchesId = order.order_number.toLowerCase().includes(q) || order.id.toLowerCase().includes(q);
        const matchesSku = order.sku.toLowerCase().includes(q);
        if (!matchesName && !matchesProduct && !matchesId && !matchesSku) {
          return false;
        }
      }

      return true;
    });
  }, [orders, activeFilter, searchQuery]);

  // Sort Logic
  const sortedOrders = useMemo(() => {
    const list = [...filteredOrders];
    if (sortBy === 'Amount (High-Low)') {
      list.sort((a, b) => b.amount - a.amount);
    } else if (sortBy === 'Amount (Low-High)') {
      list.sort((a, b) => a.amount - b.amount);
    } else if (sortBy === 'Customer Name') {
      list.sort((a, b) => a.customer_name.localeCompare(b.customer_name));
    } else {
      // Default: Latest
      list.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
    }
    return list;
  }, [filteredOrders, sortBy]);

  // Pagination slice
  const totalPages = Math.ceil(sortedOrders.length / pageSize) || 1;
  const paginatedOrders = useMemo(() => {
    const start = (currentPage - 1) * pageSize;
    return sortedOrders.slice(start, start + pageSize);
  }, [sortedOrders, currentPage, pageSize]);

  // Helper Toast trigger
  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  const updateStatus = async (order: OrderRecord, backendStatus: OrderState, displayStatus: OrderState) => {
    try {
      await dispatch(updateOrderStatusThunk({
        id: order.id,
        status: backendStatus,
        version: order.version,
      })).unwrap();
      dispatch(updateOrderStatus({ id: order.id, status: displayStatus }));
      showToast(`Status updated to ${displayStatus}`);
    } catch (requestError) {
      showToast(`Unable to update order: ${requestError instanceof Error ? requestError.message : 'Backend request failed.'}`);
    }
  };

  const deleteOrder = async (order: OrderRecord) => {
    try {
      await dispatch(deleteOrderThunk(order.id)).unwrap();
      dispatch(removeOrder(order.id));
      showToast(`Order ${order.order_number} deleted`);
    } catch (requestError) {
      showToast(`Unable to delete order: ${requestError instanceof Error ? requestError.message : 'Backend request failed.'}`);
    }
  };

  // Copy Order ID
  const handleCopyId = (orderNum: string, id: string) => {
    navigator.clipboard.writeText(orderNum);
    setCopiedId(id);
    showToast(`Order number ${orderNum} copied to clipboard!`);
    setTimeout(() => setCopiedId(null), 2000);
  };

  // Select all / toggle row selection
  const handleSelectAll = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.checked) {
      setSelectedIds(paginatedOrders.map((o) => o.id));
    } else {
      setSelectedIds([]);
    }
  };

  const handleToggleRow = (id: string) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    );
  };

  // Bulk Delete
  const handleBulkDelete = async () => {
    if (selectedIds.length === 0) return;
    const ids = [...selectedIds];
    const results = await Promise.allSettled(
      ids.map((id) => dispatch(deleteOrderThunk(id)).unwrap())
    );
    const deletedIds = ids.filter((_, index) => results[index].status === 'fulfilled');
    const failedCount = results.length - deletedIds.length;
    if (deletedIds.length > 0) dispatch(removeMultipleOrders(deletedIds));
    setSelectedIds([]);
    showToast(
      failedCount === 0
        ? `Deleted ${deletedIds.length} orders.`
        : `Deleted ${deletedIds.length}; ${failedCount} order(s) could not be deleted.`
    );
  };

  // Add Order Submit
  const handleAddOrderSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newOrderForm.customerName || !newOrderForm.productName || !newOrderForm.amount) {
      showToast('Please fill out all required fields.');
      return;
    }

    const numAmount = parseFloat(newOrderForm.amount);
    if (isNaN(numAmount) || numAmount <= 0) {
      showToast('Please enter a valid order amount.');
      return;
    }

    dispatch(
      addOrder({
        customerName: newOrderForm.customerName,
        productName: newOrderForm.productName,
        sku: newOrderForm.sku || 'SK-45',
        paymentMethod: newOrderForm.paymentMethod,
        amount: numAmount,
        currency: '$',
        status: newOrderForm.status,
      })
    );

    setIsAddModalOpen(false);
    setNewOrderForm({
      customerName: '',
      productName: '',
      sku: 'SK-45',
      paymentMethod: 'Cash on Delivery',
      amount: '',
      status: 'Pending',
    });
    showToast('New Order created successfully!');
  };

  // Status Badge Renderer
  const renderStatusBadge = (status?: string | OrderState) => {
    const s = String(status || 'Pending');
    switch (s) {
      case 'Pending':
        return (
          <Badge variant="outline" className="bg-purple-50 text-purple-700 border-purple-200 text-[10px]">
            Pending
          </Badge>
        );
      case 'Processing':
      case 'Paid':
      case 'Reserved':
        return (
          <Badge variant="outline" className="bg-amber-50 text-amber-700 border-amber-200 text-[10px]">
            {s === 'Paid' ? 'Paid' : s === 'Reserved' ? 'Reserved' : 'Processing'}
          </Badge>
        );
      case 'Delivered':
      case 'Shipped':
        return (
          <Badge variant="outline" className="bg-emerald-50 text-emerald-700 border-emerald-200 text-[10px]">
            {s === 'Shipped' ? 'Shipped' : 'Delivered'}
          </Badge>
        );
      case 'Returned':
        return (
          <Badge variant="destructive" className="text-[10px]">
            Returned
          </Badge>
        );
      case 'Cancelled':
        return (
          <Badge variant="secondary" className="text-[10px]">
            Cancelled
          </Badge>
        );
      default:
        return (
          <Badge variant="secondary" className="text-[10px]">
            {s}
          </Badge>
        );
    }
  };

  return (
    <DashboardLayout activeTab="Orders">
      {error && (
        <Alert variant="destructive" className="mx-auto max-w-7xl">
          <AlertDescription>Unable to load or update orders: {error}</AlertDescription>
        </Alert>
      )}

      {/* Toast Notification */}
      {toastMessage && (
        <div className="fixed bottom-6 right-6 z-50 bg-neutral-900 text-white text-xs font-semibold px-4 py-3 rounded-xl shadow-xl flex items-center space-x-2 animate-in fade-in slide-in-from-bottom-2">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{toastMessage}</span>
        </div>
      )}

      <div className="max-w-7xl mx-auto space-y-6 select-none font-sans">
        {/* Top Header Bar */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <h1 className="text-2xl md:text-3xl font-extrabold text-neutral-900 tracking-tight font-serif">
                Orders
              </h1>
              <Badge
                variant={isSuperAdmin ? 'default' : 'secondary'}
                className={`font-mono text-[10px] uppercase font-bold ${
                  isSuperAdmin ? 'bg-amber-700 text-white' : 'bg-purple-100 text-purple-800 border-purple-200'
                }`}
              >
                {role || 'Staff'}
              </Badge>
            </div>

            {/* Filter Tabs */}
            <div className="flex items-center space-x-1.5 bg-white p-1 rounded-xl border border-neutral-200 shadow-2xs">
              <Button
                variant={activeFilter === 'All Order' ? 'secondary' : 'ghost'}
                size="sm"
                onClick={() => dispatch(setActiveOrderFilter('All Order'))}
                className="h-8 text-xs font-semibold rounded-lg"
              >
                <SlidersHorizontal className="w-3.5 h-3.5 mr-1.5 text-neutral-500" />
                <span>All Order</span>
              </Button>

              <Button
                variant={activeFilter === 'Active' ? 'secondary' : 'ghost'}
                size="sm"
                onClick={() => dispatch(setActiveOrderFilter('Active'))}
                className="h-8 text-xs font-semibold rounded-lg text-purple-700 bg-purple-50"
              >
                <Filter className="w-3.5 h-3.5 mr-1.5 text-purple-600" />
                <span>Active</span>
              </Button>

              <Button
                variant={activeFilter === 'Delivered' ? 'secondary' : 'ghost'}
                size="sm"
                onClick={() => dispatch(setActiveOrderFilter('Delivered'))}
                className="h-8 text-xs font-semibold rounded-lg text-emerald-700 bg-emerald-50"
              >
                <CheckCircle className="w-3.5 h-3.5 mr-1.5 text-emerald-600" />
                <span>Delivered</span>
              </Button>

              <Button
                variant={activeFilter === 'Returned' ? 'secondary' : 'ghost'}
                size="sm"
                onClick={() => dispatch(setActiveOrderFilter('Returned'))}
                className="h-8 text-xs font-semibold rounded-lg text-rose-700 bg-rose-50"
              >
                <RotateCcw className="w-3.5 h-3.5 mr-1.5 text-rose-500" />
                <span>Returned</span>
              </Button>
            </div>
          </div>

          {/* Right Header Controls */}
          <div className="flex items-center space-x-3">
            {isSuperAdmin && selectedIds.length > 0 && (
              <Button
                variant="destructive"
                size="sm"
                onClick={handleBulkDelete}
                className="rounded-xl h-9 text-xs font-bold"
              >
                <Trash2 className="w-3.5 h-3.5 mr-1.5" />
                <span>Delete ({selectedIds.length})</span>
              </Button>
            )}

            <span className="text-xs text-neutral-500 hidden sm:inline">
              Orders created through customer checkout.
            </span>

            {/* Sort Dropdown */}
            <div className="relative">
              <select
                value={sortBy}
                onChange={(e) =>
                  dispatch(
                    setOrderSortBy(
                      e.target.value as
                        | 'Latest'
                        | 'Amount (High-Low)'
                        | 'Amount (Low-High)'
                        | 'Customer Name'
                    )
                  )
                }
                className="appearance-none bg-white border border-neutral-200 text-neutral-700 text-xs font-semibold px-3 py-2 pr-8 rounded-xl cursor-pointer shadow-2xs focus:outline-none focus:ring-2 focus:ring-purple-500/20"
              >
                <option value="Latest">Sort by Latest</option>
                <option value="Amount (High-Low)">Amount: High to Low</option>
                <option value="Amount (Low-High)">Amount: Low to High</option>
                <option value="Customer Name">Customer Name</option>
              </select>
              <ChevronDown className="w-3.5 h-3.5 text-neutral-400 absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            </div>
          </div>
        </div>

        {/* Filter Search input bar */}
        <Card className="rounded-2xl border-neutral-200 shadow-2xs bg-white">
          <CardContent className="p-3 flex items-center justify-between">
            <div className="relative flex-1 max-w-md">
              <Search className="w-4 h-4 text-neutral-400 absolute left-3 top-1/2 -translate-y-1/2" />
              <Input
                type="text"
                value={searchQuery}
                onChange={(e) => dispatch(setOrderSearchQuery(e.target.value))}
                placeholder="Search by Order ID, Customer Name, Product, or SKU..."
                className="pl-9 h-9 text-xs bg-neutral-50 border-neutral-200"
              />
            </div>

            <div className="text-xs font-semibold text-neutral-500">
              Total Results: <span className="text-neutral-900 font-bold">{sortedOrders.length}</span>
            </div>
          </CardContent>
        </Card>

        {/* Orders Table Container */}
        <Card className="rounded-2xl border-neutral-200 shadow-xs overflow-hidden bg-white">
          <Table>
            <TableHeader>
              <TableRow className="border-b border-neutral-100 bg-neutral-50/50 text-[11px] uppercase font-bold text-neutral-400 tracking-wider">
                <TableHead className="py-3.5 px-4 w-10 text-center">
                  <input
                    type="checkbox"
                    onChange={handleSelectAll}
                    checked={
                      paginatedOrders.length > 0 &&
                      selectedIds.length === paginatedOrders.length
                    }
                    className="rounded border-neutral-300 text-purple-600 focus:ring-purple-500 cursor-pointer"
                  />
                </TableHead>
                <TableHead className="py-3.5 px-4">ORDER ID</TableHead>
                <TableHead className="py-3.5 px-4">CUSTOMER NAME</TableHead>
                <TableHead className="py-3.5 px-4">PRODUCT NAME</TableHead>
                <TableHead className="py-3.5 px-4">SKU</TableHead>
                <TableHead className="py-3.5 px-4">PAYMENT</TableHead>
                <TableHead className="py-3.5 px-4">AMOUNT</TableHead>
                <TableHead className="py-3.5 px-4">STATUS</TableHead>
                <TableHead className="py-3.5 px-4 text-right">ACTION</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody className="text-xs font-medium text-neutral-700">
              {paginatedOrders.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={9} className="py-12 text-center text-neutral-400">
                    <div className="flex flex-col items-center justify-center space-y-2">
                      <Package className="w-8 h-8 text-neutral-300" />
                      <span className="font-semibold text-neutral-500">No orders found</span>
                      <span className="text-[11px] text-neutral-400">
                        Try adjusting your active filter or search term.
                      </span>
                    </div>
                  </TableCell>
                </TableRow>
              ) : (
                paginatedOrders.map((order) => {
                  const isSelected = selectedIds.includes(order.id);

                  return (
                    <TableRow
                      key={order.id}
                      className={`hover:bg-purple-50/20 transition-colors ${
                        isSelected ? 'bg-purple-50/50' : ''
                      }`}
                    >
                      {/* Checkbox */}
                      <TableCell className="py-3.5 px-4 text-center">
                        <input
                          type="checkbox"
                          checked={isSelected}
                          onChange={() => handleToggleRow(order.id)}
                          className="rounded border-neutral-300 text-purple-600 focus:ring-purple-500 cursor-pointer"
                        />
                      </TableCell>

                      {/* Order ID with Copy Icon */}
                      <TableCell className="py-3.5 px-4">
                        <div className="flex items-center space-x-1.5 text-neutral-600 font-mono text-[11px]">
                          <span>{order.order_number}</span>
                          <button
                            onClick={() => handleCopyId(order.order_number, order.id)}
                            title="Copy Order ID"
                            className="p-1 hover:bg-neutral-100 rounded text-neutral-400 hover:text-neutral-700 transition-colors cursor-pointer"
                          >
                            {copiedId === order.id ? (
                              <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500" />
                            ) : (
                              <Copy className="w-3.5 h-3.5" />
                            )}
                          </button>
                        </div>
                      </TableCell>

                      {/* Customer Name & Phone */}
                      <TableCell className="py-3.5 px-4">
                        <div className="font-bold text-neutral-900">{order.customer_name}</div>
                        {order.shipping_address?.phone && (
                          <div className="text-[11px] text-neutral-500 font-mono">
                            📞 {order.shipping_address.phone}
                          </div>
                        )}
                      </TableCell>

                      {/* Product Name */}
                      <TableCell className="py-3.5 px-4 text-neutral-700">
                        {order.product_name}
                      </TableCell>

                      {/* SKU */}
                      <TableCell className="py-3.5 px-4 text-neutral-500 font-mono text-[11px]">
                        {order.sku}
                      </TableCell>

                      {/* Payment */}
                      <TableCell className="py-3.5 px-4 text-neutral-600">
                        {order.payment_method}
                      </TableCell>

                      {/* Amount */}
                      <TableCell className="py-3.5 px-4 font-bold text-neutral-900 font-mono">
                        {order.currency}{order.amount.toFixed(2)}
                      </TableCell>

                      {/* Status Badge */}
                      <TableCell className="py-3.5 px-4">
                        {renderStatusBadge(order.status)}
                      </TableCell>

                      {/* Actions DropdownMenu */}
                      <TableCell className="py-3.5 px-4 text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-neutral-400 hover:text-neutral-700">
                              <MoreVertical className="w-4 h-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end" className="w-48 bg-white border-neutral-200">
                            <DropdownMenuItem
                              onClick={() => setInspectOrder(order)}
                              className="text-xs cursor-pointer"
                            >
                              <Eye className="w-3.5 h-3.5 mr-2 text-neutral-500" />
                              <span>View Domain Details</span>
                            </DropdownMenuItem>

                            <DropdownMenuSeparator />
                            <DropdownMenuLabel className="text-[10px] uppercase font-bold text-neutral-400">
                              Change Status
                            </DropdownMenuLabel>

                            <DropdownMenuItem
                              onClick={() => void updateStatus(order, 'Pending', 'Pending')}
                              className="text-xs text-purple-700 cursor-pointer"
                            >
                              <span className="w-2 h-2 rounded-full bg-purple-500 mr-2 shrink-0" />
                              <span>Mark Pending</span>
                            </DropdownMenuItem>

                            <DropdownMenuItem
                              onClick={() => void updateStatus(order, 'Paid', 'Processing')}
                              className="text-xs text-amber-700 cursor-pointer"
                            >
                              <span className="w-2 h-2 rounded-full bg-amber-500 mr-2 shrink-0" />
                              <span>Mark Processing</span>
                            </DropdownMenuItem>

                            <DropdownMenuItem
                              onClick={() => void updateStatus(order, 'Shipped', 'Delivered')}
                              className="text-xs text-emerald-700 cursor-pointer"
                            >
                              <span className="w-2 h-2 rounded-full bg-emerald-500 mr-2 shrink-0" />
                              <span>Mark Delivered</span>
                            </DropdownMenuItem>

                            <DropdownMenuItem
                              onClick={() => void updateStatus(order, 'Cancelled', 'Returned')}
                              className="text-xs text-rose-700 cursor-pointer"
                            >
                              <span className="w-2 h-2 rounded-full bg-rose-500 mr-2 shrink-0" />
                              <span>Mark Returned</span>
                            </DropdownMenuItem>

                            {isSuperAdmin && (
                              <>
                                <DropdownMenuSeparator />
                                <DropdownMenuItem
                                  onClick={() => void deleteOrder(order)}
                                  className="text-xs text-rose-600 font-semibold cursor-pointer"
                                >
                                  <Trash2 className="w-3.5 h-3.5 mr-2 text-rose-500" />
                                  <span>Delete Order</span>
                                </DropdownMenuItem>
                              </>
                            )}
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  );
                })
              )}
            </TableBody>
          </Table>

          {/* Footer Pagination Bar */}
          <div className="px-6 py-4 border-t border-neutral-100 bg-neutral-50/40 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs font-semibold text-neutral-500">
            <div>
              Showing {sortedOrders.length > 0 ? (currentPage - 1) * pageSize + 1 : 0} -{' '}
              {Math.min(currentPage * pageSize, sortedOrders.length)} of {sortedOrders.length} orders
            </div>

            {/* Pagination buttons */}
            <div className="flex items-center space-x-2">
              <Button
                variant="outline"
                size="sm"
                disabled={currentPage === 1}
                onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                className="h-8 rounded-xl text-xs"
              >
                <ChevronLeft className="w-3.5 h-3.5 mr-1" />
                <span>Previous</span>
              </Button>

              {Array.from({ length: totalPages }, (_, i) => i + 1).map((pg) => (
                <Button
                  key={pg}
                  variant={currentPage === pg ? 'primary' : 'outline'}
                  size="sm"
                  onClick={() => setCurrentPage(pg)}
                  className="w-8 h-8 p-0 rounded-xl font-bold text-xs"
                >
                  {pg}
                </Button>
              ))}

              <Button
                variant="outline"
                size="sm"
                disabled={currentPage >= totalPages}
                onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
                className="h-8 rounded-xl text-xs"
              >
                <span>Next</span>
                <ChevronRight className="w-3.5 h-3.5 ml-1" />
              </Button>
            </div>
          </div>
        </Card>
      </div>

      {/* Add New Order Modal */}
      {isAddModalOpen && (
        <Dialog open={true} onOpenChange={(open) => !open && setIsAddModalOpen(false)}>
          <DialogContent className="max-w-md bg-white border-neutral-200 rounded-2xl">
            <DialogHeader>
              <DialogTitle className="font-extrabold text-base text-neutral-900 font-serif">
                Create New Order
              </DialogTitle>
              <DialogDescription className="text-xs text-neutral-500">
                Manually record a bespoke or phone order into PostgreSQL.
              </DialogDescription>
            </DialogHeader>

            <form onSubmit={handleAddOrderSubmit} className="space-y-4 text-xs font-semibold pt-2">
              <div className="space-y-1.5">
                <Label htmlFor="add-cust-name" className="text-neutral-700 text-xs">Customer Name *</Label>
                <Input
                  id="add-cust-name"
                  type="text"
                  required
                  placeholder="e.g. Jane Cooper"
                  value={newOrderForm.customerName}
                  onChange={(e) =>
                    setNewOrderForm({ ...newOrderForm, customerName: e.target.value })
                  }
                  className="h-9 bg-neutral-50"
                />
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="add-prod-name" className="text-neutral-700 text-xs">Product Name *</Label>
                <Input
                  id="add-prod-name"
                  type="text"
                  required
                  placeholder="e.g. Noir Silk Crepe Abaya"
                  value={newOrderForm.productName}
                  onChange={(e) =>
                    setNewOrderForm({ ...newOrderForm, productName: e.target.value })
                  }
                  className="h-9 bg-neutral-50"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label htmlFor="add-sku" className="text-neutral-700 text-xs">SKU Code</Label>
                  <Input
                    id="add-sku"
                    type="text"
                    placeholder="SK-45"
                    value={newOrderForm.sku}
                    onChange={(e) => setNewOrderForm({ ...newOrderForm, sku: e.target.value })}
                    className="h-9 bg-neutral-50 font-mono"
                  />
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="add-amount" className="text-neutral-700 text-xs">Amount ($) *</Label>
                  <Input
                    id="add-amount"
                    type="number"
                    step="0.01"
                    required
                    placeholder="110.21"
                    value={newOrderForm.amount}
                    onChange={(e) =>
                      setNewOrderForm({ ...newOrderForm, amount: e.target.value })
                    }
                    className="h-9 bg-neutral-50 font-mono"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label htmlFor="add-payment" className="text-neutral-700 text-xs">Payment Method</Label>
                  <select
                    id="add-payment"
                    value={newOrderForm.paymentMethod}
                    onChange={(e) =>
                      setNewOrderForm({
                        ...newOrderForm,
                        paymentMethod: e.target.value as any,
                      })
                    }
                    className="w-full h-9 px-3 bg-neutral-50 border border-neutral-200 rounded-lg text-xs"
                  >
                    <option value="Cash on Delivery">Cash on Delivery</option>
                    <option value="Paid via PayPal">Paid via PayPal</option>
                    <option value="Credit Card">Credit Card</option>
                    <option value="Apple Pay">Apple Pay</option>
                  </select>
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="add-status" className="text-neutral-700 text-xs">Order Status</Label>
                  <select
                    id="add-status"
                    value={newOrderForm.status}
                    onChange={(e) =>
                      setNewOrderForm({ ...newOrderForm, status: e.target.value as OrderState })
                    }
                    className="w-full h-9 px-3 bg-neutral-50 border border-neutral-200 rounded-lg text-xs"
                  >
                    <option value="Pending">Pending</option>
                    <option value="Processing">Processing</option>
                    <option value="Delivered">Delivered</option>
                    <option value="Returned">Returned</option>
                  </select>
                </div>
              </div>

              <DialogFooter className="pt-3 border-t border-neutral-100 flex items-center justify-end space-x-2">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => setIsAddModalOpen(false)}
                  className="rounded-xl h-9"
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  variant="primary"
                  size="sm"
                  className="rounded-xl h-9 font-bold"
                >
                  Create Order
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      )}

      {/* Inspect Domain Order Modal */}
      {inspectOrder && (
        <Dialog open={true} onOpenChange={(open) => !open && setInspectOrder(null)}>
          <DialogContent className="max-w-lg bg-white border-neutral-200 rounded-2xl">
            <DialogHeader>
              <div className="flex items-center space-x-2">
                <Shield className="w-4 h-4 text-amber-700" />
                <DialogTitle className="font-extrabold text-base text-neutral-900 font-serif">
                  Domain Order Specification ({inspectOrder.order_number})
                </DialogTitle>
              </div>
              <DialogDescription className="text-xs text-neutral-500">
                PostgreSQL domain entity snapshot with OCC version and idempotency metadata.
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 text-xs font-medium text-neutral-700 pt-2">
              <div className="p-3 bg-purple-50/50 rounded-xl border border-purple-100 space-y-1">
                <div className="flex items-center justify-between text-neutral-500 font-mono text-[11px]">
                  <span>Backend Internal UUID:</span>
                  <span className="font-bold text-neutral-800">{inspectOrder.id}</span>
                </div>
                <div className="flex items-center justify-between text-neutral-500 font-mono text-[11px]">
                  <span>Idempotency Key:</span>
                  <span className="text-purple-700">{inspectOrder.idempotency_key || 'N/A'}</span>
                </div>
                <div className="flex items-center justify-between text-neutral-500 font-mono text-[11px]">
                  <span>Optimistic Lock (OCC Version):</span>
                  <span className="font-bold text-neutral-900">v{inspectOrder.version || 1}</span>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <div className="text-neutral-400 text-[10px] uppercase font-bold tracking-wider mb-1">
                    Customer & Contact Phone
                  </div>
                  <div className="font-bold text-neutral-900 flex items-center space-x-1">
                    <User className="w-3.5 h-3.5 text-neutral-400" />
                    <span>{inspectOrder.customer_name}</span>
                  </div>
                  <div className="text-neutral-600 font-mono text-[11px] mt-0.5">
                    📞 {inspectOrder.shipping_address?.phone || '+966 50 123 4567'}
                  </div>
                </div>

                <div>
                  <div className="text-neutral-400 text-[10px] uppercase font-bold tracking-wider mb-1">
                    Payment & Amount
                  </div>
                  <div className="font-bold text-neutral-900 flex items-center space-x-1">
                    <CreditCard className="w-3.5 h-3.5 text-neutral-400" />
                    <span>
                      {inspectOrder.currency}
                      {inspectOrder.amount.toFixed(2)} ({inspectOrder.payment_method})
                    </span>
                  </div>
                </div>
              </div>

              <div>
                <div className="text-neutral-400 text-[10px] uppercase font-bold tracking-wider mb-1">
                  Product Item Breakdown
                </div>
                <div className="p-3 bg-neutral-50 rounded-xl border border-neutral-200 flex items-center justify-between">
                  <div>
                    <div className="font-bold text-neutral-900">{inspectOrder.product_name}</div>
                    <div className="text-neutral-500 font-mono text-[11px]">SKU: {inspectOrder.sku}</div>
                  </div>
                  {renderStatusBadge(inspectOrder.status)}
                </div>
              </div>

              {inspectOrder.shipping_address && (
                <div>
                  <div className="text-neutral-400 text-[10px] uppercase font-bold tracking-wider mb-1">
                    Shipping Address & Phone
                  </div>
                  <div className="p-3 bg-neutral-50 rounded-xl border border-neutral-200 space-y-1">
                    <div className="font-bold text-neutral-900 flex items-center justify-between">
                      <div className="flex items-center space-x-1">
                        <MapPin className="w-3.5 h-3.5 text-purple-600" />
                        <span>{inspectOrder.shipping_address.recipient_name}</span>
                      </div>
                      <span className="font-mono text-[11px] text-neutral-600">
                        📞 {inspectOrder.shipping_address.phone}
                      </span>
                    </div>
                    <div className="text-neutral-600">{inspectOrder.shipping_address.street_line1}</div>
                    <div className="text-neutral-600">
                      {inspectOrder.shipping_address.city}, {inspectOrder.shipping_address.state}{' '}
                      {inspectOrder.shipping_address.postal_code}, {inspectOrder.shipping_address.country}
                    </div>
                    {inspectOrder.shipping_address.delivery_instructions && (
                      <div className="text-[11px] text-amber-700 bg-amber-50 p-2 rounded-lg border border-amber-200 mt-1">
                        Note: {inspectOrder.shipping_address.delivery_instructions}
                      </div>
                    )}
                  </div>
                </div>
              )}

              <DialogFooter className="pt-2 text-right">
                <Button
                  variant="primary"
                  size="sm"
                  onClick={() => setInspectOrder(null)}
                  className="rounded-xl h-9 font-bold"
                >
                  Close Specification
                </Button>
              </DialogFooter>
            </div>
          </DialogContent>
        </Dialog>
      )}
    </DashboardLayout>
  );
};

export default OrdersPage;
