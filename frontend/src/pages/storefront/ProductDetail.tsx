import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useRootStore } from '@/slicers/root_store';
import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import { ProductDto } from '@/shared/types/contracts';
import { Button } from '@/shared/ui/button';
import { Badge } from '@/shared/ui/badge';
import { Separator } from '@/shared/ui/separator';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/shared/ui/tabs';
import { Card, CardContent } from '@/shared/ui/card';
import {
  ArrowLeft,
  Heart,
  ShoppingBag,
  Sparkles,
  ShieldCheck,
  Truck,
  RotateCcw,
  Check,
  Ruler,
  Star,
  Loader2,
} from 'lucide-react';

interface BackendProductDetail {
  id: string;
  name: string;
  price_cents: number;
  price_formatted: string;
  category: string;
  images: string[];
  stock: number;
  incoming: number;
}

export const ProductDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const products = useRootStore((s) => s.products);
  const addToCart = useRootStore((s) => s.addToCart);

  const [product, setProduct] = useState<ProductDto | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [selectedSize, setSelectedSize] = useState<string>('54 (M)');
  const [isWishlisted, setIsWishlisted] = useState<boolean>(false);
  const [addedNotice, setAddedNotice] = useState<boolean>(false);
  const [activeImageIndex, setActiveImageIndex] = useState<number>(0);

  useEffect(() => {
    async function loadProduct() {
      if (!id) return;
      setLoading(true);

      // Check client-side store first
      const storeItem = products.find((p) => p.id === id);
      if (storeItem) {
        setProduct(storeItem);
        setLoading(false);
      }

      // Query live backend endpoint GET /products/{id}
      try {
        const data = await apiFetch<BackendProductDetail>(ENDPOINTS.PRODUCT_BY_ID(id));
        if (data) {
          const mapped: ProductDto = {
            id: data.id,
            name: data.name,
            price: data.price_cents / 100,
            currency: 'SAR',
            category: data.category || 'Haute Couture',
            image:
              data.images?.[0] ||
              storeItem?.image ||
              'https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?auto=format&fit=crop&q=80&w=1000',
            alt: data.name,
            rating: 5.0,
            reviews: 14,
            sale: false,
          };
          setProduct(mapped);
        }
      } catch (err) {
        console.debug('Falling back to local store record for product view:', err);
      } finally {
        setLoading(false);
      }
    }

    loadProduct();
  }, [id, products]);

  const handleAddToCart = () => {
    if (!product) return;
    addToCart({
      productId: product.id,
      name: `${product.name} [Size ${selectedSize}]`,
      quantity: 1,
      price: product.price,
    });
    setAddedNotice(true);
    setTimeout(() => setAddedNotice(false), 2500);
  };

  const handleBuyNow = () => {
    handleAddToCart();
    navigate('/checkout');
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-white flex flex-col items-center justify-center space-y-3">
        <Loader2 className="w-8 h-8 text-neutral-800 animate-spin" />
        <p className="text-xs font-mono uppercase tracking-widest text-neutral-500">
          Loading Atelier Piece...
        </p>
      </div>
    );
  }

  if (!product) {
    return (
      <div className="min-h-screen bg-white flex flex-col items-center justify-center p-6 text-center space-y-4">
        <h2 className="font-serif text-2xl text-neutral-900">Garment Not Found</h2>
        <p className="text-xs text-neutral-500">
          The requested abaya piece may have been de-listed or re-cataloged.
        </p>
        <Button variant="luxury" onClick={() => navigate('/')}>
          Return to Storefront
        </Button>
      </div>
    );
  }

  const galleryImages = [
    product.image,
    'https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?auto=format&fit=crop&q=80&w=1000',
    'https://images.unsplash.com/photo-1515886657613-9f3515b0c78f?auto=format&fit=crop&q=80&w=1000',
  ];

  const sizes = ['50 (XS)', '52 (S)', '54 (M)', '56 (L)', '58 (XL)', 'Bespoke Fit'];

  return (
    <div className="min-h-screen bg-white text-neutral-900 font-sans select-none">
      {/* Top Bar Navigation */}
      <nav className="border-b border-neutral-100 py-3.5 px-6 sticky top-0 bg-white/95 backdrop-blur-xs z-30">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <button
            type="button"
            onClick={() => navigate('/')}
            className="text-xs font-semibold uppercase tracking-wider text-neutral-600 hover:text-neutral-900 transition-colors flex items-center space-x-2"
          >
            <ArrowLeft size={14} />
            <span>Return to Catalog</span>
          </button>

          <span className="font-serif text-lg font-bold tracking-tight text-neutral-900">
            Ajrah Noor
          </span>

          <Button
            variant="ghost"
            size="sm"
            onClick={() => navigate('/checkout')}
            className="flex items-center space-x-1.5 text-xs font-bold"
          >
            <ShoppingBag size={15} />
            <span className="hidden sm:inline">Bag</span>
          </Button>
        </div>
      </nav>

      {/* Main Detail Container */}
      <main className="max-w-7xl mx-auto px-6 py-10">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-start">
          {/* Left Column: Visual Gallery */}
          <div className="lg:col-span-7 space-y-4">
            <div className="relative aspect-[0.78/1] bg-neutral-100 rounded-3xl overflow-hidden shadow-sm border border-neutral-200/60">
              <img
                src={galleryImages[activeImageIndex] || product.image}
                alt={product.alt || product.name}
                className="w-full h-full object-cover"
              />

              <button
                type="button"
                onClick={() => setIsWishlisted(!isWishlisted)}
                className="absolute top-4 right-4 w-10 h-10 rounded-full bg-white/90 backdrop-blur-xs flex items-center justify-center text-neutral-800 hover:text-rose-600 shadow-sm transition-all"
                title="Save to Wishlist"
              >
                <Heart
                  size={18}
                  fill={isWishlisted ? '#e11d48' : 'none'}
                  className={isWishlisted ? 'text-rose-600' : 'text-neutral-700'}
                />
              </button>

              <div className="absolute bottom-4 left-4">
                <Badge variant="secondary" className="bg-white/90 backdrop-blur-xs text-[10px] font-mono tracking-widest uppercase">
                  Atelier Pure Silk
                </Badge>
              </div>
            </div>

            {/* Thumbnail Carousel Selector */}
            <div className="flex items-center space-x-3 overflow-x-auto pb-2">
              {galleryImages.map((img, idx) => (
                <button
                  key={idx}
                  type="button"
                  onClick={() => setActiveImageIndex(idx)}
                  className={`w-20 h-24 rounded-xl overflow-hidden border-2 transition-all shrink-0 cursor-pointer ${
                    activeImageIndex === idx
                      ? 'border-neutral-900 shadow-sm ring-1 ring-neutral-900'
                      : 'border-neutral-200 opacity-60 hover:opacity-100'
                  }`}
                >
                  <img src={img} alt="Thumbnail" className="w-full h-full object-cover" />
                </button>
              ))}
            </div>
          </div>

          {/* Right Column: Garment Specifications & Actions */}
          <div className="lg:col-span-5 space-y-6">
            <div className="space-y-2">
              <div className="flex items-center space-x-2">
                <Badge variant="outline" className="text-[10px] uppercase tracking-widest font-mono border-neutral-300">
                  {product.category || 'THE OCCASION EDIT'}
                </Badge>
                <span className="flex items-center space-x-1 text-xs text-amber-700 font-medium">
                  <Star size={13} fill="currentColor" />
                  <span>5.0 (14 reviews)</span>
                </span>
              </div>

              <h1 className="font-serif text-3xl sm:text-4xl font-normal text-neutral-900 tracking-tight leading-tight">
                {product.name}
              </h1>

              <div className="pt-2 flex items-baseline space-x-3">
                <span className="font-serif text-2xl font-bold text-neutral-900">
                  SAR {product.price.toLocaleString()}
                </span>
                <span className="text-xs text-neutral-400 font-medium">
                  Inclusive of Atelier Custom Tailoring
                </span>
              </div>
            </div>

            <Separator />

            {/* Size & Length Selector */}
            <div className="space-y-3">
              <div className="flex items-center justify-between text-xs">
                <span className="font-bold uppercase tracking-wider text-neutral-700">
                  Select Abaya Length & Size
                </span>
                <span className="text-neutral-500 hover:text-neutral-900 cursor-pointer inline-flex items-center space-x-1 text-[11px]">
                  <Ruler size={13} />
                  <span>Size Guide</span>
                </span>
              </div>

              <div className="grid grid-cols-3 gap-2">
                {sizes.map((sz) => (
                  <button
                    key={sz}
                    type="button"
                    onClick={() => setSelectedSize(sz)}
                    className={`py-2.5 px-3 rounded-xl border text-xs font-semibold text-center transition-all cursor-pointer ${
                      selectedSize === sz
                        ? 'border-neutral-900 bg-neutral-900 text-white shadow-xs'
                        : 'border-neutral-200 bg-white text-neutral-700 hover:border-neutral-300'
                    }`}
                  >
                    {sz}
                  </button>
                ))}
              </div>
            </div>

            {/* Action Buttons */}
            <div className="space-y-3 pt-2">
              <Button
                variant="luxury"
                size="lg"
                className="w-full font-mono text-xs uppercase tracking-widest h-12 shadow-md justify-between"
                onClick={handleAddToCart}
              >
                <span className="flex items-center space-x-2">
                  <ShoppingBag size={16} />
                  <span>{addedNotice ? 'Added to Bag!' : 'Add Piece to Bag'}</span>
                </span>
                <span className="font-serif font-bold text-amber-200 normal-case">
                  SAR {product.price.toLocaleString()}
                </span>
              </Button>

              <Button
                variant="outline"
                size="lg"
                className="w-full font-mono text-xs uppercase tracking-widest h-12 border-neutral-300 hover:bg-neutral-50"
                onClick={handleBuyNow}
              >
                Instant Checkout
              </Button>
            </div>

            {/* Value Guarantees */}
            <Card className="border-neutral-200/80 bg-neutral-50/60 shadow-2xs">
              <CardContent className="p-4 space-y-3 text-xs text-neutral-600">
                <div className="flex items-center space-x-3">
                  <Truck size={16} className="text-amber-800 shrink-0" />
                  <span>Complimentary Express GCC Courier Delivery (2-4 business days)</span>
                </div>
                <div className="flex items-center space-x-3">
                  <ShieldCheck size={16} className="text-amber-800 shrink-0" />
                  <span>Handcrafted from 100% Japanese Silk Crepe & Italian Velvets</span>
                </div>
                <div className="flex items-center space-x-3">
                  <RotateCcw size={16} className="text-amber-800 shrink-0" />
                  <span>Hassle-free 14-day atelier exchange policy</span>
                </div>
              </CardContent>
            </Card>

            {/* Tabbed In-Depth Information */}
            <Tabs defaultValue="details" className="w-full pt-4">
              <TabsList className="grid grid-cols-3 w-full bg-neutral-100 p-1 rounded-xl">
                <TabsTrigger value="details" className="text-xs font-semibold rounded-lg">
                  Details
                </TabsTrigger>
                <TabsTrigger value="fabric" className="text-xs font-semibold rounded-lg">
                  Fabric & Care
                </TabsTrigger>
                <TabsTrigger value="shipping" className="text-xs font-semibold rounded-lg">
                  Shipping
                </TabsTrigger>
              </TabsList>

              <TabsContent value="details" className="pt-4 text-xs text-neutral-600 space-y-2 leading-relaxed">
                <p>
                  Cut for a graceful drape, this abaya celebrates minimalist modesty with thoughtful architectural precision. Features discreet concealed snap fastenings and bespoke hand-rolled hem finishing.
                </p>
                <ul className="list-disc list-inside space-y-1 text-neutral-700 font-medium">
                  <li>Relaxed, flowing silhouette with tailored sleeve accents</li>
                  <li>Deep inseam pockets lined with pure silk organza</li>
                  <li>Includes matching tonal lightweight chiffon Sheila scarf</li>
                </ul>
              </TabsContent>

              <TabsContent value="fabric" className="pt-4 text-xs text-neutral-600 space-y-2 leading-relaxed">
                <p>
                  Sourced exclusively from heritage mills in Kyoto and Como. Dry clean only. Iron on reverse using a low temperature silk setting or gentle handheld vertical steam.
                </p>
              </TabsContent>

              <TabsContent value="shipping" className="pt-4 text-xs text-neutral-600 space-y-2 leading-relaxed">
                <p>
                  Each piece is individually inspected, pressed, and wrapped in archival tissue paper before dispatch in our signature matte black Ajrah Noor gift box.
                </p>
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </main>
    </div>
  );
};

export default ProductDetail;
