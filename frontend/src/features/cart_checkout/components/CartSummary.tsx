import React from 'react';
import { ProductDto } from '../../../shared/types/contracts';
import { Button } from '../../../shared/ui/Button';

interface CartSummaryProps {
  items: { product: ProductDto; quantity: number }[];
  onRemove: (id: string) => void;
  onCheckout: () => void;
  isSubmitting: boolean;
}

export function CartSummary({ items, onRemove, onCheckout, isSubmitting }: CartSummaryProps) {
  const total = items.reduce((acc, curr) => acc + (curr.product.price_cents * curr.quantity), 0);

  return (
    <div className="bg-slate-900 border border-slate-800 rounded-xl p-6 shadow-xl text-white">
      <h3 className="text-xl font-bold tracking-tight mb-4">Cart Summary</h3>
      {items.length === 0 ? (
        <p className="text-slate-500 text-sm">Your cart is empty.</p>
      ) : (
        <div className="space-y-4">
          <ul className="divide-y divide-slate-800 max-h-60 overflow-y-auto pr-1">
            {items.map((item) => (
              <li key={item.product.id} className="py-3 flex justify-between items-center text-sm">
                <div>
                  <span className="font-semibold block">{item.product.name}</span>
                  <span className="text-slate-400 font-mono">Qty: {item.quantity}</span>
                </div>
                <div className="flex items-center space-x-3">
                  <span className="font-mono">${((item.product.price_cents * item.quantity) / 100).toFixed(2)}</span>
                  <button 
                    onClick={() => onRemove(item.product.id)}
                    className="text-red-500 hover:text-red-400 text-xs"
                  >
                    Remove
                  </button>
                </div>
              </li>
            ))}
          </ul>
          
          <div className="border-t border-slate-800 pt-4 flex justify-between items-center font-bold">
            <span>Total:</span>
            <span className="font-mono text-lg text-emerald-400">${(total / 100).toFixed(2)}</span>
          </div>

          <Button 
            variant="primary" 
            className="w-full mt-2" 
            onClick={onCheckout}
            disabled={isSubmitting}
          >
            {isSubmitting ? 'Processing...' : 'Proceed to Checkout'}
          </Button>
        </div>
      )}
    </div>
  );
}
