import React, { useEffect, useState, useMemo } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { useNavigate } from 'react-router-dom';
import type { AppDispatch, RootState } from '@/slicers/root_store';
import {
  fetchOrders,
  updateOrderStatusThunk,
  updateOrderStatus,
} from '@/slicers/order_slicer/order_slice';
import type { OrderRecord, OrderState } from '@/shared/types/contracts';
import {
  ShieldCheck,
  Search,
  MessageSquare,
  Send,
  Eye,
  CheckCircle2,
  Clock,
  PackageCheck,
  Phone,
  MapPin,
  User,
  ShoppingBag,
  ArrowLeft,
  RefreshCw,
  X,
  MessageCircle,
  Truck,
  RotateCcw,
  Sparkles,
} from 'lucide-react';

interface ChatMessage {
  id: string;
  sender: 'moderator' | 'customer';
  text: string;
  timestamp: string;
}

export const ModeratorPanel: React.FC = () => {
  const navigate = useNavigate();
  const dispatch = useDispatch<AppDispatch>();

  const { orders, loading } = useSelector((state: RootState) => state.orders);

  const [activeFilter, setActiveFilter] = useState<'All' | 'Pending' | 'Processing' | 'Delivered' | 'Returned'>('All');
  const [searchQuery, setSearchQuery] = useState('');
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // Inspector & Chat Modals
  const [inspectOrder, setInspectOrder] = useState<OrderRecord | null>(null);
  const [chatOrder, setChatOrder] = useState<OrderRecord | null>(null);
  const [chatHistory, setChatHistory] = useState<Record<string, ChatMessage[]>>({
    '2083d7ff-1cf9-4755-905b-1585533c6857': [
      {
        id: 'msg-1',
        sender: 'customer',
        text: 'Hello! Can you please check when my order will be delivered to Ashkona, Dhaka?',
        timestamp: '10:14 AM',
      },
      {
        id: 'msg-2',
        sender: 'moderator',
        text: 'Greetings! Your order has been processed at the Ajrah Noor Atelier and is dispatched for delivery.',
        timestamp: '10:20 AM',
      },
    ],
  });
  const [newMessageText, setNewMessageText] = useState('');

  useEffect(() => {
    dispatch(fetchOrders());
  }, [dispatch]);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  // Filter Orders
  const filteredOrders = useMemo(() => {
    return orders.filter((order) => {
      // 1. Filter by Tab
      if (activeFilter === 'Pending' && order.status !== 'Pending') return false;
      if (activeFilter === 'Processing' && order.status !== 'Processing' && order.status !== 'Paid' && order.status !== 'Reserved') return false;
      if (activeFilter === 'Delivered' && order.status !== 'Delivered' && order.status !== 'Shipped') return false;
      if (activeFilter === 'Returned' && order.status !== 'Returned' && order.status !== 'Cancelled') return false;

      // 2. Filter by Search Query
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesName = order.customer_name.toLowerCase().includes(q);
        const matchesProduct = order.product_name.toLowerCase().includes(q);
        const matchesId = order.order_number.toLowerCase().includes(q) || order.id.toLowerCase().includes(q);
        const matchesPhone = order.shipping_address?.phone?.toLowerCase().includes(q);
        if (!matchesName && !matchesProduct && !matchesId && !matchesPhone) return false;
      }
      return true;
    });
  }, [orders, activeFilter, searchQuery]);

  // Handle Status Update
  const handleUpdateStatus = (order: OrderRecord, newStatus: OrderState) => {
    let backendState: OrderState = newStatus;
    if (newStatus === 'Processing') backendState = 'Paid';
    if (newStatus === 'Delivered') backendState = 'Shipped';
    if (newStatus === 'Returned') backendState = 'Cancelled';

    dispatch(updateOrderStatusThunk({ id: order.id, status: backendState, version: order.version }));
    dispatch(updateOrderStatus({ id: order.id, status: newStatus }));
    showToast(`Order ${order.order_number} status updated to ${newStatus}`);
  };

  // Handle Send Chat Message
  const handleSendMessage = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!newMessageText.trim() || !chatOrder) return;

    const orderId = chatOrder.id;
    const newMsg: ChatMessage = {
      id: `msg-${Date.now()}`,
      sender: 'moderator',
      text: newMessageText.trim(),
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    };

    setChatHistory((prev) => ({
      ...prev,
      [orderId]: [...(prev[orderId] || []), newMsg],
    }));

    setNewMessageText('');
    showToast(`Message sent to ${chatOrder.customer_name}`);
  };

  const renderStatusBadge = (status?: string | OrderState) => {
    const s = String(status || 'Pending');
    switch (s) {
      case 'Pending':
        return (
          <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-purple-100 text-purple-700 border border-purple-200">
            <Clock className="w-3 h-3 mr-1" />
            Pending
          </span>
        );
      case 'Processing':
      case 'Paid':
      case 'Reserved':
        return (
          <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-amber-100 text-amber-700 border border-amber-200">
            <PackageCheck className="w-3 h-3 mr-1" />
            Processing
          </span>
        );
      case 'Delivered':
      case 'Shipped':
        return (
          <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-100 text-emerald-700 border border-emerald-200">
            <CheckCircle2 className="w-3 h-3 mr-1" />
            Delivered
          </span>
        );
      case 'Returned':
      case 'Cancelled':
        return (
          <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-rose-100 text-rose-700 border border-rose-200">
            <RotateCcw className="w-3 h-3 mr-1" />
            Returned
          </span>
        );
      default:
        return (
          <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-gray-100 text-gray-700">
            {s}
          </span>
        );
    }
  };

  return (
    <div className="min-h-screen bg-[#f8fafc] text-gray-800 font-sans flex flex-col">
      {/* Toast Notification */}
      {toastMessage && (
        <div className="fixed top-5 right-5 z-50 bg-gray-900 text-white text-xs font-semibold px-4 py-3 rounded-xl shadow-2xl flex items-center space-x-2 animate-in fade-in slide-in-from-top-3">
          <Sparkles className="w-4 h-4 text-[#f59e0b]" />
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Moderator Header */}
      <header className="bg-white border-b border-gray-200 px-6 py-4 sticky top-0 z-30 shadow-2xs">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center space-x-3.5">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-purple-700 to-indigo-900 flex items-center justify-center text-amber-400 shadow-md">
              <ShieldCheck className="w-6 h-6" />
            </div>
            <div>
              <div className="flex items-center space-x-2">
                <h1 className="text-xl font-extrabold text-gray-900 font-serif tracking-tight">
                  Ajrah Noor Moderator Console
                </h1>
                <span className="px-2 py-0.5 text-[10px] uppercase font-bold tracking-widest bg-purple-100 text-purple-800 rounded-md border border-purple-200">
                  Moderator
                </span>
              </div>
              <p className="text-xs text-gray-500 font-medium">
                Check live customer orders, update delivery statuses, and message customers directly.
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-3">
            <button
              onClick={() => dispatch(fetchOrders())}
              className="p-2 text-gray-600 hover:text-gray-900 bg-gray-100 hover:bg-gray-200 rounded-xl transition-all flex items-center space-x-1.5 text-xs font-semibold cursor-pointer"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
              <span>Refresh Orders</span>
            </button>
            <button
              onClick={() => navigate('/dashboard')}
              className="px-3.5 py-2 text-xs font-bold text-gray-700 bg-white border border-gray-300 rounded-xl hover:bg-gray-50 transition-all flex items-center space-x-1.5 cursor-pointer"
            >
              <ArrowLeft className="w-3.5 h-3.5" />
              <span>Admin Dashboard</span>
            </button>
          </div>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 max-w-7xl w-full mx-auto p-4 md:p-6 space-y-6">
        {/* Controls Bar */}
        <div className="bg-white p-4 rounded-2xl border border-gray-200 shadow-2xs flex flex-col md:flex-row md:items-center justify-between gap-4">
          {/* Search Input */}
          <div className="relative flex-1 max-w-md">
            <Search className="w-4 h-4 text-gray-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search by Order ID, Customer Name, Product, or Phone..."
              className="w-full pl-10 pr-4 py-2 bg-gray-50 border border-gray-200 rounded-xl text-xs focus:bg-white focus:outline-none focus:ring-2 focus:ring-purple-500/20 focus:border-purple-500 transition-all"
            />
          </div>

          {/* Status Filter Tabs */}
          <div className="flex items-center space-x-1 overflow-x-auto bg-gray-100 p-1 rounded-xl text-xs font-semibold">
            {(['All', 'Pending', 'Processing', 'Delivered', 'Returned'] as const).map((filter) => (
              <button
                key={filter}
                onClick={() => setActiveFilter(filter)}
                className={`px-3 py-1.5 rounded-lg transition-all cursor-pointer whitespace-nowrap ${
                  activeFilter === filter
                    ? 'bg-white text-gray-900 shadow-xs font-bold'
                    : 'text-gray-600 hover:text-gray-900 hover:bg-gray-200/60'
                }`}
              >
                {filter === 'All' ? 'All Orders' : filter}
              </button>
            ))}
          </div>
        </div>

        {/* Orders List Table */}
        <div className="bg-white rounded-2xl border border-gray-200 shadow-2xs overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="bg-gray-50/80 border-b border-gray-200 text-gray-500 font-bold uppercase tracking-wider text-[10px]">
                  <th className="py-3.5 px-4">Order ID</th>
                  <th className="py-3.5 px-4">Customer Details</th>
                  <th className="py-3.5 px-4">Product / Item</th>
                  <th className="py-3.5 px-4">Payment</th>
                  <th className="py-3.5 px-4">Amount</th>
                  <th className="py-3.5 px-4">Status</th>
                  <th className="py-3.5 px-4 text-center">Change Order Status</th>
                  <th className="py-3.5 px-4 text-right">Message & Actions</th>
                </tr>
              </thead>

              <tbody className="divide-y divide-gray-100">
                {loading && orders.length === 0 ? (
                  <tr>
                    <td colSpan={8} className="py-12 text-center text-gray-400 font-medium">
                      <RefreshCw className="w-6 h-6 animate-spin mx-auto mb-2 text-purple-600" />
                      Loading live orders from PostgreSQL...
                    </td>
                  </tr>
                ) : filteredOrders.length === 0 ? (
                  <tr>
                    <td colSpan={8} className="py-12 text-center text-gray-400 font-medium">
                      No orders found matching your search or status filter.
                    </td>
                  </tr>
                ) : (
                  filteredOrders.map((order) => (
                    <tr key={order.id} className="hover:bg-purple-50/30 transition-colors">
                      {/* Order ID */}
                      <td className="py-3.5 px-4 font-mono font-bold text-gray-900">
                        {order.order_number}
                        <div className="text-[10px] text-gray-400 font-sans font-normal">
                          {new Date(order.created_at).toLocaleDateString()}
                        </div>
                      </td>

                      {/* Customer Details */}
                      <td className="py-3.5 px-4">
                        <div className="font-bold text-gray-900 flex items-center space-x-1.5">
                          <User className="w-3.5 h-3.5 text-gray-400 shrink-0" />
                          <span>{order.customer_name}</span>
                        </div>
                        {order.shipping_address?.phone && (
                          <div className="text-[11px] text-gray-500 font-mono flex items-center space-x-1 mt-0.5">
                            <Phone className="w-3 h-3 text-purple-500 shrink-0" />
                            <span>{order.shipping_address.phone}</span>
                          </div>
                        )}
                        {order.shipping_address?.street_line1 && (
                          <div className="text-[10px] text-gray-400 truncate max-w-[180px] flex items-center space-x-1 mt-0.5">
                            <MapPin className="w-3 h-3 text-amber-500 shrink-0" />
                            <span className="truncate">{order.shipping_address.street_line1}</span>
                          </div>
                        )}
                      </td>

                      {/* Product */}
                      <td className="py-3.5 px-4">
                        <div className="font-semibold text-gray-800 flex items-center space-x-1.5">
                          <ShoppingBag className="w-3.5 h-3.5 text-gray-400 shrink-0" />
                          <span>{order.product_name}</span>
                        </div>
                        <div className="text-[10px] text-gray-400 font-mono">SKU: {order.sku}</div>
                      </td>

                      {/* Payment */}
                      <td className="py-3.5 px-4 text-gray-600 font-medium">{order.payment_method}</td>

                      {/* Amount */}
                      <td className="py-3.5 px-4 font-bold text-gray-900">
                        {order.currency}{order.amount.toFixed(2)}
                      </td>

                      {/* Status */}
                      <td className="py-3.5 px-4">{renderStatusBadge(order.status)}</td>

                      {/* Quick Change Order Status Buttons */}
                      <td className="py-3.5 px-4">
                        <div className="flex items-center justify-center space-x-1">
                          <button
                            onClick={() => handleUpdateStatus(order, 'Pending')}
                            title="Mark Pending"
                            className={`px-2 py-1 rounded-lg text-[10px] font-bold transition-all cursor-pointer ${
                              order.status === 'Pending'
                                ? 'bg-purple-600 text-white shadow-xs'
                                : 'bg-purple-50 text-purple-700 hover:bg-purple-100'
                            }`}
                          >
                            Pending
                          </button>
                          <button
                            onClick={() => handleUpdateStatus(order, 'Processing')}
                            title="Mark Processing"
                            className={`px-2 py-1 rounded-lg text-[10px] font-bold transition-all cursor-pointer ${
                              order.status === 'Processing' || order.status === 'Paid'
                                ? 'bg-amber-600 text-white shadow-xs'
                                : 'bg-amber-50 text-amber-700 hover:bg-amber-100'
                            }`}
                          >
                            Processing
                          </button>
                          <button
                            onClick={() => handleUpdateStatus(order, 'Delivered')}
                            title="Mark Delivered"
                            className={`px-2 py-1 rounded-lg text-[10px] font-bold transition-all cursor-pointer ${
                              order.status === 'Delivered' || order.status === 'Shipped'
                                ? 'bg-emerald-600 text-white shadow-xs'
                                : 'bg-emerald-50 text-emerald-100 text-emerald-700 hover:bg-emerald-100'
                            }`}
                          >
                            Delivered
                          </button>
                          <button
                            onClick={() => handleUpdateStatus(order, 'Returned')}
                            title="Mark Returned"
                            className={`px-2 py-1 rounded-lg text-[10px] font-bold transition-all cursor-pointer ${
                              order.status === 'Returned' || order.status === 'Cancelled'
                                ? 'bg-rose-600 text-white shadow-xs'
                                : 'bg-rose-50 text-rose-700 hover:bg-rose-100'
                            }`}
                          >
                            Returned
                          </button>
                        </div>
                      </td>

                      {/* Actions & Message Button */}
                      <td className="py-3.5 px-4 text-right">
                        <div className="flex items-center justify-end space-x-2">
                          <button
                            onClick={() => setChatOrder(order)}
                            className="px-2.5 py-1.5 bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-700 hover:to-indigo-700 text-white rounded-lg text-xs font-bold flex items-center space-x-1 shadow-2xs transition-all cursor-pointer"
                          >
                            <MessageCircle className="w-3.5 h-3.5" />
                            <span>Message</span>
                          </button>

                          <button
                            onClick={() => setInspectOrder(order)}
                            className="p-1.5 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-lg transition-colors cursor-pointer"
                            title="View Full Order & Customer Details"
                          >
                            <Eye className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      </main>

      {/* Customer Messaging Chat Modal */}
      {chatOrder && (
        <div className="fixed inset-0 z-50 bg-gray-900/60 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in">
          <div className="bg-white w-full max-w-lg rounded-2xl shadow-2xl border border-gray-200 overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95">
            {/* Modal Header */}
            <div className="bg-gradient-to-r from-purple-900 to-indigo-950 px-5 py-4 text-white flex items-center justify-between">
              <div className="flex items-center space-x-3">
                <div className="w-9 h-9 rounded-full bg-amber-400 text-gray-900 font-extrabold flex items-center justify-center text-sm shadow-sm">
                  {chatOrder.customer_name.slice(0, 2).toUpperCase()}
                </div>
                <div>
                  <h3 className="font-bold text-sm font-serif">{chatOrder.customer_name}</h3>
                  <p className="text-[11px] text-purple-200 font-mono">
                    Order {chatOrder.order_number} • {chatOrder.shipping_address?.phone || 'No phone'}
                  </p>
                </div>
              </div>
              <button
                onClick={() => setChatOrder(null)}
                className="p-1.5 text-purple-200 hover:text-white hover:bg-white/10 rounded-lg transition-colors cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Chat Body */}
            <div className="flex-1 p-4 overflow-y-auto space-y-3 bg-gray-50 min-h-[250px]">
              <div className="text-center text-[10px] text-gray-400 font-mono my-2 uppercase tracking-wider">
                Direct Communication for Order {chatOrder.order_number}
              </div>

              {/* Message Thread */}
              {(chatHistory[chatOrder.id] || [
                {
                  id: 'default-1',
                  sender: 'customer',
                  text: `Hi! I placed order ${chatOrder.order_number} for ${chatOrder.product_name}. Please update me on delivery status.`,
                  timestamp: 'Just now',
                },
              ]).map((msg) => (
                <div
                  key={msg.id}
                  className={`flex flex-col ${
                    msg.sender === 'moderator' ? 'items-end' : 'items-start'
                  }`}
                >
                  <div
                    className={`max-w-[80%] px-3.5 py-2.5 rounded-2xl text-xs leading-relaxed ${
                      msg.sender === 'moderator'
                        ? 'bg-purple-600 text-white rounded-br-none shadow-xs font-sans'
                        : 'bg-white text-gray-800 border border-gray-200 rounded-bl-none shadow-2xs font-sans'
                    }`}
                  >
                    {msg.text}
                  </div>
                  <span className="text-[10px] text-gray-400 mt-1 font-mono px-1">
                    {msg.sender === 'moderator' ? 'Moderator' : chatOrder.customer_name} • {msg.timestamp}
                  </span>
                </div>
              ))}
            </div>

            {/* Quick Action Responses */}
            <div className="p-2 bg-white border-t border-gray-100 flex items-center space-x-1.5 overflow-x-auto text-[11px]">
              <button
                onClick={() => setNewMessageText('Your order has been confirmed by our Atelier and is being prepared!')}
                className="px-2.5 py-1 bg-purple-50 text-purple-700 hover:bg-purple-100 rounded-full font-medium whitespace-nowrap cursor-pointer transition-colors"
              >
                ⚡ Order Confirmed
              </button>
              <button
                onClick={() => setNewMessageText('Your package has been dispatched via courier with tracking number!')}
                className="px-2.5 py-1 bg-amber-50 text-amber-700 hover:bg-amber-100 rounded-full font-medium whitespace-nowrap cursor-pointer transition-colors"
              >
                🚚 Dispatched Today
              </button>
              <button
                onClick={() => setNewMessageText('Thank you! Please feel free to reach out if you have any questions.')}
                className="px-2.5 py-1 bg-emerald-50 text-emerald-700 hover:bg-emerald-100 rounded-full font-medium whitespace-nowrap cursor-pointer transition-colors"
              >
                ✨ Thank You
              </button>
            </div>

            {/* Chat Input */}
            <form onSubmit={handleSendMessage} className="p-3 bg-white border-t border-gray-200 flex items-center space-x-2">
              <input
                type="text"
                value={newMessageText}
                onChange={(e) => setNewMessageText(e.target.value)}
                placeholder="Type your message to the customer..."
                className="flex-1 px-3.5 py-2 bg-gray-50 border border-gray-200 rounded-xl text-xs focus:bg-white focus:outline-none focus:ring-2 focus:ring-purple-500/20 focus:border-purple-500"
              />
              <button
                type="submit"
                disabled={!newMessageText.trim()}
                className="px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:opacity-50 text-white text-xs font-bold rounded-xl flex items-center space-x-1 shadow-md transition-all cursor-pointer"
              >
                <Send className="w-3.5 h-3.5" />
                <span>Send</span>
              </button>
            </form>
          </div>
        </div>
      )}

      {/* Inspect Order Details Modal */}
      {inspectOrder && (
        <div className="fixed inset-0 z-50 bg-gray-900/60 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in">
          <div className="bg-white w-full max-w-lg rounded-2xl shadow-2xl border border-gray-200 p-6 space-y-5 animate-in zoom-in-95">
            <div className="flex items-center justify-between border-b border-gray-100 pb-3">
              <div className="flex items-center space-x-2">
                <ShoppingBag className="w-5 h-5 text-purple-600" />
                <h3 className="font-bold text-base font-serif text-gray-900">
                  Order Details ({inspectOrder.order_number})
                </h3>
              </div>
              <button
                onClick={() => setInspectOrder(null)}
                className="p-1 text-gray-400 hover:text-gray-700 rounded-lg transition-colors cursor-pointer"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-4 text-xs">
              <div className="grid grid-cols-2 gap-3 bg-gray-50 p-3.5 rounded-xl border border-gray-200/80">
                <div>
                  <span className="text-gray-400 font-bold block uppercase text-[10px]">Customer</span>
                  <span className="font-semibold text-gray-900 text-sm">{inspectOrder.customer_name}</span>
                </div>
                <div>
                  <span className="text-gray-400 font-bold block uppercase text-[10px]">Current Status</span>
                  {renderStatusBadge(inspectOrder.status)}
                </div>
                <div>
                  <span className="text-gray-400 font-bold block uppercase text-[10px]">Total Amount</span>
                  <span className="font-extrabold text-gray-900 text-sm">
                    {inspectOrder.currency}{inspectOrder.amount.toFixed(2)}
                  </span>
                </div>
                <div>
                  <span className="text-gray-400 font-bold block uppercase text-[10px]">Payment Method</span>
                  <span className="font-medium text-gray-700">{inspectOrder.payment_method}</span>
                </div>
              </div>

              {/* Shipping Information */}
              <div className="space-y-2">
                <span className="font-bold text-gray-900 block text-xs uppercase tracking-wider">
                  Shipping & Contact Address
                </span>
                <div className="p-3.5 bg-purple-50/50 rounded-xl border border-purple-100 space-y-1.5">
                  <div className="flex items-center space-x-2 font-medium text-gray-800">
                    <User className="w-3.5 h-3.5 text-purple-600" />
                    <span>{inspectOrder.shipping_address?.recipient_name || inspectOrder.customer_name}</span>
                  </div>
                  {inspectOrder.shipping_address?.phone && (
                    <div className="flex items-center space-x-2 font-mono text-purple-900">
                      <Phone className="w-3.5 h-3.5 text-purple-600" />
                      <span>{inspectOrder.shipping_address.phone}</span>
                    </div>
                  )}
                  {inspectOrder.shipping_address?.street_line1 && (
                    <div className="flex items-start space-x-2 text-gray-600">
                      <MapPin className="w-3.5 h-3.5 text-amber-500 mt-0.5 shrink-0" />
                      <span>
                        {inspectOrder.shipping_address.street_line1},{' '}
                        {inspectOrder.shipping_address.city} {inspectOrder.shipping_address.postal_code}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>

            <div className="pt-3 border-t border-gray-100 flex items-center justify-end space-x-2">
              <button
                onClick={() => {
                  setChatOrder(inspectOrder);
                  setInspectOrder(null);
                }}
                className="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-xl text-xs font-bold flex items-center space-x-1.5 shadow-md cursor-pointer"
              >
                <MessageCircle className="w-4 h-4" />
                <span>Message Customer</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ModeratorPanel;
