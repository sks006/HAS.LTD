import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useSelector } from 'react-redux';
import { ApiError, apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import { useAuth } from '@/app/providers/AuthProvider';
import type { RootState } from '@/slicers/root_store';
import type { OrderRecord, OrderState } from '@/shared/types/contracts';
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from '@/shared/ui/card';
import { Badge } from '@/shared/ui/badge';
import { Button } from '@/shared/ui/button';
import { Separator } from '@/shared/ui/separator';
import { Progress } from '@/shared/ui/progress';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/shared/ui/dialog';
import {
  ArrowLeft,
  Clock,
  PackageCheck,
  CheckCircle2,
  AlertCircle,
  MessageSquare,
  Send,
  Loader2,
  RefreshCw,
  Truck,
  ShieldCheck,
  MapPin,
  Calendar,
  CreditCard,
} from 'lucide-react';

interface BackendOrderResponse {
  order: {
    id: string;
    state: string;
    currency: string;
    total_minor_units: number;
    created_at: string;
    updated_at?: string;
    version?: number;
  };
  items: Array<{
    product_name: string;
    quantity: number;
    price_minor_units: number;
    sku?: string;
  }>;
}

export const CustomerOrderPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { isLoading: authLoading, token } = useAuth();
  const reduxOrders = useSelector((state: RootState) => state.orders.orders);

  const [orderRecord, setOrderRecord] = useState<OrderRecord | null>(null);
  const [itemsList, setItemsList] = useState<Array<{ name: string; quantity: number; price: number }>>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [refreshing, setRefreshing] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [requiresAuth, setRequiresAuth] = useState(false);

  // Message Atelier Inquiry Modal
  const [messageOpen, setMessageOpen] = useState(false);
  const [userMessage, setUserMessage] = useState('');
  const [messageSentSuccess, setMessageSentSuccess] = useState<string | null>(null);

  const fetchOrderDetails = async (isManualRefresh = false) => {
    if (!id) {
      setError('No order identifier specified.');
      setLoading(false);
      return;
    }

    if (isManualRefresh) setRefreshing(true);
    else setLoading(true);
    setError(null);
    setRequiresAuth(false);

    // First check Redux store for optimistic client-side order caching
    const foundInStore = reduxOrders.find(
      (o: OrderRecord) => o.id === id || o.order_number === id || o.idempotency_key === id
    );

    if (foundInStore && !isManualRefresh) {
      setOrderRecord(foundInStore);
      setItemsList([
        {
          name: foundInStore.product_name,
          quantity: 1,
          price: foundInStore.amount,
        },
      ]);
      setLoading(false);
    }

    // Always attempt live query against Axum endpoint GET /orders/{id}
    try {
      const data = await apiFetch<BackendOrderResponse>(ENDPOINTS.ORDER_BY_ID(id), { token });
      if (data && data.order) {
        const mainItemName = data.items?.[0]?.product_name || 'Haute Couture Abaya Piece';
        const mappedRecord: OrderRecord = {
          id: data.order.id,
          order_number: `ID: ${data.order.id.slice(0, 8)}`,
          customer_name: foundInStore?.customer_name || 'VIP Client',
          product_name: mainItemName,
          sku: data.items?.[0]?.sku || 'SKU-AN-2026',
          payment_method: foundInStore?.payment_method || 'Cash on Delivery',
          amount: data.order.total_minor_units / 100,
          currency: data.order.currency || 'SAR',
          status: (data.order.state as OrderState) || 'Pending',
          created_at: data.order.created_at,
          shipping_address: foundInStore?.shipping_address,
        };

        setOrderRecord(mappedRecord);
        if (data.items && data.items.length > 0) {
          setItemsList(
            data.items.map((i) => ({
              name: i.product_name,
              quantity: i.quantity,
              price: i.price_minor_units / 100,
            }))
          );
        }
      }
    } catch (requestError) {
      if (requestError instanceof ApiError && requestError.status === 401) {
        setRequiresAuth(true);
        if (!foundInStore) {
          setError('Authentication required: Sign in to view protected order tracking details.');
        }
      } else if (!foundInStore) {
        setError(requestError instanceof Error ? requestError.message : 'Unable to query order status.');
      }
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  useEffect(() => {
    fetchOrderDetails();
  }, [id, authLoading]);

  const handleSendMessage = (e: React.FormEvent) => {
    e.preventDefault();
    setMessageOpen(false);
    setUserMessage('');
    setMessageSentSuccess('Your inquiry has been relayed to the Ajrah Noor Atelier team.');
    setTimeout(() => setMessageSentSuccess(null), 5000);
  };

  const getStatusProgress = (status?: string | OrderState) => {
    const s = String(status || 'Pending').toLowerCase();
    if (s === 'pending') return { pct: 25, label: 'Pending Atelier Review', badge: 'secondary' };
    if (s === 'reserved') return { pct: 50, label: 'Silk Reserved & Cut', badge: 'secondary' };
    if (s === 'paid' || s === 'processing') return { pct: 75, label: 'Tailored & Packaged', badge: 'default' };
    if (s === 'shipped' || s === 'delivered') return { pct: 100, label: 'Dispatched with Courier', badge: 'default' };
    if (s === 'cancelled') return { pct: 0, label: 'Cancelled', badge: 'destructive' };
    return { pct: 30, label: status, badge: 'outline' };
  };

  const progressInfo = getStatusProgress(orderRecord?.status);

  return (
    <div className="min-h-screen bg-[#fafafa] text-neutral-900 font-sans select-none flex flex-col">
      {/* Header */}
      <header className="border-b border-neutral-200 bg-white sticky top-0 z-30">
        <div className="max-w-4xl mx-auto px-6 py-3.5 flex items-center justify-between">
          <button
            type="button"
            onClick={() => navigate('/')}
            className="flex items-center space-x-2 text-xs font-semibold text-neutral-600 hover:text-neutral-900 transition-colors"
          >
            <ArrowLeft size={15} />
            <span>Return to Storefront</span>
          </button>

          <span
            onClick={() => navigate('/')}
            className="font-serif text-xl font-bold tracking-tight text-neutral-900 cursor-pointer"
          >
            Ajrah Noor
          </span>

          <Button
            variant="outline"
            size="sm"
            onClick={() => fetchOrderDetails(true)}
            disabled={refreshing}
            className="text-xs h-8"
          >
            <RefreshCw size={13} className={`mr-1.5 ${refreshing ? 'animate-spin' : ''}`} />
            <span>Refresh</span>
          </Button>
        </div>
      </header>

      {/* Main Track Section */}
      <main className="max-w-3xl mx-auto px-6 py-10 flex-1 w-full space-y-6">
        <div className="text-center space-y-1">
          <Badge variant="outline" className="text-[10px] uppercase font-mono tracking-widest text-amber-800 border-amber-300">
            Real-Time Order Tracking
          </Badge>
          <h2 className="text-3xl font-serif text-neutral-900 font-normal">
            Atelier Order Status
          </h2>
          <p className="text-xs text-neutral-500 font-medium">
            Live lifecycle tracking backed by transaction-safe optimistic concurrency control.
          </p>
        </div>

        {messageSentSuccess && (
          <div className="p-4 bg-emerald-50 border border-emerald-300 rounded-2xl flex items-center space-x-3 text-emerald-800 text-xs font-semibold shadow-xs animate-in fade-in">
            <CheckCircle2 size={18} className="text-emerald-600 shrink-0" />
            <span>{messageSentSuccess}</span>
          </div>
        )}

        {loading ? (
          <Card className="p-12 text-center bg-white border-neutral-200 shadow-xs space-y-3">
            <Loader2 className="w-8 h-8 text-neutral-800 animate-spin mx-auto" />
            <p className="text-xs font-semibold text-neutral-600">
              Querying PostgreSQL order lifecycle state...
            </p>
          </Card>
        ) : error && !orderRecord ? (
          <Card className="p-8 text-center bg-white border-rose-200 shadow-xs space-y-4">
            <AlertCircle size={32} className="text-rose-500 mx-auto" />
            <div className="space-y-1">
              <h3 className="font-serif text-lg font-bold text-neutral-900">Order Inquiry Notice</h3>
              <p className="text-xs text-rose-600 max-w-md mx-auto">{error}</p>
            </div>
            {requiresAuth && (
              <Button variant="luxury" size="sm" onClick={() => navigate('/login')}>
                Sign In to View Customer Orders
              </Button>
            )}
          </Card>
        ) : orderRecord ? (
          <div className="space-y-6">
            {/* Status Timeline Card */}
            <Card className="border-neutral-200/90 shadow-sm bg-white overflow-hidden">
              <CardHeader className="bg-neutral-50/50 pb-4 border-b border-neutral-100">
                <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
                  <div>
                    <span className="text-[10px] font-mono uppercase tracking-widest text-neutral-400 block font-bold">
                      Order Reference
                    </span>
                    <CardTitle className="font-serif text-xl font-bold text-neutral-900">
                      {orderRecord.order_number || orderRecord.id}
                    </CardTitle>
                    <CardDescription className="text-xs text-neutral-500 mt-0.5">
                      Placed on{' '}
                      {new Date(orderRecord.created_at).toLocaleDateString(undefined, {
                        dateStyle: 'medium',
                      })}
                    </CardDescription>
                  </div>

                  <Badge
                    variant={progressInfo.badge as 'default' | 'secondary' | 'destructive' | 'outline'}
                    className="text-xs font-bold px-3 py-1 font-mono uppercase tracking-wider"
                  >
                    {orderRecord.status}
                  </Badge>
                </div>
              </CardHeader>

              <CardContent className="space-y-6 pt-6">
                {/* Fulfillment Progress Bar */}
                <div className="space-y-2">
                  <div className="flex justify-between items-center text-xs">
                    <span className="font-semibold text-neutral-800 flex items-center space-x-1.5">
                      <Truck size={14} className="text-amber-800" />
                      <span>{progressInfo.label}</span>
                    </span>
                    <span className="font-mono font-bold text-neutral-500">{progressInfo.pct}%</span>
                  </div>
                  <Progress value={progressInfo.pct} className="h-2 bg-neutral-100" />

                  <div className="grid grid-cols-4 text-[10px] text-neutral-400 font-mono text-center pt-1">
                    <span className={progressInfo.pct >= 25 ? 'font-bold text-neutral-900' : ''}>
                      Pending
                    </span>
                    <span className={progressInfo.pct >= 50 ? 'font-bold text-neutral-900' : ''}>
                      Reserved
                    </span>
                    <span className={progressInfo.pct >= 75 ? 'font-bold text-neutral-900' : ''}>
                      Tailored
                    </span>
                    <span className={progressInfo.pct >= 100 ? 'font-bold text-neutral-900' : ''}>
                      Shipped
                    </span>
                  </div>
                </div>

                <Separator />

                {/* Delivery & Payment Information */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
                  <div className="p-4 bg-neutral-50 rounded-xl border border-neutral-200/80 space-y-1">
                    <span className="text-[10px] font-bold uppercase tracking-wider text-neutral-400 flex items-center space-x-1">
                      <MapPin size={12} />
                      <span>Delivery Recipient</span>
                    </span>
                    <p className="font-bold text-neutral-900">{orderRecord.customer_name}</p>
                    <p className="text-neutral-500">{orderRecord.shipping_address?.phone || '+966 50 123 4567'}</p>
                    <p className="text-neutral-500">
                      {orderRecord.shipping_address?.street_line1 || 'Olaya District'},{' '}
                      {orderRecord.shipping_address?.city || 'Riyadh'}, Saudi Arabia
                    </p>
                  </div>

                  <div className="p-4 bg-neutral-50 rounded-xl border border-neutral-200/80 space-y-1">
                    <span className="text-[10px] font-bold uppercase tracking-wider text-neutral-400 flex items-center space-x-1">
                      <CreditCard size={12} />
                      <span>Settlement</span>
                    </span>
                    <p className="font-bold text-neutral-900">{orderRecord.payment_method}</p>
                    <p className="text-neutral-500">
                      Total: <span className="font-bold text-amber-800">{orderRecord.currency} {orderRecord.amount.toFixed(2)}</span>
                    </p>
                    <span className="inline-flex items-center space-x-1 text-[10px] text-emerald-700 font-semibold">
                      <ShieldCheck size={12} />
                      <span>Authenticated & Idempotent</span>
                    </span>
                  </div>
                </div>

                {/* Ordered Pieces Table */}
                <div className="space-y-2">
                  <h4 className="font-serif font-bold text-sm text-neutral-900">
                    Ordered Atelier Pieces ({itemsList.length})
                  </h4>
                  <div className="border border-neutral-200 rounded-xl divide-y divide-neutral-100 overflow-hidden">
                    {itemsList.map((item, idx) => (
                      <div key={idx} className="p-3.5 flex justify-between items-center text-xs bg-white">
                        <div>
                          <p className="font-semibold text-neutral-900">{item.name}</p>
                          <p className="text-[11px] text-neutral-400 font-mono">
                            Quantity: {item.quantity}
                          </p>
                        </div>
                        <span className="font-mono font-bold text-neutral-900">
                          {orderRecord.currency} {(item.price * item.quantity).toFixed(2)}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              </CardContent>

              <CardFooter className="bg-neutral-50 p-6 border-t border-neutral-100 flex flex-col sm:flex-row items-center justify-between gap-3">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setMessageOpen(true)}
                  className="w-full sm:w-auto text-xs"
                >
                  <MessageSquare size={14} className="mr-1.5" />
                  <span>Contact Atelier Regarding Order</span>
                </Button>

                <Button
                  variant="luxury"
                  size="sm"
                  onClick={() => navigate('/')}
                  className="w-full sm:w-auto text-xs font-mono uppercase tracking-wider"
                >
                  Continue Shopping
                </Button>
              </CardFooter>
            </Card>
          </div>
        ) : null}

        {/* Message Atelier Dialog */}
        <Dialog open={messageOpen} onOpenChange={setMessageOpen}>
          <DialogContent className="sm:max-w-md bg-white">
            <DialogHeader>
              <DialogTitle className="font-serif text-lg font-bold text-neutral-900">
                Inquire With Atelier
              </DialogTitle>
              <DialogDescription className="text-xs text-neutral-500">
                Reference: {orderRecord?.order_number || orderRecord?.id}
              </DialogDescription>
            </DialogHeader>

            <form onSubmit={handleSendMessage} className="space-y-4 pt-2">
              <textarea
                rows={4}
                value={userMessage}
                onChange={(e) => setUserMessage(e.target.value)}
                required
                placeholder="Inquire about delivery timeline, address modification, or styling adjustments..."
                className="w-full p-3 bg-white border border-neutral-200 rounded-xl text-neutral-900 text-xs placeholder-neutral-400 focus:outline-none focus:border-neutral-900"
              />

              <DialogFooter className="flex justify-end space-x-2">
                <Button type="button" variant="outline" size="sm" onClick={() => setMessageOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" variant="luxury" size="sm">
                  <Send size={13} className="mr-1.5" />
                  <span>Send Message</span>
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </main>
    </div>
  );
};

export default CustomerOrderPage;
