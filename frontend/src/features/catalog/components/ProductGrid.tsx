import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useRootStore } from '@/slicers/root_store';
import { useProducts } from '@/features/catalog/hooks/useProducts';
import { ProductDto } from '@/shared/types/contracts';
import Button from '@/shared/ui/Button';
import Modal from '@/shared/ui/Modal';
import {
  MessageSquare,
  Send,
  CheckCircle2,
  Loader2,
  PackageX,
  Heart,
  Plus,
  ArrowRight,
  ShoppingBag,
} from 'lucide-react';

export const ProductGrid: React.FC = () => {
  const { isLoading } = useProducts();
  const products = useRootStore((s) => s.products);
  const activeFilter = useRootStore((s) => s.activeFilter);
  const searchQuery = useRootStore((s) => s.searchQuery);
  const addToCart = useRootStore((s) => s.addToCart);
  const navigate = useNavigate();

  const [messageProduct, setMessageProduct] = useState<ProductDto | null>(null);
  const [userMessage, setUserMessage] = useState('');
  const [sentSuccess, setSentSuccess] = useState<string | null>(null);
  const [wishlist, setWishlist] = useState<string[]>([]);

  const toggleWishlist = (id: string) => {
    setWishlist((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    );
  };

  const filteredProducts = products.filter((product) => {
    const matchesFilter =
      activeFilter === 'All Abayas' ||
      activeFilter === 'All pieces' ||
      product.category === activeFilter;
    const matchesSearch = product.name
      .toLowerCase()
      .includes(searchQuery.toLowerCase());
    return matchesFilter && matchesSearch;
  });

  const handleSendMessage = (e: React.FormEvent) => {
    e.preventDefault();
    if (!messageProduct) return;

    const productName = messageProduct.name;
    setMessageProduct(null);
    setUserMessage('');
    setSentSuccess(`Message sent to Ajrah Noor Atelier for "${productName}"!`);
    setTimeout(() => setSentSuccess(null), 4000);
  };

  if (isLoading && products.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-20 bg-white border border-lightgray rounded-2xl p-8 space-y-3 font-sans">
        <Loader2 className="w-8 h-8 text-[#7c3aed] animate-spin" />
        <p className="font-semibold text-navy text-sm">Fetching atelier collection...</p>
      </div>
    );
  }

  if (filteredProducts.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-slateblue bg-white border border-lightgray rounded-2xl p-8 space-y-2 font-sans">
        <PackageX className="w-10 h-10 text-slateblue/40" />
        <p className="font-semibold text-navy text-sm">No products found in backend catalog.</p>
        <p className="text-xs text-slateblue">
          Use the Admin Dashboard to populate items.
        </p>
      </div>
    );
  }

  return (
    <>
      {sentSuccess && (
        <div className="mb-6 p-4 bg-emerald-50 border border-emerald-300 rounded-2xl flex items-center space-x-3 text-emerald-800 text-xs font-semibold shadow-sm animate-in fade-in font-sans">
          <CheckCircle2 size={18} className="text-emerald-600 shrink-0" />
          <span>{sentSuccess}</span>
        </div>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-8">
        {filteredProducts.map((product) => (
          <ProductCardItem
            key={product.id}
            product={product}
            isWishlisted={wishlist.includes(product.id)}
            onWishlist={() => toggleWishlist(product.id)}
            onAddToCart={() => {
              addToCart({
                productId: product.id,
                name: product.name,
                quantity: 1,
                price: product.price,
              });
              setSentSuccess(`Added "${product.name}" to your bag.`);
              setTimeout(() => setSentSuccess(null), 3000);
            }}
            onBuyNow={() => {
              addToCart({
                productId: product.id,
                name: product.name,
                quantity: 1,
                price: product.price,
              });
              navigate('/checkout');
            }}
            onOpenMessage={() => setMessageProduct(product)}
          />
        ))}
      </div>

      {/* Message Atelier Modal */}
      <Modal
        isOpen={Boolean(messageProduct)}
        onClose={() => setMessageProduct(null)}
        title={`Message Atelier - ${messageProduct?.name || ''}`}
      >
        <form onSubmit={handleSendMessage} className="space-y-4 pt-2 font-sans">
          <div className="p-3 bg-cream/50 border border-lightgray rounded-xl flex items-center space-x-3">
            <img
              src={messageProduct?.image}
              alt={messageProduct?.alt}
              className="w-12 h-14 object-cover rounded-lg shrink-0"
            />
            <div className="text-xs">
              <p className="font-bold text-navy">{messageProduct?.name}</p>
              <p className="text-slateblue">
                {messageProduct?.category} • SAR {messageProduct?.price.toLocaleString()}
              </p>
            </div>
          </div>

          <div>
            <label className="block text-xs font-bold uppercase tracking-wider text-slateblue mb-1">
              Your Inquiry or Custom Sizing Request
            </label>
            <textarea
              rows={4}
              value={userMessage}
              onChange={(e) => setUserMessage(e.target.value)}
              required
              placeholder="Inquire about custom tailoring, silk color options, or delivery schedule..."
              className="w-full p-3 bg-white border border-lightgray rounded-xl text-navy text-xs placeholder-slateblue/40 focus:outline-none focus:border-navy"
            />
          </div>

          <div className="flex space-x-3 justify-end pt-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setMessageProduct(null)}
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
              <span>Send Inquiry</span>
            </Button>
          </div>
        </form>
      </Modal>
    </>
  );
};

const ProductCardItem: React.FC<{
  product: ProductDto;
  isWishlisted: boolean;
  onWishlist: () => void;
  onAddToCart: () => void;
  onBuyNow: () => void;
  onOpenMessage: () => void;
}> = ({ product, isWishlisted, onWishlist, onAddToCart, onBuyNow, onOpenMessage }) => {
  return (
    <article className="group flex flex-col justify-between select-none">
      {/* Product Image Container */}
      <div className="relative aspect-[0.72/1] w-full bg-[#f5f5f4] rounded-2xl overflow-hidden shadow-2xs transition-all duration-300">
        <img
          src={product.image}
          alt={product.alt}
          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-700 ease-out"
        />

        {/* Wishlist Button top-left matching design screenshot */}
        <button
          type="button"
          onClick={onWishlist}
          title={isWishlisted ? 'Remove from wishlist' : 'Add to wishlist'}
          className="absolute top-3.5 left-3.5 w-8 h-8 rounded-full bg-white/90 backdrop-blur-xs flex items-center justify-center text-gray-800 hover:text-rose-600 shadow-sm transition-all cursor-pointer z-10"
        >
          <Heart
            size={16}
            fill={isWishlisted ? '#e11d48' : 'none'}
            className={isWishlisted ? 'text-rose-600' : 'text-gray-700'}
          />
        </button>

        {/* Couture Sale Badge if applicable */}
        {product.sale && (
          <span className="absolute top-3.5 right-3.5 bg-neutral-900/80 text-white text-[9px] uppercase font-bold px-2.5 py-1 rounded-full tracking-wider backdrop-blur-xs">
            Sale
          </span>
        )}

        {/* Hover Action Bar matching exact user screenshot */}
        <div className="absolute inset-x-0 bottom-0 flex items-stretch translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-in-out z-20 shadow-lg">
          {/* Left Dark Action Bar: ADD TO BAG + BUY NOW */}
          <div className="flex-1 bg-[#1c1917] text-white flex items-center justify-evenly px-3 py-3 font-mono text-[10px] uppercase font-bold tracking-wider">
            <button
              onClick={onAddToCart}
              className="hover:text-amber-300 transition-colors flex items-center space-x-1 cursor-pointer"
            >
              <span>ADD TO BAG</span>
            </button>

            <span className="text-neutral-600 font-light">+</span>

            <button
              onClick={onBuyNow}
              className="hover:text-amber-300 transition-colors flex items-center space-x-1 cursor-pointer"
            >
              <span>BUY NOW</span>
              <ArrowRight size={12} className="inline ml-0.5" />
            </button>
          </div>

          {/* Right Light Action Bar: MESSAGE */}
          <button
            onClick={onOpenMessage}
            className="bg-[#f5f5f4] hover:bg-white text-neutral-800 px-3.5 py-3 font-mono text-[10px] uppercase font-bold tracking-wider flex items-center space-x-1.5 transition-colors cursor-pointer border-l border-neutral-200"
          >
            <MessageSquare size={13} className="text-neutral-700" />
            <span>MESSAGE</span>
          </button>
        </div>
      </div>

      {/* Card Info Below Image */}
      <div className="pt-3 px-1 space-y-1">
        <div className="flex items-center justify-between text-[10px] uppercase font-bold tracking-[0.2em] text-neutral-400">
          <span>{product.category || 'THE OCCASION EDIT'}</span>
          <span className="font-semibold text-neutral-800 text-xs">
            {product.currency} {product.price.toLocaleString()}
          </span>
        </div>

        <h3 className="font-serif text-base font-medium text-neutral-900 group-hover:text-amber-700 transition-colors tracking-tight">
          {product.name}
        </h3>
      </div>
    </article>
  );
};

export default ProductGrid;
