import React from 'react';
import { useRootStore } from '@/slicers/root_store';
import Button from '@/shared/ui/Button';
import { useNavigate } from 'react-router-dom';

export const CartDrawer: React.FC<{ open: boolean; onClose: () => void }> = ({ open, onClose }) => {
  const items = useRootStore((s) => s.items);
  const updateQuantity = useRootStore((s) => s.updateQuantity);
  const removeFromCart = useRootStore((s) => s.removeFromCart);
  const clearCart = useRootStore((s) => s.clearCart);
  const navigate = useNavigate();

  if (!open) return null;

  const total = items.reduce((sum, item) => sum + item.price * item.quantity, 0);

  return (
    <div className="fixed inset-0 z-50 flex justify-end bg-black/40 backdrop-blur-sm" onClick={onClose}>
      <div
        className="w-full max-w-md bg-white border-l border-lightgray h-full flex flex-col justify-between p-6 shadow-2xl text-navy animate-in slide-in-from-right duration-300"
        onClick={(e) => e.stopPropagation()}
      >
        <div>
          <div className="flex items-center justify-between pb-4 border-b border-lightgray">
            <h3 className="text-xl font-bold text-navy">Shopping Cart ({items.length})</h3>
            <button onClick={onClose} className="text-slateblue hover:text-navy">
              ✕
            </button>
          </div>

          <div className="mt-4 space-y-4 max-h-[calc(100vh-250px)] overflow-y-auto pr-1">
            {items.length === 0 ? (
              <p className="text-slateblue text-center py-8">Your cart is empty.</p>
            ) : (
              items.map((item) => (
                <div
                  key={item.productId}
                  className="flex items-center justify-between p-3 bg-cream/40 border border-lightgray rounded-lg"
                >
                  <div>
                    <h4 className="font-semibold text-navy text-sm">{item.name}</h4>
                    <p className="text-xs text-slateblue mt-0.5">SAR {item.price} each</p>
                  </div>

                  <div className="flex items-center space-x-2">
                    <button
                      onClick={() => updateQuantity(item.productId, item.quantity - 1)}
                      className="px-2 py-0.5 bg-white border border-lightgray text-navy rounded hover:bg-cream text-xs"
                    >
                      -
                    </button>
                    <span className="text-sm font-semibold text-navy">{item.quantity}</span>
                    <button
                      onClick={() => updateQuantity(item.productId, item.quantity + 1)}
                      className="px-2 py-0.5 bg-white border border-lightgray text-navy rounded hover:bg-cream text-xs"
                    >
                      +
                    </button>
                    <button
                      onClick={() => removeFromCart(item.productId)}
                      className="text-taupe hover:text-navy ml-2 text-xs"
                    >
                      ✕
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="pt-4 border-t border-lightgray space-y-4">
          <div className="flex justify-between items-center text-lg font-bold text-navy">
            <span>Total:</span>
            <span>SAR {total}</span>
          </div>

          <Button
            variant="primary"
            className="w-full"
            disabled={items.length === 0}
            onClick={() => {
              onClose();
              navigate('/checkout');
            }}
          >
            Proceed to Checkout
          </Button>
          {items.length > 0 && (
            <button
              onClick={clearCart}
              className="w-full text-center text-xs text-slateblue hover:text-navy"
            >
              Clear Cart
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default CartDrawer;
