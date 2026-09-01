import React from 'react';
import { PaymentGatewayForm } from '@/features/checkout/components/PaymentGatewayForm';
import { useRootStore } from '@/slicers/root_store';
import { useNavigate } from 'react-router-dom';

export const Checkout: React.FC = () => {
  const items = useRootStore((s) => s.items);
  const navigate = useNavigate();

  const total = items.reduce((sum, item) => sum + item.price * item.quantity, 0);

  return (
    <div className="min-h-screen bg-white text-navy p-6 md:p-12 font-sans">
      <div className="max-w-4xl mx-auto space-y-8">
        <button
          onClick={() => navigate('/')}
          className="text-sm text-slateblue hover:text-navy transition-colors flex items-center space-x-2 font-medium"
        >
          <span>←</span> <span>Return to Storefront</span>
        </button>

        <h1 className="text-3xl font-extrabold text-navy">Checkout & Order Placement</h1>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div className="md:col-span-2">
            <PaymentGatewayForm />
          </div>

          <div className="bg-white border border-lightgray rounded-2xl p-6 h-fit space-y-4 shadow-sm">
            <h3 className="font-bold text-navy text-lg">Order Summary</h3>

            <div className="space-y-3 divide-y divide-lightgray text-sm">
              {items.map((item) => (
                <div key={item.productId} className="pt-3 flex justify-between">
                  <div>
                    <p className="font-medium text-navy">{item.name}</p>
                    <p className="text-xs text-slateblue">Qty: {item.quantity}</p>
                  </div>
                  <span className="font-semibold text-navy">
                    SAR {item.price * item.quantity}
                  </span>
                </div>
              ))}
            </div>

            <div className="pt-4 border-t border-lightgray flex justify-between font-bold text-base text-navy">
              <span>Total Amount:</span>
              <span>SAR {total}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Checkout;
