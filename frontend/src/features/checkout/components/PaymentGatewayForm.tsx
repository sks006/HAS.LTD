import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useRootStore } from '@/slicers/root_store';
import { useAuth } from '@/app/providers/AuthProvider';
import { apiFetch, ApiError } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from '@/shared/ui/card';
import { Input } from '@/shared/ui/input';
import { Label } from '@/shared/ui/label';
import { Button } from '@/shared/ui/button';
import { Badge } from '@/shared/ui/badge';
import { Separator } from '@/shared/ui/separator';
import { Alert, AlertTitle, AlertDescription } from '@/shared/ui/alert';
import {
  CheckCircle2,
  Lock,
  CreditCard,
  Banknote,
  ArrowRight,
  ShieldCheck,
  Package,
} from 'lucide-react';

export const PaymentGatewayForm: React.FC<{ onSuccess?: (orderId: string) => void }> = ({
  onSuccess,
}) => {
  const navigate = useNavigate();
  const items = useRootStore((s) => s.items);
  const clearCart = useRootStore((s) => s.clearCart);
  const token = useRootStore((s) => s.token);
  const { user } = useAuth();

  // Form Fields
  const [recipientName, setRecipientName] = useState(user?.email?.split('@')[0] || '');
  const [phone, setPhone] = useState('+966 50 123 4567');
  const [streetLine1, setStreetLine1] = useState('Olaya Street, District 4');
  const [city, setCity] = useState('Riyadh');
  const [postalCode, setPostalCode] = useState('12211');
  const [deliveryNotes, setDeliveryNotes] = useState('Leave with reception');
  const [paymentMethod, setPaymentMethod] = useState<'card' | 'cod'>('cod');

  // Card details mock
  const [cardNumber, setCardNumber] = useState('•••• •••• •••• 4242');
  const [cardExpiry, setCardExpiry] = useState('12/28');
  const [cardCvc, setCardCvc] = useState('888');

  // Processing state
  const [loading, setLoading] = useState(false);
  const [completedOrderId, setCompletedOrderId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const totalAmount = items.reduce((sum, i) => sum + i.price * i.quantity, 0);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (items.length === 0) {
      setError('Your shopping bag is empty.');
      return;
    }

    if (!token) {
      setError('Authentication required: Please sign in to finalize your luxury purchase.');
      return;
    }

    setLoading(true);
    setError(null);

    const idempotencyKey = `ord-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;

    const shippingAddress = {
      recipient_name: recipientName,
      phone,
      street_line1: streetLine1,
      city,
      postal_code: postalCode,
      country: 'Saudi Arabia',
      delivery_instructions: deliveryNotes || undefined,
    };

    try {
      const response = await apiFetch<{ ok: boolean; order_id: string; message: string }>(
        ENDPOINTS.CHECKOUT,
        {
          method: 'POST',
          token,
          headers: {
            'x-idempotency-key': idempotencyKey,
          },
          body: {
            currency: 'SAR',
            shipping_address: shippingAddress,
            billing_address: shippingAddress,
            items: items.map((it) => ({
              product_id: it.productId,
              price_minor_units: Math.round((it.price || 0) * 100),
              product_name: it.name,
              sku: it.productId,
              quantity: it.quantity,
            })),
          },
        }
      );

      if (!response.ok || !response.order_id) {
        throw new Error(response.message || 'The atelier backend could not confirm this order.');
      }

      setCompletedOrderId(response.order_id);
      clearCart();
      onSuccess?.(response.order_id);
    } catch (err: unknown) {
      if (err instanceof ApiError && err.data && typeof err.data === 'object') {
        const d = err.data as { message?: string; error?: { message?: string } };
        setError(d.error?.message || d.message || err.message);
      } else if (err instanceof Error) {
        setError(err.message);
      } else {
        setError('Checkout could not be completed. Please verify your connection.');
      }
    } finally {
      setLoading(false);
    }
  };

  if (completedOrderId) {
    return (
      <Card className="border-emerald-200 bg-emerald-50/40 shadow-sm text-neutral-900">
        <CardHeader className="text-center pb-2">
          <div className="w-14 h-14 bg-emerald-100 text-emerald-700 rounded-2xl flex items-center justify-center mx-auto mb-2">
            <CheckCircle2 size={30} />
          </div>
          <Badge variant="outline" className="mx-auto border-emerald-300 text-emerald-800 bg-white">
            Order Confirmed & Reserved
          </Badge>
          <CardTitle className="font-serif text-2xl font-bold text-neutral-900 mt-2">
            Thank You For Your Order
          </CardTitle>
          <CardDescription className="text-neutral-600 text-xs">
            Your piece is now queued for tailored inspection and dispatch.
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4 pt-4 text-center">
          <div className="p-4 bg-white border border-emerald-200 rounded-xl space-y-1">
            <span className="text-[10px] uppercase font-mono tracking-widest text-neutral-400 block font-bold">
              Unique Order Identifier
            </span>
            <span className="font-mono text-sm font-bold text-neutral-900 break-all">
              {completedOrderId}
            </span>
          </div>

          <div className="grid grid-cols-2 gap-3 text-left text-xs">
            <div className="p-3 bg-white/80 rounded-lg border border-neutral-200/60">
              <span className="text-neutral-400 block text-[10px] uppercase font-bold">Recipient</span>
              <span className="font-semibold text-neutral-800">{recipientName}</span>
            </div>
            <div className="p-3 bg-white/80 rounded-lg border border-neutral-200/60">
              <span className="text-neutral-400 block text-[10px] uppercase font-bold">City</span>
              <span className="font-semibold text-neutral-800">{city}, Saudi Arabia</span>
            </div>
          </div>
        </CardContent>

        <CardFooter className="flex flex-col sm:flex-row gap-3 pt-2">
          <Button
            variant="luxury"
            className="w-full font-mono text-xs uppercase tracking-wider"
            onClick={() => navigate(`/orders/${completedOrderId}`)}
          >
            <span>Track Order Status</span>
            <ArrowRight size={14} className="ml-1.5" />
          </Button>
          <Button
            variant="outline"
            className="w-full font-mono text-xs uppercase tracking-wider bg-white"
            onClick={() => navigate('/')}
          >
            Back to Storefront
          </Button>
        </CardFooter>
      </Card>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {error && (
        <Alert variant="destructive">
          <AlertTitle className="text-xs font-bold">Checkout Notice</AlertTitle>
          <AlertDescription className="text-xs">{error}</AlertDescription>
        </Alert>
      )}

      {/* Shipping Information Card */}
      <Card className="border-neutral-200/80 shadow-xs bg-white">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <CardTitle className="font-serif text-lg font-bold text-neutral-900">
              1. Delivery Address & Recipient
            </CardTitle>
            <Badge variant="secondary" className="text-[10px] font-mono">
              Saudi Arabia
            </Badge>
          </div>
          <CardDescription className="text-xs text-neutral-500">
            Provide the destination for your courier delivery.
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="recipientName" className="text-xs font-semibold text-neutral-700">
                Full Recipient Name
              </Label>
              <Input
                id="recipientName"
                value={recipientName}
                onChange={(e) => setRecipientName(e.target.value)}
                placeholder="e.g. Princess Noura"
                required
                className="text-xs"
              />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="phone" className="text-xs font-semibold text-neutral-700">
                Mobile Phone
              </Label>
              <Input
                id="phone"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                placeholder="+966 5x xxx xxxx"
                required
                className="text-xs"
              />
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="streetLine1" className="text-xs font-semibold text-neutral-700">
              Street & Villa / Apartment No.
            </Label>
            <Input
              id="streetLine1"
              value={streetLine1}
              onChange={(e) => setStreetLine1(e.target.value)}
              placeholder="e.g. Olaya St, Building 14, Apt 4B"
              required
              className="text-xs"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="city" className="text-xs font-semibold text-neutral-700">
                City / Region
              </Label>
              <Input
                id="city"
                value={city}
                onChange={(e) => setCity(e.target.value)}
                placeholder="Riyadh, Jeddah, Dammam..."
                required
                className="text-xs"
              />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="postalCode" className="text-xs font-semibold text-neutral-700">
                Postal Code
              </Label>
              <Input
                id="postalCode"
                value={postalCode}
                onChange={(e) => setPostalCode(e.target.value)}
                placeholder="12211"
                required
                className="text-xs"
              />
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="notes" className="text-xs font-semibold text-neutral-700">
              Atelier Delivery Notes (Optional)
            </Label>
            <Input
              id="notes"
              value={deliveryNotes}
              onChange={(e) => setDeliveryNotes(e.target.value)}
              placeholder="e.g. Gate code, deliver between 2-5pm"
              className="text-xs"
            />
          </div>
        </CardContent>
      </Card>

      {/* Payment Method Card */}
      <Card className="border-neutral-200/80 shadow-xs bg-white">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <CardTitle className="font-serif text-lg font-bold text-neutral-900">
              2. Settlement Method
            </CardTitle>
            <span className="flex items-center space-x-1 text-[11px] text-neutral-400 font-mono">
              <ShieldCheck size={14} className="text-emerald-600" />
              <span>TLS 1.3 Verified</span>
            </span>
          </div>
          <CardDescription className="text-xs text-neutral-500">
            Choose your preferred checkout settlement.
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <button
              type="button"
              onClick={() => setPaymentMethod('cod')}
              className={`p-3.5 rounded-xl border text-left transition-all flex flex-col justify-between space-y-2 cursor-pointer ${
                paymentMethod === 'cod'
                  ? 'border-neutral-900 bg-neutral-50 ring-1 ring-neutral-900'
                  : 'border-neutral-200 bg-white hover:border-neutral-300'
              }`}
            >
              <div className="flex items-center justify-between">
                <Banknote size={18} className={paymentMethod === 'cod' ? 'text-amber-700' : 'text-neutral-400'} />
                {paymentMethod === 'cod' && (
                  <Badge variant="default" className="text-[9px] px-1.5 py-0 bg-neutral-900">
                    Selected
                  </Badge>
                )}
              </div>
              <div>
                <p className="font-semibold text-xs text-neutral-900">Cash on Delivery</p>
                <p className="text-[10px] text-neutral-500">Pay when tailored garment arrives</p>
              </div>
            </button>

            <button
              type="button"
              onClick={() => setPaymentMethod('card')}
              className={`p-3.5 rounded-xl border text-left transition-all flex flex-col justify-between space-y-2 cursor-pointer ${
                paymentMethod === 'card'
                  ? 'border-neutral-900 bg-neutral-50 ring-1 ring-neutral-900'
                  : 'border-neutral-200 bg-white hover:border-neutral-300'
              }`}
            >
              <div className="flex items-center justify-between">
                <CreditCard size={18} className={paymentMethod === 'card' ? 'text-amber-700' : 'text-neutral-400'} />
                {paymentMethod === 'card' && (
                  <Badge variant="default" className="text-[9px] px-1.5 py-0 bg-neutral-900">
                    Selected
                  </Badge>
                )}
              </div>
              <div>
                <p className="font-semibold text-xs text-neutral-900">Debit / Credit Card</p>
                <p className="text-[10px] text-neutral-500">Mada, Visa, Mastercard, AMEX</p>
              </div>
            </button>
          </div>

          {paymentMethod === 'card' && (
            <div className="p-4 bg-neutral-50 rounded-xl border border-neutral-200 space-y-3 animate-in fade-in">
              <div className="space-y-1">
                <Label htmlFor="cardNumber" className="text-[11px] font-semibold text-neutral-600">
                  Card Number
                </Label>
                <Input
                  id="cardNumber"
                  value={cardNumber}
                  onChange={(e) => setCardNumber(e.target.value)}
                  className="bg-white text-xs font-mono"
                  placeholder="4242 4242 4242 4242"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label htmlFor="expiry" className="text-[11px] font-semibold text-neutral-600">
                    Expiry MM/YY
                  </Label>
                  <Input
                    id="expiry"
                    value={cardExpiry}
                    onChange={(e) => setCardExpiry(e.target.value)}
                    className="bg-white text-xs font-mono"
                    placeholder="12/28"
                  />
                </div>
                <div className="space-y-1">
                  <Label htmlFor="cvc" className="text-[11px] font-semibold text-neutral-600">
                    CVC Code
                  </Label>
                  <Input
                    id="cvc"
                    value={cardCvc}
                    onChange={(e) => setCardCvc(e.target.value)}
                    className="bg-white text-xs font-mono"
                    placeholder="888"
                  />
                </div>
              </div>
            </div>
          )}
        </CardContent>

        <CardFooter className="pt-2 border-t border-neutral-100 flex flex-col space-y-3">
          <Button
            type="submit"
            variant="luxury"
            size="lg"
            className="w-full font-mono text-xs uppercase tracking-wider justify-between"
            disabled={loading || items.length === 0}
            isLoading={loading}
          >
            <span>Confirm Order Placement</span>
            <span className="font-serif normal-case font-bold text-amber-200">
              SAR {totalAmount.toFixed(2)}
            </span>
          </Button>

          <p className="text-center text-[10px] text-neutral-400">
            By confirming, you agree to the Ajrah Noor Atelier Bespoke Order Terms & Conditions.
          </p>
        </CardFooter>
      </Card>
    </form>
  );
};

export default PaymentGatewayForm;
