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
import {
  Copy,
  CheckCircle2,
  Filter,
  CheckCircle,
  RotateCcw,
  SlidersHorizontal,
  Plus,
  ChevronDown,
  MoreVertical,
  ChevronLeft,
  ChevronRight,
  Search,
  Eye,
  Trash2,
  X,
  Package,
  MapPin,
  Clock,
  Shield,
  CreditCard,
  DollarSign,
  User,
} from 'lucide-react';

export const OrdersPage: React.FC = () => {
  const dispatch = useDispatch<AppDispatch>();
  const { orders, activeFilter, searchQuery, sortBy } = useSelector(
    (state: RootState) => state.orders
  );

  useEffect(() => {
    dispatch(fetchOrders());
  }, [dispatch]);

  // Local UI state
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [activeMenuId, setActiveMenuId] = useState<string | null>(null);
  
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
  const handleBulkDelete = () => {
    if (selectedIds.length === 0) return;
    dispatch(removeMultipleOrders(selectedIds));
    showToast(`Successfully removed ${selectedIds.length} orders.`);
    setSelectedIds([]);
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
          <span className="inline-flex items-center px-3 py-1 rounded-full text-[11px] font-semibold bg-[#f3e8ff] text-[#9333ea] border border-[#e9d5ff]">
            Pending
          </span>
        );
      case 'Processing':
      case 'Paid':
      case 'Reserved':
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-[11px] font-semibold bg-[#fef3c7] text-[#d97706] border border-[#fde68a]">
            {s === 'Paid' ? 'Paid' : s === 'Reserved' ? 'Reserved' : 'Processing'}
          </span>
        );
      case 'Delivered':
      case 'Shipped':
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-[11px] font-semibold bg-[#dcfce7] text-[#16a34a] border border-[#bbf7d0]">
            {s === 'Shipped' ? 'Shipped' : 'Delivered'}
          </span>
        );
      case 'Returned':
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-[11px] font-semibold bg-[#fee2e2] text-[#dc2626] border border-[#fecaca]">
            Returned
          </span>
        );
      case 'Cancelled':
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-[11px] font-semibold bg-gray-100 text-gray-700 border border-gray-200">
            Cancelled
          </span>
        );
      default:
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-[11px] font-semibold bg-gray-100 text-gray-700 border border-gray-200">
            {s}
          </span>
        );
    }
  };

  return (
    <DashboardLayout activeTab="Orders">
      {/* Toast Notification */}
      {toastMessage && (
        <div className="fixed bottom-6 right-6 z-50 bg-[#1e1b4b] text-white text-xs font-semibold px-4 py-3 rounded-xl shadow-xl flex items-center space-x-2 animate-in fade-in slide-in-from-bottom-2">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{toastMessage}</span>
        </div>
      )}

      <div className="max-w-7xl mx-auto space-y-6 select-none font-sans">
        {/* Top Header Bar */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center space-x-4">
            <h1 className="text-2xl md:text-3xl font-extrabold text-gray-900 tracking-tight">
              Orders
            </h1>

            {/* Filter Tabs matching Screenshot */}
            <div className="flex items-center space-x-1.5 bg-white p-1 rounded-xl border border-gray-200/80 shadow-2xs">
              {/* All Order Tab */}
              <button
                onClick={() => dispatch(setActiveOrderFilter('All Order'))}
                className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                  activeFilter === 'All Order'
                    ? 'bg-gray-100 text-gray-900 shadow-2xs'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
                }`}
              >
                <SlidersHorizontal className="w-3.5 h-3.5 text-gray-500" />
                <span>All Order</span>
              </button>

              {/* Active Tab (Highlighted in soft purple box as in screenshot) */}
              <button
                onClick={() => dispatch(setActiveOrderFilter('Active'))}
                className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                  activeFilter === 'Active'
                    ? 'bg-[#ede9fe] text-[#7c3aed] border border-[#ddd6fe] shadow-2xs'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
                }`}
              >
                <Filter className="w-3.5 h-3.5 text-[#7c3aed]" />
                <span>Active</span>
              </button>

              {/* Delivered Tab */}
              <button
                onClick={() => dispatch(setActiveOrderFilter('Delivered'))}
                className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                  activeFilter === 'Delivered'
                    ? 'bg-emerald-50 text-emerald-700 border border-emerald-200 shadow-2xs'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
                }`}
              >
                <CheckCircle className="w-3.5 h-3.5 text-emerald-600" />
                <span>Delivered</span>
              </button>

              {/* Returned Tab */}
              <button
                onClick={() => dispatch(setActiveOrderFilter('Returned'))}
                className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                  activeFilter === 'Returned'
                    ? 'bg-rose-50 text-rose-700 border border-rose-200 shadow-2xs'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
                }`}
              >
                <RotateCcw className="w-3.5 h-3.5 text-rose-500" />
                <span>Returned</span>
              </button>
            </div>
          </div>

          {/* Right Header Controls */}
          <div className="flex items-center space-x-3">
            {/* Bulk Actions if items selected */}
            {selectedIds.length > 0 && (
              <button
                onClick={handleBulkDelete}
                className="bg-rose-600 hover:bg-rose-700 text-white px-3 py-2 rounded-xl text-xs font-bold flex items-center space-x-1.5 shadow-2xs transition-all cursor-pointer"
              >
                <Trash2 className="w-3.5 h-3.5" />
                <span>Delete ({selectedIds.length})</span>
              </button>
            )}

            {/* + Add New Order Button */}
            <button
              onClick={() => setIsAddModalOpen(true)}
              className="bg-[#7c3aed] hover:bg-[#6d28d9] text-white px-4 py-2 rounded-xl text-xs font-bold flex items-center space-x-1.5 shadow-sm transition-all cursor-pointer"
            >
              <Plus className="w-4 h-4" />
              <span>Add New Order</span>
            </button>

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
                className="appearance-none bg-white border border-gray-200 text-gray-700 text-xs font-semibold px-3 py-2 pr-8 rounded-xl cursor-pointer shadow-2xs focus:outline-none focus:ring-2 focus:ring-[#7c3aed]/20"
              >
                <option value="Latest">Sort by Latest</option>
                <option value="Amount (High-Low)">Amount: High to Low</option>
                <option value="Amount (Low-High)">Amount: Low to High</option>
                <option value="Customer Name">Customer Name</option>
              </select>
              <ChevronDown className="w-3.5 h-3.5 text-gray-400 absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            </div>
          </div>
        </div>

        {/* Filter Search input bar */}
        <div className="flex items-center justify-between bg-white p-3 rounded-2xl border border-gray-200/80 shadow-2xs">
          <div className="relative flex-1 max-w-md">
            <Search className="w-4 h-4 text-gray-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => dispatch(setOrderSearchQuery(e.target.value))}
              placeholder="Search by Order ID, Customer Name, Product, or SKU..."
              className="w-full pl-9 pr-4 py-1.5 text-xs bg-gray-50 border border-gray-200 rounded-xl text-gray-800 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-[#7c3aed]/20 focus:border-[#7c3aed]"
            />
          </div>

          <div className="text-xs font-semibold text-gray-500">
            Total Results: <span className="text-gray-900 font-bold">{sortedOrders.length}</span>
          </div>
        </div>

        {/* Orders Table Container */}
        <div className="bg-white rounded-2xl border border-gray-200/80 shadow-xs overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="border-b border-gray-100 bg-gray-50/50 text-[11px] uppercase font-bold text-gray-400 tracking-wider">
                  <th className="py-3.5 px-4 w-10 text-center">
                    <input
                      type="checkbox"
                      onChange={handleSelectAll}
                      checked={
                        paginatedOrders.length > 0 &&
                        selectedIds.length === paginatedOrders.length
                      }
                      className="rounded border-gray-300 text-[#7c3aed] focus:ring-[#7c3aed] cursor-pointer"
                    />
                  </th>
                  <th className="py-3.5 px-4">ORDER ID</th>
                  <th className="py-3.5 px-4">CUSTOMER NAME</th>
                  <th className="py-3.5 px-4">PRODUCT NAME</th>
                  <th className="py-3.5 px-4">SKU</th>
                  <th className="py-3.5 px-4">PAYMENT</th>
                  <th className="py-3.5 px-4">AMOUNT</th>
                  <th className="py-3.5 px-4">STATUS</th>
                  <th className="py-3.5 px-4 text-right">ACTION</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100 text-xs font-medium text-gray-700">
                {paginatedOrders.length === 0 ? (
                  <tr>
                    <td colSpan={9} className="py-12 text-center text-gray-400">
                      <div className="flex flex-col items-center justify-center space-y-2">
                        <Package className="w-8 h-8 text-gray-300" />
                        <span className="font-semibold text-gray-500">No orders found</span>
                        <span className="text-[11px] text-gray-400">
                          Try adjusting your active filter or search term.
                        </span>
                      </div>
                    </td>
                  </tr>
                ) : (
                  paginatedOrders.map((order) => {
                    const isSelected = selectedIds.includes(order.id);
                    const isMenuOpen = activeMenuId === order.id;

                    return (
                      <tr
                        key={order.id}
                        className={`hover:bg-purple-50/30 transition-colors ${
                          isSelected ? 'bg-purple-50/50' : ''
                        }`}
                      >
                        {/* Checkbox */}
                        <td className="py-3.5 px-4 text-center">
                          <input
                            type="checkbox"
                            checked={isSelected}
                            onChange={() => handleToggleRow(order.id)}
                            className="rounded border-gray-300 text-[#7c3aed] focus:ring-[#7c3aed] cursor-pointer"
                          />
                        </td>

                        {/* Order ID with Copy Icon */}
                        <td className="py-3.5 px-4">
                          <div className="flex items-center space-x-1.5 text-gray-600 font-mono text-[11px]">
                            <span>{order.order_number}</span>
                            <button
                              onClick={() => handleCopyId(order.order_number, order.id)}
                              title="Copy Order ID"
                              className="p-1 hover:bg-gray-100 rounded text-gray-400 hover:text-gray-700 transition-colors cursor-pointer"
                            >
                              {copiedId === order.id ? (
                                <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500" />
                              ) : (
                                <Copy className="w-3.5 h-3.5" />
                              )}
                            </button>
                          </div>
                        </td>

                        {/* Customer Name & Phone */}
                        <td className="py-3.5 px-4">
                          <div className="font-bold text-gray-900">{order.customer_name}</div>
                          {order.shipping_address?.phone && (
                            <div className="text-[11px] text-gray-500 font-mono">
                              📞 {order.shipping_address.phone}
                            </div>
                          )}
                        </td>

                        {/* Product Name */}
                        <td className="py-3.5 px-4 text-gray-700">
                          {order.product_name}
                        </td>

                        {/* SKU */}
                        <td className="py-3.5 px-4 text-gray-500 font-mono text-[11px]">
                          {order.sku}
                        </td>

                        {/* Payment */}
                        <td className="py-3.5 px-4 text-gray-600">
                          {order.payment_method}
                        </td>

                        {/* Amount */}
                        <td className="py-3.5 px-4 font-bold text-gray-900">
                          {order.currency}{order.amount.toFixed(2)}
                        </td>

                        {/* Status Badge */}
                        <td className="py-3.5 px-4">
                          {renderStatusBadge(order.status)}
                        </td>

                        {/* Actions Menu */}
                        <td className="py-3.5 px-4 text-right relative">
                          <button
                            onClick={() =>
                              setActiveMenuId(isMenuOpen ? null : order.id)
                            }
                            className="p-1.5 hover:bg-gray-100 rounded-lg text-gray-400 hover:text-gray-700 transition-colors cursor-pointer"
                          >
                            <MoreVertical className="w-4 h-4" />
                          </button>

                          {/* Dropdown Menu */}
                          {isMenuOpen && (
                            <div className="absolute right-4 top-10 z-40 w-48 bg-white rounded-xl shadow-xl border border-gray-100 py-1.5 text-xs text-left animate-in fade-in zoom-in-95">
                              <button
                                onClick={() => {
                                  setInspectOrder(order);
                                  setActiveMenuId(null);
                                }}
                                className="w-full px-3.5 py-2 hover:bg-gray-50 flex items-center space-x-2 text-gray-700 font-medium cursor-pointer"
                              >
                                <Eye className="w-3.5 h-3.5 text-gray-400" />
                                <span>View Domain Details</span>
                              </button>

                              <div className="my-1 border-t border-gray-100" />
                              <div className="px-3 py-1 text-[10px] uppercase font-bold text-gray-400">
                                Change Status
                              </div>

                              <button
                                onClick={() => {
                                  dispatch(updateOrderStatusThunk({ id: order.id, status: 'Pending', version: order.version }));
                                  dispatch(updateOrderStatus({ id: order.id, status: 'Pending' }));
                                  setActiveMenuId(null);
                                  showToast(`Status updated to Pending`);
                                }}
                                className="w-full px-3.5 py-1.5 hover:bg-purple-50 flex items-center space-x-2 text-purple-700 cursor-pointer text-left text-xs font-medium"
                              >
                                <span className="w-2 h-2 rounded-full bg-purple-500 shrink-0" />
                                <span>Mark Pending</span>
                              </button>

                              <button
                                onClick={() => {
                                  dispatch(updateOrderStatusThunk({ id: order.id, status: 'Paid', version: order.version }));
                                  dispatch(updateOrderStatus({ id: order.id, status: 'Processing' }));
                                  setActiveMenuId(null);
                                  showToast(`Status updated to Processing`);
                                }}
                                className="w-full px-3.5 py-1.5 hover:bg-amber-50 flex items-center space-x-2 text-amber-700 cursor-pointer text-left text-xs font-medium"
                              >
                                <span className="w-2 h-2 rounded-full bg-amber-500 shrink-0" />
                                <span>Mark Processing</span>
                              </button>

                              <button
                                onClick={() => {
                                  dispatch(updateOrderStatusThunk({ id: order.id, status: 'Shipped', version: order.version }));
                                  dispatch(updateOrderStatus({ id: order.id, status: 'Delivered' }));
                                  setActiveMenuId(null);
                                  showToast(`Status updated to Delivered`);
                                }}
                                className="w-full px-3.5 py-1.5 hover:bg-emerald-50 flex items-center space-x-2 text-emerald-700 cursor-pointer text-left text-xs font-medium"
                              >
                                <span className="w-2 h-2 rounded-full bg-emerald-500 shrink-0" />
                                <span>Mark Delivered</span>
                              </button>

                              <button
                                onClick={() => {
                                  dispatch(updateOrderStatusThunk({ id: order.id, status: 'Cancelled', version: order.version }));
                                  dispatch(updateOrderStatus({ id: order.id, status: 'Returned' }));
                                  setActiveMenuId(null);
                                  showToast(`Status updated to Returned`);
                                }}
                                className="w-full px-3.5 py-1.5 hover:bg-rose-50 flex items-center space-x-2 text-rose-700 cursor-pointer text-left text-xs font-medium"
                              >
                                <span className="w-2 h-2 rounded-full bg-rose-500 shrink-0" />
                                <span>Mark Returned</span>
                              </button>

                              <div className="my-1 border-t border-gray-100" />

                              <button
                                onClick={() => {
                                  dispatch(deleteOrderThunk(order.id));
                                  dispatch(removeOrder(order.id));
                                  setActiveMenuId(null);
                                  showToast(`Order ${order.order_number} deleted`);
                                }}
                                className="w-full px-3.5 py-2 hover:bg-rose-50 flex items-center space-x-2 text-rose-600 font-semibold cursor-pointer text-left text-xs"
                              >
                                <Trash2 className="w-3.5 h-3.5 text-rose-500 shrink-0" />
                                <span>Delete Order</span>
                              </button>
                            </div>
                          )}
                        </td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>

          {/* Footer Pagination Bar matching Screenshot */}
          <div className="px-6 py-4 border-t border-gray-100 bg-gray-50/40 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs font-semibold text-gray-500">
            <div>
              Showing {sortedOrders.length > 0 ? (currentPage - 1) * pageSize + 1 : 0} -{' '}
              {Math.min(currentPage * pageSize, sortedOrders.length)} of {40} products
            </div>

            {/* Pagination buttons */}
            <div className="flex items-center space-x-2">
              <button
                disabled={currentPage === 1}
                onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                className="px-3 py-1.5 rounded-xl border border-gray-200 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed flex items-center space-x-1 cursor-pointer transition-all shadow-2xs"
              >
                <ChevronLeft className="w-3.5 h-3.5" />
                <span>Previous</span>
              </button>

              {Array.from({ length: totalPages }, (_, i) => i + 1).map((pg) => (
                <button
                  key={pg}
                  onClick={() => setCurrentPage(pg)}
                  className={`w-7 h-7 rounded-xl font-bold transition-all cursor-pointer ${
                    currentPage === pg
                      ? 'bg-[#7c3aed] text-white shadow-2xs'
                      : 'bg-white border border-gray-200 text-gray-700 hover:bg-gray-50'
                  }`}
                >
                  {pg}
                </button>
              ))}

              <button
                disabled={currentPage >= totalPages}
                onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
                className="px-3 py-1.5 rounded-xl border border-gray-200 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed flex items-center space-x-1 cursor-pointer transition-all shadow-2xs"
              >
                <span>Next</span>
                <ChevronRight className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Add New Order Modal */}
      {isAddModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="fixed inset-0 bg-black/40 backdrop-blur-xs"
            onClick={() => setIsAddModalOpen(false)}
          />
          <div className="relative z-10 bg-white rounded-2xl shadow-2xl border border-gray-200 w-full max-w-md overflow-hidden animate-in fade-in zoom-in-95">
            <div className="px-6 py-4 bg-gray-50 border-b border-gray-100 flex items-center justify-between">
              <h3 className="font-extrabold text-base text-gray-900 flex items-center space-x-2">
                <Plus className="w-4 h-4 text-[#7c3aed]" />
                <span>Create New Order</span>
              </h3>
              <button
                onClick={() => setIsAddModalOpen(false)}
                className="p-1 rounded-lg text-gray-400 hover:text-gray-700 hover:bg-gray-200/50 cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleAddOrderSubmit} className="p-6 space-y-4 text-xs font-semibold">
              <div>
                <label className="block text-gray-700 mb-1">Customer Name *</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. Jane Cooper"
                  value={newOrderForm.customerName}
                  onChange={(e) =>
                    setNewOrderForm({ ...newOrderForm, customerName: e.target.value })
                  }
                  className="w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-[#7c3aed]/20 focus:border-[#7c3aed]"
                />
              </div>

              <div>
                <label className="block text-gray-700 mb-1">Product Name *</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. Nike Men's Pullover"
                  value={newOrderForm.productName}
                  onChange={(e) =>
                    setNewOrderForm({ ...newOrderForm, productName: e.target.value })
                  }
                  className="w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-[#7c3aed]/20 focus:border-[#7c3aed]"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-gray-700 mb-1">SKU Code</label>
                  <input
                    type="text"
                    placeholder="SK-45"
                    value={newOrderForm.sku}
                    onChange={(e) => setNewOrderForm({ ...newOrderForm, sku: e.target.value })}
                    className="w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-[#7c3aed]/20 focus:border-[#7c3aed]"
                  />
                </div>

                <div>
                  <label className="block text-gray-700 mb-1">Amount ($) *</label>
                  <input
                    type="number"
                    step="0.01"
                    required
                    placeholder="110.21"
                    value={newOrderForm.amount}
                    onChange={(e) =>
                      setNewOrderForm({ ...newOrderForm, amount: e.target.value })
                    }
                    className="w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-[#7c3aed]/20 focus:border-[#7c3aed]"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-gray-700 mb-1">Payment Method</label>
                  <select
                    value={newOrderForm.paymentMethod}
                    onChange={(e) =>
                      setNewOrderForm({
                        ...newOrderForm,
                        paymentMethod: e.target.value as any,
                      })
                    }
                    className="w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-[#7c3aed]/20 focus:border-[#7c3aed]"
                  >
                    <option value="Cash on Delivery">Cash on Delivery</option>
                    <option value="Paid via PayPal">Paid via PayPal</option>
                    <option value="Credit Card">Credit Card</option>
                    <option value="Apple Pay">Apple Pay</option>
                  </select>
                </div>

                <div>
                  <label className="block text-gray-700 mb-1">Order Status</label>
                  <select
                    value={newOrderForm.status}
                    onChange={(e) =>
                      setNewOrderForm({ ...newOrderForm, status: e.target.value as OrderState })
                    }
                    className="w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-[#7c3aed]/20 focus:border-[#7c3aed]"
                  >
                    <option value="Pending">Pending</option>
                    <option value="Processing">Processing</option>
                    <option value="Delivered">Delivered</option>
                    <option value="Returned">Returned</option>
                  </select>
                </div>
              </div>

              <div className="pt-4 border-t border-gray-100 flex items-center justify-end space-x-2">
                <button
                  type="button"
                  onClick={() => setIsAddModalOpen(false)}
                  className="px-4 py-2 rounded-xl text-gray-600 hover:bg-gray-100 font-semibold cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-[#7c3aed] hover:bg-[#6d28d9] text-white font-bold rounded-xl shadow-sm cursor-pointer"
                >
                  Create Order
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Inspect Domain Order Modal */}
      {inspectOrder && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="fixed inset-0 bg-black/40 backdrop-blur-xs"
            onClick={() => setInspectOrder(null)}
          />
          <div className="relative z-10 bg-white rounded-2xl shadow-2xl border border-gray-200 w-full max-w-lg overflow-hidden animate-in fade-in zoom-in-95">
            <div className="px-6 py-4 bg-gray-50 border-b border-gray-100 flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <Shield className="w-4 h-4 text-[#7c3aed]" />
                <h3 className="font-extrabold text-base text-gray-900">
                  Domain Order Specification ({inspectOrder.order_number})
                </h3>
              </div>
              <button
                onClick={() => setInspectOrder(null)}
                className="p-1 rounded-lg text-gray-400 hover:text-gray-700 hover:bg-gray-200/50 cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="p-6 space-y-4 text-xs font-medium text-gray-700">
              <div className="p-3 bg-purple-50/50 rounded-xl border border-purple-100 space-y-1">
                <div className="flex items-center justify-between text-gray-500 font-mono text-[11px]">
                  <span>Backend Internal UUID:</span>
                  <span className="font-bold text-gray-800">{inspectOrder.id}</span>
                </div>
                <div className="flex items-center justify-between text-gray-500 font-mono text-[11px]">
                  <span>Idempotency Key:</span>
                  <span className="text-purple-700">{inspectOrder.idempotency_key || 'N/A'}</span>
                </div>
                <div className="flex items-center justify-between text-gray-500 font-mono text-[11px]">
                  <span>Optimistic Lock (OCC Version):</span>
                  <span className="font-bold text-gray-900">v{inspectOrder.version || 1}</span>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <div className="text-gray-400 text-[10px] uppercase font-bold tracking-wider mb-1">
                    Customer & Contact Phone
                  </div>
                  <div className="font-bold text-gray-900 flex items-center space-x-1">
                    <User className="w-3.5 h-3.5 text-gray-400" />
                    <span>{inspectOrder.customer_name}</span>
                  </div>
                  <div className="text-gray-600 font-mono text-[11px] mt-0.5">
                    📞 {inspectOrder.shipping_address?.phone || '+966 50 123 4567'}
                  </div>
                </div>

                <div>
                  <div className="text-gray-400 text-[10px] uppercase font-bold tracking-wider mb-1">
                    Payment & Amount
                  </div>
                  <div className="font-bold text-gray-900 flex items-center space-x-1">
                    <CreditCard className="w-3.5 h-3.5 text-gray-400" />
                    <span>
                      {inspectOrder.currency}
                      {inspectOrder.amount.toFixed(2)} ({inspectOrder.payment_method})
                    </span>
                  </div>
                </div>
              </div>

              <div>
                <div className="text-gray-400 text-[10px] uppercase font-bold tracking-wider mb-1">
                  Product Item Breakdown
                </div>
                <div className="p-3 bg-gray-50 rounded-xl border border-gray-200 flex items-center justify-between">
                  <div>
                    <div className="font-bold text-gray-900">{inspectOrder.product_name}</div>
                    <div className="text-gray-500 font-mono text-[11px]">SKU: {inspectOrder.sku}</div>
                  </div>
                  {renderStatusBadge(inspectOrder.status)}
                </div>
              </div>

              {inspectOrder.shipping_address && (
                <div>
                  <div className="text-gray-400 text-[10px] uppercase font-bold tracking-wider mb-1">
                    Shipping Address & Phone
                  </div>
                  <div className="p-3 bg-gray-50 rounded-xl border border-gray-200 space-y-1">
                    <div className="font-bold text-gray-900 flex items-center justify-between">
                      <div className="flex items-center space-x-1">
                        <MapPin className="w-3.5 h-3.5 text-[#7c3aed]" />
                        <span>{inspectOrder.shipping_address.recipient_name}</span>
                      </div>
                      <span className="font-mono text-[11px] text-gray-600">
                        📞 {inspectOrder.shipping_address.phone}
                      </span>
                    </div>
                    <div className="text-gray-600">{inspectOrder.shipping_address.street_line1}</div>
                    <div className="text-gray-600">
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

              <div className="pt-2 text-right">
                <button
                  onClick={() => setInspectOrder(null)}
                  className="px-4 py-2 bg-[#7c3aed] text-white font-bold rounded-xl cursor-pointer"
                >
                  Close Specification
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </DashboardLayout>
  );
};

export default OrdersPage;
