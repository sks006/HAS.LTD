import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useRootStore } from '@/slicers/root_store';
import { useAuth } from '@/app/providers/AuthProvider';
import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetFooter,
} from '@/shared/ui/sheet';
import { Button } from '@/shared/ui/button';
import { Badge } from '@/shared/ui/badge';
import { Separator } from '@/shared/ui/separator';
import {
  ShoppingBag,
  Trash2,
  Plus,
  Minus,
  ArrowRight,
  ShieldCheck,
  Package,
} from 'lucide-react';

interface CartDrawerProps {
  open: boolean;
  onClose: () => void;
}

export const CartDrawer: React.FC<CartDrawerProps> = ({ open, onClose }) => {
  const items = useRootStore((s) => s.items);
  const products = useRootStore((s) => s.products);
  const updateQuantity = useRootStore((s) => s.updateQuantity);
  const removeFromCart = useRootStore((s) => s.removeFromCart);
  const clearCart = useRootStore((s) => s.clearCart);
  const token = useRootStore((s) => s.token);
  const { user } = useAuth();
  const navigate = useNavigate();

  const total = items.reduce((sum, item) => sum + item.price * item.quantity, 0);
  const totalItemCount = items.reduce((sum, item) => sum + item.quantity, 0);

  // Sync cart mutations to backend /cart/sync when customer is logged in
  useEffect(() => {
    if (!open || !token || items.length === 0) return;

    const syncWithBackend = async () => {
      try {
        await apiFetch(ENDPOINTS.CART_SYNC, {
          method: 'POST',
          token,
          body: items.map((it) => ({
            product_id: it.productId,
            variant_id: null,
            quantity_delta: it.quantity,
          })),
        });
      } catch (err) {
        // Soft fail on background cart sync, user can still proceed to checkout
        console.debug('Background cart sync deferred:', err);
      }
    };

    const timer = setTimeout(syncWithBackend, 400);
    return () => clearTimeout(timer);
  }, [items, open, token]);

  return (
    <Sheet open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <SheetContent
        side="right"
        className="w-full sm:max-w-md flex flex-col p-0 bg-white border-l border-neutral-200"
      >
        {/* Header */}
        <SheetHeader className="p-6 pb-4 border-b border-neutral-100 bg-neutral-50/50">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2.5">
              <div className="w-9 h-9 rounded-xl bg-[#1c1917] text-white flex items-center justify-center shadow-xs">
                <ShoppingBag size={18} className="text-amber-300" />
              </div>
              <div>
                <SheetTitle className="font-serif text-lg font-bold text-neutral-900 tracking-tight">
                  Shopping Bag
                </SheetTitle>
                <SheetDescription className="text-[11px] text-neutral-500 font-medium">
                  Atelier Haute Couture Selection
                </SheetDescription>
              </div>
            </div>
            {totalItemCount > 0 && (
              <Badge variant="secondary" className="font-mono text-xs font-semibold px-2.5 py-0.5">
                {totalItemCount} {totalItemCount === 1 ? 'piece' : 'pieces'}
              </Badge>
            )}
          </div>
        </SheetHeader>

        {/* Content Line Items */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
          {items.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-center p-8 space-y-4 my-auto">
              <div className="w-16 h-16 rounded-full bg-neutral-100 flex items-center justify-center text-neutral-400">
                <Package size={28} strokeWidth={1.5} />
              </div>
              <div className="space-y-1">
                <h4 className="font-serif font-bold text-neutral-900 text-base">Your Bag is Empty</h4>
                <p className="text-xs text-neutral-500 max-w-xs leading-relaxed">
                  Discover our newest handcrafted silks, velvets, and tailored abayas.
                </p>
              </div>
              <Button
                variant="luxury"
                size="sm"
                onClick={onClose}
                className="mt-2 font-mono uppercase tracking-wider text-[11px]"
              >
                Browse Collection
              </Button>
            </div>
          ) : (
            <div className="divide-y divide-neutral-100">
              {items.map((item) => {
                const productInfo = products.find((p) => p.id === item.productId);
                const imageUrl =
                  productInfo?.image ||
                  'https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?auto=format&fit=crop&q=80&w=300';

                return (
                  <div key={item.productId} className="py-4 first:pt-0 flex items-center gap-4 group">
                    <div className="w-16 h-20 rounded-xl overflow-hidden bg-neutral-100 shrink-0 border border-neutral-200/80">
                      <img
                        src={imageUrl}
                        alt={item.name}
                        className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                      />
                    </div>

                    <div className="flex-1 min-w-0 space-y-1">
                      <h4 className="font-serif font-semibold text-neutral-900 text-sm truncate">
                        {item.name}
                      </h4>
                      <p className="font-mono text-xs font-bold text-amber-800">
                        SAR {item.price.toFixed(2)}
                      </p>

                      {/* Quantity Controls */}
                      <div className="flex items-center space-x-2 pt-1">
                        <div className="flex items-center border border-neutral-200 rounded-lg bg-neutral-50">
                          <button
                            type="button"
                            onClick={() => updateQuantity(item.productId, item.quantity - 1)}
                            className="w-7 h-7 flex items-center justify-center text-neutral-600 hover:text-neutral-900 transition-colors"
                            title="Decrease quantity"
                          >
                            <Minus size={12} />
                          </button>
                          <span className="w-7 text-center text-xs font-mono font-bold text-neutral-900">
                            {item.quantity}
                          </span>
                          <button
                            type="button"
                            onClick={() => updateQuantity(item.productId, item.quantity + 1)}
                            className="w-7 h-7 flex items-center justify-center text-neutral-600 hover:text-neutral-900 transition-colors"
                            title="Increase quantity"
                          >
                            <Plus size={12} />
                          </button>
                        </div>

                        <button
                          type="button"
                          onClick={() => removeFromCart(item.productId)}
                          className="p-1.5 text-neutral-400 hover:text-rose-600 rounded-md transition-colors"
                          title="Remove item"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </div>

                    <div className="text-right">
                      <span className="font-mono font-bold text-xs text-neutral-900">
                        SAR {(item.price * item.quantity).toFixed(2)}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Footer with Subtotal & Checkout Action */}
        {items.length > 0 && (
          <SheetFooter className="p-6 bg-neutral-50 border-t border-neutral-200/80 space-y-4">
            <div className="w-full space-y-2">
              <div className="flex justify-between items-center text-xs text-neutral-500 font-medium">
                <span>Shipping & Insurance</span>
                <span className="text-emerald-700 font-bold uppercase text-[10px] tracking-wider">
                  Complimentary
                </span>
              </div>
              <div className="flex justify-between items-center text-sm font-medium text-neutral-700">
                <span>Estimated Total</span>
                <span className="font-serif font-extrabold text-base text-neutral-900">
                  SAR {total.toFixed(2)}
                </span>
              </div>
            </div>

            <Separator />

            <div className="w-full space-y-2">
              <Button
                variant="luxury"
                size="lg"
                className="w-full justify-between font-mono uppercase tracking-wider text-xs"
                onClick={() => {
                  onClose();
                  navigate('/checkout');
                }}
              >
                <span>Proceed to Checkout</span>
                <ArrowRight size={14} />
              </Button>

              <div className="flex items-center justify-between text-[11px] text-neutral-400 pt-1">
                <span className="inline-flex items-center gap-1">
                  <ShieldCheck size={13} className="text-emerald-600" />
                  <span>Secure 256-bit checkout</span>
                </span>
                <button
                  type="button"
                  onClick={clearCart}
                  className="hover:text-rose-600 underline transition-colors cursor-pointer"
                >
                  Clear Bag
                </button>
              </div>
            </div>
          </SheetFooter>
        )}
      </SheetContent>
    </Sheet>
  );
};

export default CartDrawer;
