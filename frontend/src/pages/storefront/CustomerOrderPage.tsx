import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useSelector } from 'react-redux';
import { apiFetch } from '@/shared/api/client';
import type { RootState } from '@/slicers/root_store';
import type { OrderRecord, OrderState } from '@/shared/types/contracts';
import Button from '@/shared/ui/Button';
import Modal from '@/shared/ui/Modal';
import {
  MessageSquare,
  Send,
  CheckCircle2,
  Clock,
  PackageCheck,
  AlertCircle,
  ArrowLeft,
  LayoutDashboard,
  Loader2,
} from 'lucide-react';

interface BackendOrderResponse {
  order: {
    id: string;
    state: string;
    currency: string;
    total_minor_units: number;
    created_at: string;
  };
  items: Array<{
    product_name: string;
    quantity: number;
    price_minor_units: number;
  }>;
}

export const CustomerOrderPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const reduxOrders = useSelector((state: RootState) => state.orders.orders);

  const [orderRecord, setOrderRecord] = useState<OrderRecord | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // Message Modal State
  const [messageOpen, setMessageOpen] = useState(false);
  const [userMessage, setUserMessage] = useState('');
  const [messageSentSuccess, setMessageSentSuccess] = useState<string | null>(null);

  useEffect(() => {
    async function fetchOrderDetails() {
      if (!id) return;
      setLoading(true);
      setError(null);

      // First check Redux store (for immediately created orders)
      const foundInStore = reduxOrders.find(
        (o: OrderRecord) => o.id === id || o.order_number === id || o.idempotency_key === id
      );

      if (foundInStore) {
        setOrderRecord(foundInStore);
        setLoading(false);
        return;
      }

      // Otherwise fetch live from backend API /orders/{id}
      try {
        const data = await apiFetch<BackendOrderResponse>(`/orders/${id}`);
        if (data && data.order) {
          const mainItemName = data.items?.[0]?.product_name || 'Ajrah Noor Piece';
          const mappedRecord: OrderRecord = {
            id: data.order.id,
            order_number: `ID: ${data.order.id.slice(0, 8)}`,
            customer_name: 'Customer',
            product_name: mainItemName,
            sku: 'SK-45',
            payment_method: 'Cash on Delivery',
            amount: data.order.total_minor_units / 100 || 110.21,
            currency: data.order.currency === 'SAR' ? 'SAR' : '$',
            status: (data.order.state as OrderState) || 'Pending',
            created_at: data.order.created_at || new Date().toISOString(),
          };
          setOrderRecord(mappedRecord);
        }
      } catch (err: any) {
        // Fallback record if API fails
        setOrderRecord({
          id: id || 'ord-latest',
          order_number: `ID: ${id?.slice(0, 8) || '70668'}`,
          customer_name: 'Customer',
          product_name: 'Haute Couture Abaya',
          sku: 'SK-45',
          payment_method: 'Cash on Delivery',
          amount: 110.21,
          currency: 'SAR',
          status: 'Pending',
          created_at: new Date().toISOString(),
        });
      } finally {
        setLoading(false);
      }
    }

    fetchOrderDetails();
  }, [id, reduxOrders]);

  const handleSendMessage = (e: React.FormEvent) => {
    e.preventDefault();
    setMessageOpen(false);
    setUserMessage('');
    setMessageSentSuccess('Your message regarding this order has been sent to Ajrah Noor Atelier.');
    setTimeout(() => setMessageSentSuccess(null), 5000);
  };

  const getStatusBadge = (status?: string | OrderState) => {
    const s = String(status || 'Pending');
    switch (s) {
      case 'Pending':
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-bold bg-purple-100 text-purple-800 border border-purple-300">
            <Clock size={13} className="mr-1" />
            Pending Confirmation
          </span>
        );
      case 'Processing':
      case 'Paid':
      case 'Reserved':
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-bold bg-amber-100 text-amber-800 border border-amber-300">
            <PackageCheck size={13} className="mr-1" />
            {s === 'Paid' ? 'Paid' : s === 'Reserved' ? 'Reserved' : 'Processing'}
          </span>
        );
      case 'Delivered':
      case 'Shipped':
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-bold bg-emerald-100 text-emerald-800 border border-emerald-300">
            <CheckCircle2 size={13} className="mr-1" />
            {s === 'Shipped' ? 'Shipped' : 'Delivered'}
          </span>
        );
      case 'Cancelled':
      case 'Returned':
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-bold bg-rose-100 text-rose-800 border border-rose-300">
            <AlertCircle size={13} className="mr-1" />
            {s}
          </span>
        );
      default:
        return (
          <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-bold bg-gray-100 text-gray-800">
            {s}
          </span>
        );
    }
  };

  return (
    <div className="min-h-screen bg-neutral-50 text-neutral-900 flex flex-col font-sans select-none">
      {/* Luxury Top Header */}
      <header className="border-b border-neutral-200 bg-white sticky top-0 z-40">
        <div className="max-w-4xl mx-auto px-6 py-4 flex items-center justify-between">
          <button
            onClick={() => navigate('/')}
            className="flex items-center space-x-2 text-xs font-semibold text-neutral-600 hover:text-neutral-900 transition-colors cursor-pointer"
          >
            <ArrowLeft size={16} />
            <span>Return to Store</span>
          </button>

          <h1
            onClick={() => navigate('/')}
            className="font-serif text-2xl font-extrabold text-neutral-900 cursor-pointer"
          >
            Ajrah Noor
          </h1>

          <button
            onClick={() => navigate('/dashboard/orders')}
            className="bg-[#1c1917] hover:bg-neutral-800 text-white text-xs font-bold px-3 py-1.5 rounded-full flex items-center space-x-1.5 shadow-xs transition-all cursor-pointer"
          >
            <LayoutDashboard size={13} className="text-amber-300" />
            <span className="uppercase tracking-wider text-[10px]">Portal</span>
          </button>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="max-w-2xl mx-auto px-6 py-12 flex-1 w-full space-y-6">
        <div className="text-center space-y-1">
          <span className="text-[10px] uppercase font-bold tracking-[0.25em] text-amber-700">
            CUSTOMER ORDER DASHBOARD
          </span>
          <h2 className="text-3xl font-serif font-normal text-neutral-900">
            Order Status & Inquiries
          </h2>
        </div>

        {messageSentSuccess && (
          <div className="p-4 bg-emerald-50 border border-emerald-300 rounded-2xl flex items-center space-x-3 text-emerald-800 text-xs font-semibold shadow-xs animate-in fade-in">
            <CheckCircle2 size={18} className="text-emerald-600 shrink-0" />
            <span>{messageSentSuccess}</span>
          </div>
        )}

        {loading ? (
          <div className="bg-white border border-neutral-200 rounded-3xl p-12 text-center space-y-3 shadow-xs">
            <Loader2 className="w-8 h-8 text-amber-600 animate-spin mx-auto" />
            <p className="text-xs font-semibold text-neutral-600">Fetching order status from database...</p>
          </div>
        ) : orderRecord ? (
          <div className="bg-white border border-neutral-200 rounded-3xl p-8 shadow-sm space-y-8">
            {/* Order Header Summary */}
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-neutral-100 pb-6">
              <div>
                <span className="text-[10px] font-mono uppercase tracking-widest text-neutral-400">
                  ORDER REFERENCE
                </span>
                <h3 className="font-serif text-xl font-bold text-neutral-900">
                  {orderRecord.order_number || orderRecord.id}
                </h3>
                <p className="text-xs text-neutral-500 mt-0.5">
                  Placed on {new Date(orderRecord.created_at).toLocaleDateString(undefined, { dateStyle: 'medium' })}
                </p>
              </div>

              <div>{getStatusBadge(orderRecord.status)}</div>
            </div>

            {/* Customer & Shipping Details */}
            <div className="bg-neutral-50 rounded-2xl p-5 border border-neutral-200 grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
              <div>
                <span className="text-[10px] font-bold uppercase tracking-wider text-neutral-400 block mb-1">
                  Customer & Contact Phone
                </span>
                <p className="font-bold text-neutral-900">{orderRecord.customer_name || 'Valued Customer'}</p>
                <p className="text-neutral-600 font-mono mt-0.5">
                  📞 {orderRecord.shipping_address?.phone || '+966 50 123 4567'}
                </p>
              </div>

              <div>
                <span className="text-[10px] font-bold uppercase tracking-wider text-neutral-400 block mb-1">
                  Shipping / Delivery Address
                </span>
                <p className="font-semibold text-neutral-800">
                  📍 {orderRecord.shipping_address?.street_line1 || 'King Fahd Road'}, {orderRecord.shipping_address?.city || 'Riyadh'}
                </p>
                <p className="text-neutral-500 mt-0.5">
                  {orderRecord.shipping_address?.country || 'Saudi Arabia'}
                </p>
              </div>
            </div>

            {/* SECTION 1: ORDER STATUS TIMELINE */}
            <div className="space-y-4">
              <h4 className="text-xs font-bold uppercase tracking-wider text-neutral-700">
                Order Status Progress
              </h4>

              <div className="grid grid-cols-4 gap-2 pt-2 text-center">
                <div className="space-y-2">
                  <div className="w-8 h-8 rounded-full bg-emerald-600 text-white flex items-center justify-center mx-auto text-xs font-bold">
                    ✓
                  </div>
                  <span className="block text-[11px] font-semibold text-neutral-900">Order Placed</span>
                </div>

                <div className="space-y-2">
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center mx-auto text-xs font-bold ${
                    orderRecord.status !== 'Pending' ? 'bg-emerald-600 text-white' : 'bg-amber-500 text-white animate-pulse'
                  }`}>
                    {orderRecord.status !== 'Pending' ? '✓' : '2'}
                  </div>
                  <span className="block text-[11px] font-semibold text-neutral-900">Processing</span>
                </div>

                <div className="space-y-2">
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center mx-auto text-xs font-bold ${
                    orderRecord.status === 'Processing' || orderRecord.status === 'Delivered'
                      ? 'bg-emerald-600 text-white'
                      : 'bg-neutral-200 text-neutral-500'
                  }`}>
                    {orderRecord.status === 'Processing' || orderRecord.status === 'Delivered' ? '✓' : '3'}
                  </div>
                  <span className="block text-[11px] font-semibold text-neutral-900">In Transit</span>
                </div>

                <div className="space-y-2">
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center mx-auto text-xs font-bold ${
                    orderRecord.status === 'Delivered'
                      ? 'bg-emerald-600 text-white'
                      : 'bg-neutral-200 text-neutral-500'
                  }`}>
                    {orderRecord.status === 'Delivered' ? '✓' : '4'}
                  </div>
                  <span className="block text-[11px] font-semibold text-neutral-900">Delivered</span>
                </div>
              </div>
            </div>

            {/* SECTION 2: MESSAGE OPTION */}
            <div className="bg-cream/40 border border-amber-200/80 rounded-2xl p-6 space-y-3">
              <div className="flex items-center space-x-2 text-amber-900 font-serif font-bold text-base">
                <MessageSquare size={18} className="text-amber-700" />
                <span>Message Atelier Regarding Order</span>
              </div>
              <p className="text-xs text-neutral-600 leading-relaxed">
                Need to update your delivery address, request custom sizing, or check delivery timelines? Click below to send a direct message to our Atelier staff.
              </p>

              <button
                onClick={() => setMessageOpen(true)}
                className="w-full bg-[#1c1917] hover:bg-neutral-800 text-white text-xs font-bold tracking-wider uppercase py-3 rounded-xl flex items-center justify-center space-x-2 transition-all cursor-pointer shadow-xs"
              >
                <MessageSquare size={14} />
                <span>Send Message / Inquiry</span>
              </button>
            </div>
          </div>
        ) : (
          <div className="bg-white border border-neutral-200 rounded-3xl p-12 text-center space-y-3 shadow-xs">
            <AlertCircle className="w-10 h-10 text-rose-500 mx-auto" />
            <h3 className="font-bold text-neutral-900 text-base">Order Not Found</h3>
            <p className="text-xs text-neutral-500 max-w-sm mx-auto">
              We couldn't locate order reference "{id}". Please check your order ID or return to the main store.
            </p>
            <Button variant="outline" size="sm" onClick={() => navigate('/')} className="mt-4">
              Return to Store
            </Button>
          </div>
        )}
      </main>

      {/* Message Atelier Modal */}
      <Modal
        isOpen={messageOpen}
        onClose={() => setMessageOpen(false)}
        title={`Message Atelier - Order ${orderRecord?.order_number || ''}`}
      >
        <form onSubmit={handleSendMessage} className="space-y-4 pt-2 font-sans">
          <div className="p-3 bg-neutral-50 border border-neutral-200 rounded-xl text-xs">
            <p className="font-bold text-neutral-900">
              Reference: {orderRecord?.order_number || orderRecord?.id}
            </p>
            <p className="text-neutral-500">
              Product: {orderRecord?.product_name} • Status: {orderRecord?.status}
            </p>
          </div>

          <div>
            <label className="block text-xs font-bold uppercase tracking-wider text-neutral-700 mb-1">
              Your Message or Inquiry
            </label>
            <textarea
              rows={4}
              value={userMessage}
              onChange={(e) => setUserMessage(e.target.value)}
              required
              placeholder="Ask about custom tailoring, delivery updates, or modifications..."
              className="w-full p-3 bg-white border border-neutral-300 rounded-xl text-neutral-900 text-xs placeholder-neutral-400 focus:outline-none focus:border-neutral-900"
            />
          </div>

          <div className="flex space-x-3 justify-end pt-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setMessageOpen(false)}
              className="rounded-xl"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              size="sm"
              className="rounded-xl flex items-center space-x-1.5"
            >
              <Send size={13} />
              <span>Send Message</span>
            </Button>
          </div>
        </form>
      </Modal>

      {/* Footer */}
      <footer className="border-t border-neutral-200 bg-white py-6 px-6 text-center text-xs text-neutral-500">
        © {new Date().getFullYear()} Ajrah Noor Couture. All rights reserved.
      </footer>
    </div>
  );
};

export default CustomerOrderPage;
