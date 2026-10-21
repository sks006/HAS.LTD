import React from 'react';
import { PaymentGatewayForm } from '@/features/checkout/components/PaymentGatewayForm';
import { useRootStore } from '@/slicers/root_store';
import { useNavigate } from 'react-router-dom';
import { ShoppingBag, ArrowLeft, ShoppingCart } from 'lucide-react';

export const Checkout: React.FC = () => {
  const items = useRootStore((s) => s.items);
  const navigate = useNavigate();

  const total = items.reduce((sum, item) => sum + item.price * item.quantity, 0);

  return (
    <div className="min-h-screen bg-[#fafafa] text-neutral-900 p-6 md:p-12 font-sans select-none">
      <div className="max-w-4xl mx-auto space-y-8">
        <button
          onClick={() => navigate('/')}
          className="text-xs font-bold uppercase tracking-wider text-neutral-600 hover:text-neutral-900 transition-colors flex items-center space-x-2 bg-white px-3.5 py-2 rounded-xl border border-neutral-200 shadow-2xs cursor-pointer"
        >
          <ArrowLeft size={14} />
          <span>Return to Storefront</span>
        </button>

        <div className="flex items-center justify-between border-b border-neutral-200 pb-4">
          <div>
            <h1 className="text-2xl md:text-3xl font-extrabold text-neutral-900 font-serif tracking-tight">
              Checkout & Order Placement
            </h1>
            <p className="text-xs text-neutral-500 mt-1 font-medium">
              Complete your luxury abaya order with instant atelier confirmation.
            </p>
          </div>
          <div className="text-right">
            <span className="text-xs uppercase font-mono text-neutral-400 block font-bold">Items in Bag</span>
            <span className="text-lg font-bold text-neutral-900">{items.length}</span>
          </div>
        </div>

        {items.length === 0 ? (
          <div className="bg-white border border-neutral-200 rounded-2xl p-12 text-center space-y-4 shadow-2xs">
            <div className="w-16 h-16 rounded-full bg-neutral-100 flex items-center justify-center mx-auto text-neutral-400">
              <ShoppingCart size={28} />
            </div>
            <h3 className="text-lg font-bold text-neutral-900 font-serif">Your Shopping Bag is Empty</h3>
            <p className="text-xs text-neutral-500 max-w-sm mx-auto">
              Please select a piece from our Haute Couture collection before proceeding to checkout.
            </p>
            <button
              onClick={() => navigate('/')}
              className="mt-2 bg-neutral-900 hover:bg-neutral-800 text-white font-bold text-xs px-6 py-3 rounded-xl shadow-md transition-all cursor-pointer inline-flex items-center space-x-2"
            >
              <ShoppingBag size={14} />
              <span>Browse Abaya Collection</span>
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 items-start">
            {/* Customer Details Form */}
            <div className="md:col-span-2">
              <PaymentGatewayForm />
            </div>

            {/* Dynamic Order Summary Sidebar */}
            <div className="bg-white border border-neutral-200 rounded-2xl p-6 space-y-5 shadow-2xs sticky top-8">
              <div className="flex items-center justify-between border-b border-neutral-100 pb-3">
                <h3 className="font-bold text-neutral-900 text-base font-serif">Order Summary</h3>
                <span className="text-[10px] font-mono font-bold bg-amber-100 text-amber-800 px-2 py-0.5 rounded-md">
                  {items.length} {items.length === 1 ? 'Item' : 'Items'}
                </span>
              </div>

              {/* Dynamic Items List */}
              <div className="space-y-3.5 divide-y divide-neutral-100 max-h-[300px] overflow-y-auto pr-1">
                {items.map((item, idx) => (
                  <div key={`${item.productId}-${idx}`} className={idx > 0 ? 'pt-3 flex justify-between items-start gap-3 text-xs' : 'flex justify-between items-start gap-3 text-xs'}>
                    <div className="space-y-0.5">
                      <p className="font-bold text-neutral-900 font-sans">{item.name}</p>
                      <p className="text-[11px] text-neutral-500 font-mono">
                        Qty: <span className="font-bold text-neutral-800">{item.quantity}</span> × SAR {item.price.toFixed(2)}
                      </p>
                    </div>
                    <span className="font-bold text-neutral-900 shrink-0 font-mono">
                      SAR {(item.price * item.quantity).toFixed(2)}
                    </span>
                  </div>
                ))}
              </div>

              {/* Total Calculation */}
              <div className="pt-4 border-t border-neutral-200 space-y-2">
                <div className="flex justify-between text-xs text-neutral-500">
                  <span>Subtotal</span>
                  <span className="font-mono">SAR {total.toFixed(2)}</span>
                </div>
                <div className="flex justify-between text-xs text-neutral-500">
                  <span>Shipping & Atelier Tailoring</span>
                  <span className="font-bold text-emerald-600 uppercase text-[10px]">Complimentary</span>
                </div>
                <div className="pt-2 border-t border-neutral-100 flex justify-between font-extrabold text-base text-neutral-900">
                  <span>Total Amount:</span>
                  <span className="font-serif text-amber-700">SAR {total.toFixed(2)}</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default Checkout;
