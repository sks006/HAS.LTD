import React from 'react';
import { PaymentGatewayForm } from '@/features/checkout/components/PaymentGatewayForm';
import { useRootStore } from '@/slicers/root_store';
import { useNavigate } from 'react-router-dom';
import { ShoppingBag, ArrowLeft, ShoppingCart, ShieldCheck } from 'lucide-react';
import { Button } from '@/shared/ui/button';
import { Badge } from '@/shared/ui/badge';
import { Card, CardHeader, CardTitle, CardContent } from '@/shared/ui/card';
import { Separator } from '@/shared/ui/separator';

export const Checkout: React.FC = () => {
  const items = useRootStore((s) => s.items);
  const navigate = useNavigate();

  const total = items.reduce((sum, item) => sum + item.price * item.quantity, 0);

  return (
    <div className="min-h-screen bg-[#fafafa] text-neutral-900 p-6 md:p-12 font-sans select-none">
      <div className="max-w-4xl mx-auto space-y-8">
        <Button
          variant="outline"
          size="sm"
          onClick={() => navigate('/')}
          className="text-xs bg-white border-neutral-200"
        >
          <ArrowLeft size={14} className="mr-1.5" />
          <span>Return to Storefront</span>
        </Button>

        <div className="flex items-center justify-between border-b border-neutral-200 pb-4">
          <div>
            <h1 className="text-2xl md:text-3xl font-extrabold text-neutral-900 font-serif tracking-tight">
              Checkout & Order Confirmation
            </h1>
            <p className="text-xs text-neutral-500 mt-1 font-medium">
              Complete your bespoke luxury abaya order with instant confirmation.
            </p>
          </div>
          <div className="text-right">
            <span className="text-[10px] uppercase font-mono text-neutral-400 block font-bold">
              Items in Bag
            </span>
            <Badge variant="secondary" className="text-sm font-bold font-mono">
              {items.length} {items.length === 1 ? 'piece' : 'pieces'}
            </Badge>
          </div>
        </div>

        {items.length === 0 ? (
          <Card className="border-neutral-200 bg-white p-12 text-center space-y-4 shadow-2xs">
            <div className="w-16 h-16 rounded-full bg-neutral-100 flex items-center justify-center mx-auto text-neutral-400">
              <ShoppingCart size={28} />
            </div>
            <h3 className="text-lg font-bold text-neutral-900 font-serif">Your Shopping Bag is Empty</h3>
            <p className="text-xs text-neutral-500 max-w-sm mx-auto">
              Please select a piece from our Haute Couture collection before proceeding to checkout.
            </p>
            <Button
              variant="luxury"
              onClick={() => navigate('/')}
              className="mt-2 font-mono uppercase tracking-wider text-xs"
            >
              <ShoppingBag size={14} className="mr-2" />
              <span>Browse Abaya Collection</span>
            </Button>
          </Card>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 items-start">
            {/* Customer Details Form */}
            <div className="md:col-span-2">
              <PaymentGatewayForm />
            </div>

            {/* Dynamic Order Summary Sidebar */}
            <Card className="border-neutral-200 bg-white shadow-2xs sticky top-8">
              <CardHeader className="p-5 pb-3 border-b border-neutral-100 flex flex-row items-center justify-between">
                <CardTitle className="font-bold text-neutral-900 text-sm font-serif">
                  Order Summary
                </CardTitle>
                <Badge variant="outline" className="text-[10px] font-mono border-amber-300 text-amber-800">
                  {items.length} {items.length === 1 ? 'Piece' : 'Pieces'}
                </Badge>
              </CardHeader>

              <CardContent className="p-5 space-y-4">
                {/* Dynamic Items List */}
                <div className="space-y-3.5 divide-y divide-neutral-100 max-h-[300px] overflow-y-auto pr-1">
                  {items.map((item, idx) => (
                    <div
                      key={`${item.productId}-${idx}`}
                      className={
                        idx > 0
                          ? 'pt-3 flex justify-between items-start gap-3 text-xs'
                          : 'flex justify-between items-start gap-3 text-xs'
                      }
                    >
                      <div className="space-y-0.5">
                        <p className="font-bold text-neutral-900 font-sans">{item.name}</p>
                        <p className="text-[11px] text-neutral-500 font-mono">
                          Qty: <span className="font-bold text-neutral-800">{item.quantity}</span> × SAR{' '}
                          {item.price.toFixed(2)}
                        </p>
                      </div>
                      <span className="font-bold text-neutral-900 shrink-0 font-mono">
                        SAR {(item.price * item.quantity).toFixed(2)}
                      </span>
                    </div>
                  ))}
                </div>

                <Separator />

                {/* Subtotals & Delivery */}
                <div className="space-y-2 text-xs">
                  <div className="flex justify-between text-neutral-500">
                    <span>Subtotal</span>
                    <span className="font-mono text-neutral-900">SAR {total.toFixed(2)}</span>
                  </div>
                  <div className="flex justify-between text-neutral-500">
                    <span>Atelier Courier Shipping</span>
                    <span className="font-bold text-emerald-700 uppercase text-[10px]">Complimentary</span>
                  </div>
                  <div className="flex justify-between text-neutral-500">
                    <span>Custom Tailoring Guarantee</span>
                    <span className="font-semibold text-neutral-700">Included</span>
                  </div>

                  <Separator />

                  <div className="pt-1 flex justify-between items-baseline font-extrabold text-base text-neutral-900">
                    <span>Total Amount:</span>
                    <span className="font-serif text-amber-800 text-lg">SAR {total.toFixed(2)}</span>
                  </div>
                </div>

                <div className="p-3 bg-neutral-50 rounded-xl border border-neutral-200/80 text-[10px] text-neutral-500 flex items-center space-x-2">
                  <ShieldCheck size={16} className="text-emerald-600 shrink-0" />
                  <span>Transaction protected by cryptographic idempotency deduplication.</span>
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
};

export default Checkout;
