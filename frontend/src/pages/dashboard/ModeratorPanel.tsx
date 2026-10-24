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
import { Alert, AlertDescription } from '@/shared/ui/alert';
import {
  ShieldCheck,
  Search,
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
  MessageCircle,
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

  const { orders, loading, error } = useSelector((state: RootState) => state.orders);

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
  const handleUpdateStatus = async (order: OrderRecord, newStatus: OrderState) => {
    let backendState: OrderState = newStatus;
    if (newStatus === 'Processing') backendState = 'Paid';
    if (newStatus === 'Delivered') backendState = 'Shipped';
    if (newStatus === 'Returned') backendState = 'Cancelled';

    try {
      await dispatch(updateOrderStatusThunk({
        id: order.id,
        status: backendState,
        version: order.version,
      })).unwrap();
      dispatch(updateOrderStatus({ id: order.id, status: newStatus }));
      showToast(`Order ${order.order_number} status updated to ${newStatus}`);
    } catch (requestError) {
      showToast(`Unable to update order: ${requestError instanceof Error ? requestError.message : 'Backend request failed.'}`);
    }
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
          <Badge variant="outline" className="bg-purple-50 text-purple-700 border-purple-200 text-[10px] font-semibold">
            <Clock className="w-3 h-3 mr-1" />
            Pending
          </Badge>
        );
      case 'Processing':
      case 'Paid':
      case 'Reserved':
        return (
          <Badge variant="outline" className="bg-amber-50 text-amber-700 border-amber-200 text-[10px] font-semibold">
            <PackageCheck className="w-3 h-3 mr-1" />
            Processing
          </Badge>
        );
      case 'Delivered':
      case 'Shipped':
        return (
          <Badge variant="outline" className="bg-emerald-50 text-emerald-700 border-emerald-200 text-[10px] font-semibold">
            <CheckCircle2 className="w-3 h-3 mr-1" />
            Delivered
          </Badge>
        );
      case 'Returned':
      case 'Cancelled':
        return (
          <Badge variant="destructive" className="text-[10px] font-semibold">
            <RotateCcw className="w-3 h-3 mr-1" />
            Returned
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
    <div className="min-h-screen bg-neutral-50 text-neutral-800 font-sans flex flex-col">
      {error && (
        <Alert variant="destructive" className="mx-auto mt-6 max-w-7xl">
          <AlertDescription>Unable to load orders: {error}</AlertDescription>
        </Alert>
      )}

      {/* Toast Notification */}
      {toastMessage && (
        <div className="fixed top-5 right-5 z-50 bg-neutral-900 text-white text-xs font-semibold px-4 py-3 rounded-xl shadow-2xl flex items-center space-x-2 animate-in fade-in slide-in-from-top-3">
          <Sparkles className="w-4 h-4 text-amber-400" />
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Moderator Header */}
      <header className="bg-white border-b border-neutral-200 px-6 py-4 sticky top-0 z-30 shadow-2xs">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center space-x-3.5">
            <div className="w-10 h-10 rounded-xl bg-neutral-900 flex items-center justify-center text-amber-400 shadow-md">
              <ShieldCheck className="w-6 h-6" />
            </div>
            <div>
              <div className="flex items-center space-x-2">
                <h1 className="text-xl font-extrabold text-neutral-900 font-serif tracking-tight">
                  Ajrah Noor Moderator Console
                </h1>
                <Badge variant="secondary" className="text-[10px] uppercase font-bold tracking-widest bg-purple-100 text-purple-800 border-purple-200">
                  Moderator
                </Badge>
              </div>
              <p className="text-xs text-neutral-500 font-medium">
                Check live customer orders, update delivery statuses, and message customers directly.
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-3">
            <Button
              variant="outline"
              size="sm"
              onClick={() => dispatch(fetchOrders())}
              className="text-xs font-semibold rounded-xl h-9"
            >
              <RefreshCw className={`w-3.5 h-3.5 mr-1.5 ${loading ? 'animate-spin' : ''}`} />
              <span>Refresh Orders</span>
            </Button>
            <Button
              variant="luxury"
              size="sm"
              onClick={() => navigate('/dashboard/orders')}
              className="text-xs font-bold rounded-xl h-9"
            >
              <ArrowLeft className="w-3.5 h-3.5 mr-1.5" />
              <span>Admin Dashboard</span>
            </Button>
          </div>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 max-w-7xl w-full mx-auto p-4 md:p-6 space-y-6">
        {/* Controls Bar */}
        <Card className="rounded-2xl border-neutral-200 shadow-2xs bg-white">
          <CardContent className="p-4 flex flex-col md:flex-row md:items-center justify-between gap-4">
            {/* Search Input */}
            <div className="relative flex-1 max-w-md">
              <Search className="w-4 h-4 text-neutral-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
              <Input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search by Order ID, Customer Name, Product, or Phone..."
                className="w-full pl-10 pr-4 py-2 bg-neutral-50 border-neutral-200 rounded-xl text-xs"
              />
            </div>

            {/* Status Filter Tabs */}
            <div className="flex items-center space-x-1 overflow-x-auto bg-neutral-100 p-1 rounded-xl text-xs font-semibold">
              {(['All', 'Pending', 'Processing', 'Delivered', 'Returned'] as const).map((filter) => (
                <Button
                  key={filter}
                  variant={activeFilter === filter ? 'primary' : 'ghost'}
                  size="sm"
                  onClick={() => setActiveFilter(filter)}
                  className="rounded-lg h-7 text-xs font-semibold whitespace-nowrap"
                >
                  {filter === 'All' ? 'All Orders' : filter}
                </Button>
              ))}
            </div>
          </CardContent>
        </Card>

        {/* Orders List Table */}
        <Card className="rounded-2xl border-neutral-200 shadow-2xs overflow-hidden bg-white">
          <Table>
            <TableHeader>
              <TableRow className="bg-neutral-50/80 border-b border-neutral-200 text-neutral-500 font-bold uppercase tracking-wider text-[10px]">
                <TableHead className="py-3.5 px-4">Order ID</TableHead>
                <TableHead className="py-3.5 px-4">Customer Details</TableHead>
                <TableHead className="py-3.5 px-4">Product / Item</TableHead>
                <TableHead className="py-3.5 px-4">Payment</TableHead>
                <TableHead className="py-3.5 px-4">Amount</TableHead>
                <TableHead className="py-3.5 px-4">Status</TableHead>
                <TableHead className="py-3.5 px-4 text-center">Change Order Status</TableHead>
                <TableHead className="py-3.5 px-4 text-right">Message & Actions</TableHead>
              </TableRow>
            </TableHeader>

            <TableBody className="divide-y divide-neutral-100 text-xs">
              {loading && orders.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={8} className="py-12 text-center text-neutral-400 font-medium">
                    <RefreshCw className="w-6 h-6 animate-spin mx-auto mb-2 text-purple-600" />
                    Loading live orders from PostgreSQL...
                  </TableCell>
                </TableRow>
              ) : filteredOrders.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={8} className="py-12 text-center text-neutral-400 font-medium">
                    No orders found matching your search or status filter.
                  </TableCell>
                </TableRow>
              ) : (
                filteredOrders.map((order) => (
                  <TableRow key={order.id} className="hover:bg-purple-50/20 transition-colors">
                    {/* Order ID */}
                    <TableCell className="py-3.5 px-4 font-mono font-bold text-neutral-900">
                      {order.order_number}
                      <div className="text-[10px] text-neutral-400 font-sans font-normal">
                        {new Date(order.created_at).toLocaleDateString()}
                      </div>
                    </TableCell>

                    {/* Customer Details */}
                    <TableCell className="py-3.5 px-4">
                      <div className="font-bold text-neutral-900 flex items-center space-x-1.5">
                        <User className="w-3.5 h-3.5 text-neutral-400 shrink-0" />
                        <span>{order.customer_name}</span>
                      </div>
                      {order.shipping_address?.phone && (
                        <div className="text-[11px] text-neutral-500 font-mono flex items-center space-x-1 mt-0.5">
                          <Phone className="w-3 h-3 text-purple-500 shrink-0" />
                          <span>{order.shipping_address.phone}</span>
                        </div>
                      )}
                      {order.shipping_address?.street_line1 && (
                        <div className="text-[10px] text-neutral-400 truncate max-w-[180px] flex items-center space-x-1 mt-0.5">
                          <MapPin className="w-3 h-3 text-amber-500 shrink-0" />
                          <span className="truncate">{order.shipping_address.street_line1}</span>
                        </div>
                      )}
                    </TableCell>

                    {/* Product */}
                    <TableCell className="py-3.5 px-4">
                      <div className="font-semibold text-neutral-800 flex items-center space-x-1.5">
                        <ShoppingBag className="w-3.5 h-3.5 text-neutral-400 shrink-0" />
                        <span>{order.product_name}</span>
                      </div>
                      <div className="text-[10px] text-neutral-400 font-mono">SKU: {order.sku}</div>
                    </TableCell>

                    {/* Payment */}
                    <TableCell className="py-3.5 px-4 text-neutral-600 font-medium">{order.payment_method}</TableCell>

                    {/* Amount */}
                    <TableCell className="py-3.5 px-4 font-bold text-neutral-900 font-mono">
                      {order.currency}{order.amount.toFixed(2)}
                    </TableCell>

                    {/* Status */}
                    <TableCell className="py-3.5 px-4">{renderStatusBadge(order.status)}</TableCell>

                    {/* Quick Change Order Status Buttons */}
                    <TableCell className="py-3.5 px-4">
                      <div className="flex items-center justify-center space-x-1">
                        <Button
                          variant={order.status === 'Pending' ? 'primary' : 'outline'}
                          size="sm"
                          onClick={() => handleUpdateStatus(order, 'Pending')}
                          title="Mark Pending"
                          className="h-7 px-2 text-[10px] rounded-lg"
                        >
                          Pending
                        </Button>
                        <Button
                          variant={order.status === 'Processing' || order.status === 'Paid' ? 'gold' : 'outline'}
                          size="sm"
                          onClick={() => handleUpdateStatus(order, 'Processing')}
                          title="Mark Processing"
                          className="h-7 px-2 text-[10px] rounded-lg"
                        >
                          Processing
                        </Button>
                        <Button
                          variant={order.status === 'Delivered' || order.status === 'Shipped' ? 'primary' : 'outline'}
                          size="sm"
                          onClick={() => handleUpdateStatus(order, 'Delivered')}
                          title="Mark Delivered"
                          className="h-7 px-2 text-[10px] rounded-lg"
                        >
                          Delivered
                        </Button>
                        <Button
                          variant={order.status === 'Returned' || order.status === 'Cancelled' ? 'destructive' : 'outline'}
                          size="sm"
                          onClick={() => handleUpdateStatus(order, 'Returned')}
                          title="Mark Returned"
                          className="h-7 px-2 text-[10px] rounded-lg"
                        >
                          Returned
                        </Button>
                      </div>
                    </TableCell>

                    {/* Actions & Message Button */}
                    <TableCell className="py-3.5 px-4 text-right">
                      <div className="flex items-center justify-end space-x-2">
                        <Button
                          variant="luxury"
                          size="sm"
                          onClick={() => setChatOrder(order)}
                          className="h-8 rounded-lg text-xs font-bold"
                        >
                          <MessageCircle className="w-3.5 h-3.5 mr-1" />
                          <span>Message</span>
                        </Button>

                        <Button
                          variant="outline"
                          size="icon"
                          onClick={() => setInspectOrder(order)}
                          title="View Full Order & Customer Details"
                          className="h-8 w-8 rounded-lg text-neutral-600"
                        >
                          <Eye className="w-3.5 h-3.5" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </Card>
      </main>

      {/* Customer Messaging Chat Modal */}
      {chatOrder && (
        <Dialog open={true} onOpenChange={(open) => !open && setChatOrder(null)}>
          <DialogContent className="max-w-lg p-0 overflow-hidden bg-white border-neutral-200 rounded-2xl">
            {/* Modal Header */}
            <div className="bg-[#1c1917] px-5 py-4 text-white flex items-center justify-between">
              <div className="flex items-center space-x-3">
                <div className="w-9 h-9 rounded-full bg-amber-400 text-neutral-900 font-extrabold flex items-center justify-center text-sm shadow-sm">
                  {chatOrder.customer_name.slice(0, 2).toUpperCase()}
                </div>
                <div>
                  <h3 className="font-bold text-sm font-serif">{chatOrder.customer_name}</h3>
                  <p className="text-[11px] text-neutral-300 font-mono">
                    Order {chatOrder.order_number} • {chatOrder.shipping_address?.phone || 'No phone'}
                  </p>
                </div>
              </div>
            </div>

            {/* Chat Body */}
            <div className="p-4 overflow-y-auto space-y-3 bg-neutral-50 min-h-[250px] max-h-[50vh]">
              <div className="text-center text-[10px] text-neutral-400 font-mono my-2 uppercase tracking-wider">
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
                        ? 'bg-neutral-900 text-white rounded-br-none shadow-xs font-sans'
                        : 'bg-white text-neutral-800 border border-neutral-200 rounded-bl-none shadow-2xs font-sans'
                    }`}
                  >
                    {msg.text}
                  </div>
                  <span className="text-[10px] text-neutral-400 mt-1 font-mono px-1">
                    {msg.sender === 'moderator' ? 'Moderator' : chatOrder.customer_name} • {msg.timestamp}
                  </span>
                </div>
              ))}
            </div>

            {/* Quick Action Responses */}
            <div className="p-2 bg-white border-t border-neutral-100 flex items-center space-x-1.5 overflow-x-auto text-[11px]">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setNewMessageText('Your order has been confirmed by our Atelier and is being prepared!')}
                className="h-7 rounded-full text-[11px]"
              >
                ⚡ Order Confirmed
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setNewMessageText('Your package has been dispatched via courier with tracking number!')}
                className="h-7 rounded-full text-[11px]"
              >
                🚚 Dispatched Today
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setNewMessageText('Thank you! Please feel free to reach out if you have any questions.')}
                className="h-7 rounded-full text-[11px]"
              >
                ✨ Thank You
              </Button>
            </div>

            {/* Chat Input */}
            <form onSubmit={handleSendMessage} className="p-3 bg-white border-t border-neutral-200 flex items-center space-x-2">
              <Input
                type="text"
                value={newMessageText}
                onChange={(e) => setNewMessageText(e.target.value)}
                placeholder="Type your message to the customer..."
                className="flex-1 text-xs h-9 bg-neutral-50"
              />
              <Button
                type="submit"
                variant="luxury"
                size="sm"
                disabled={!newMessageText.trim()}
                className="h-9 font-bold"
              >
                <Send className="w-3.5 h-3.5 mr-1" />
                <span>Send</span>
              </Button>
            </form>
          </DialogContent>
        </Dialog>
      )}

      {/* Inspect Order Details Modal */}
      {inspectOrder && (
        <Dialog open={true} onOpenChange={(open) => !open && setInspectOrder(null)}>
          <DialogContent className="max-w-lg bg-white border-neutral-200 rounded-2xl">
            <DialogHeader>
              <div className="flex items-center space-x-2">
                <ShoppingBag className="w-5 h-5 text-amber-700" />
                <DialogTitle className="font-bold text-base font-serif text-neutral-900">
                  Order Details ({inspectOrder.order_number})
                </DialogTitle>
              </div>
              <DialogDescription className="text-xs text-neutral-500">
                Complete order and shipping breakdown from PostgreSQL.
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 text-xs pt-2">
              <div className="grid grid-cols-2 gap-3 bg-neutral-50 p-3.5 rounded-xl border border-neutral-200/80">
                <div>
                  <span className="text-neutral-400 font-bold block uppercase text-[10px]">Customer</span>
                  <span className="font-semibold text-neutral-900 text-sm">{inspectOrder.customer_name}</span>
                </div>
                <div>
                  <span className="text-neutral-400 font-bold block uppercase text-[10px]">Current Status</span>
                  {renderStatusBadge(inspectOrder.status)}
                </div>
                <div>
                  <span className="text-neutral-400 font-bold block uppercase text-[10px]">Total Amount</span>
                  <span className="font-extrabold text-neutral-900 text-sm font-mono">
                    {inspectOrder.currency}{inspectOrder.amount.toFixed(2)}
                  </span>
                </div>
                <div>
                  <span className="text-neutral-400 font-bold block uppercase text-[10px]">Payment Method</span>
                  <span className="font-medium text-neutral-700">{inspectOrder.payment_method}</span>
                </div>
              </div>

              {/* Shipping Information */}
              <div className="space-y-2">
                <span className="font-bold text-neutral-900 block text-xs uppercase tracking-wider">
                  Shipping & Contact Address
                </span>
                <div className="p-3.5 bg-neutral-50 rounded-xl border border-neutral-200 space-y-1.5">
                  <div className="flex items-center space-x-2 font-medium text-neutral-800">
                    <User className="w-3.5 h-3.5 text-neutral-500" />
                    <span>{inspectOrder.shipping_address?.recipient_name || inspectOrder.customer_name}</span>
                  </div>
                  {inspectOrder.shipping_address?.phone && (
                    <div className="flex items-center space-x-2 font-mono text-neutral-800">
                      <Phone className="w-3.5 h-3.5 text-amber-600" />
                      <span>{inspectOrder.shipping_address.phone}</span>
                    </div>
                  )}
                  {inspectOrder.shipping_address?.street_line1 && (
                    <div className="flex items-start space-x-2 text-neutral-600">
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

            <DialogFooter className="pt-3 border-t border-neutral-100 flex items-center justify-end space-x-2">
              <Button
                variant="luxury"
                size="sm"
                onClick={() => {
                  setChatOrder(inspectOrder);
                  setInspectOrder(null);
                }}
                className="h-9 font-bold"
              >
                <MessageCircle className="w-4 h-4 mr-1.5" />
                <span>Message Customer</span>
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
};

export default ModeratorPanel;
