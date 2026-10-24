import React, { useState, useRef } from 'react';
import { useCreateProduct } from '@/features/catalog/hooks/useProducts';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/shared/ui/dialog';
import { Input } from '@/shared/ui/input';
import { Label } from '@/shared/ui/label';
import { Button } from '@/shared/ui/button';
import { Alert, AlertDescription } from '@/shared/ui/alert';
import {
  Package,
  UploadCloud,
  Image as ImageIcon,
  Loader2,
  Check,
  Sparkles,
} from 'lucide-react';

interface AddProductModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AddProductModal: React.FC<AddProductModalProps> = ({ isOpen, onClose }) => {
  const createProductMutation = useCreateProduct();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [name, setName] = useState('');
  const [category, setCategory] = useState('Velvet & Silk');
  const [incoming, setIncoming] = useState(50);
  const [stock, setStock] = useState(100);
  const [price, setPrice] = useState(1850);
  const [image, setImage] = useState(
    'https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?auto=format&fit=crop&w=400&q=80'
  );
  const [imagePreview, setImagePreview] = useState<string | null>(
    'https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?auto=format&fit=crop&w=400&q=80'
  );
  const [isDragging, setIsDragging] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleFileChange = (file: File) => {
    if (!file.type.startsWith('image/')) {
      setErrorMsg('Please select a valid image file (PNG, JPG, WEBP).');
      return;
    }
    setErrorMsg(null);
    const reader = new FileReader();
    reader.onload = (e) => {
      const dataUrl = e.target?.result as string;
      setImage(dataUrl);
      setImagePreview(dataUrl);
    };
    reader.readAsDataURL(file);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      handleFileChange(e.dataTransfer.files[0]);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setErrorMsg(null);

    const activeImage = imagePreview || image;

    try {
      await createProductMutation.mutateAsync({
        name,
        price: Number(price),
        description: `Stock:${stock}|Incoming:${incoming}`,
        fabric_type: category,
        images: [activeImage],
        stock: Number(stock),
        incoming: Number(incoming),
      });

      onClose();
      // Reset form
      setName('');
      setPrice(1850);
    } catch (requestError) {
      setErrorMsg(
        requestError instanceof Error
          ? requestError.message
          : 'Unable to commit garment creation to the PostgreSQL database.'
      );
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-lg bg-white border border-neutral-200">
        <DialogHeader>
          <div className="flex items-center space-x-2">
            <div className="w-9 h-9 rounded-xl bg-neutral-900 text-amber-300 flex items-center justify-center">
              <Package className="w-5 h-5" />
            </div>
            <div>
              <DialogTitle className="font-serif text-lg font-bold text-neutral-900">
                Catalog New Atelier Garment
              </DialogTitle>
              <DialogDescription className="text-xs text-neutral-500">
                Publishes a new abaya aggregate directly to the live Axum/PostgreSQL database.
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {errorMsg && (
            <Alert variant="destructive">
              <AlertDescription className="text-xs">{errorMsg}</AlertDescription>
            </Alert>
          )}

          <div className="space-y-1.5">
            <Label htmlFor="prodName" className="text-xs font-semibold text-neutral-700">
              Garment / Abaya Name
            </Label>
            <Input
              id="prodName"
              type="text"
              required
              placeholder="e.g. Royal Raw Silk Sheila Abaya"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="text-xs"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="prodCategory" className="text-xs font-semibold text-neutral-700">
                Collection & Fabric
              </Label>
              <select
                id="prodCategory"
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                className="w-full h-10 px-3 py-2 border border-neutral-200 rounded-md text-xs bg-white text-neutral-900 focus:outline-none focus:ring-2 focus:ring-neutral-900"
              >
                <option value="Velvet & Silk">Velvet & Silk</option>
                <option value="Gold Embroidery">Gold Embroidery</option>
                <option value="Casual Luxe">Casual Luxe</option>
                <option value="Bridal Edition">Bridal Edition</option>
                <option value="All Abayas">All Abayas</option>
              </select>
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="prodPrice" className="text-xs font-semibold text-neutral-700">
                Retail Price (SAR)
              </Label>
              <Input
                id="prodPrice"
                type="number"
                min="1"
                step="0.01"
                required
                value={price}
                onChange={(e) => setPrice(Number(e.target.value))}
                className="text-xs font-mono"
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="prodStock" className="text-xs font-semibold text-neutral-700">
                Active On-Hand Stock
              </Label>
              <Input
                id="prodStock"
                type="number"
                min="0"
                value={stock}
                onChange={(e) => setStock(Number(e.target.value))}
                className="text-xs font-mono"
              />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="prodIncoming" className="text-xs font-semibold text-neutral-700">
                Incoming Supply
              </Label>
              <Input
                id="prodIncoming"
                type="number"
                min="0"
                value={incoming}
                onChange={(e) => setIncoming(Number(e.target.value))}
                className="text-xs font-mono"
              />
            </div>
          </div>

          {/* Media / Image URL or Upload */}
          <div className="space-y-1.5">
            <Label className="text-xs font-semibold text-neutral-700">
              Media Image URL
            </Label>
            <Input
              type="url"
              value={image}
              onChange={(e) => {
                setImage(e.target.value);
                setImagePreview(e.target.value);
              }}
              placeholder="https://images.unsplash.com/..."
              className="text-xs"
            />
          </div>

          <div
            onDragOver={(e) => {
              e.preventDefault();
              setIsDragging(true);
            }}
            onDragLeave={() => setIsDragging(false)}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
            className={`border-2 border-dashed rounded-xl p-4 text-center cursor-pointer transition-colors ${
              isDragging
                ? 'border-neutral-900 bg-neutral-50'
                : 'border-neutral-200 hover:border-neutral-300 bg-neutral-50/50'
            }`}
          >
            <input
              type="file"
              ref={fileInputRef}
              className="hidden"
              accept="image/*"
              onChange={(e) => e.target.files?.[0] && handleFileChange(e.target.files[0])}
            />
            {imagePreview ? (
              <div className="flex items-center justify-center space-x-3">
                <img
                  src={imagePreview}
                  alt="Preview"
                  className="w-12 h-16 object-cover rounded-lg border border-neutral-200"
                />
                <span className="text-xs text-neutral-500">Image attached. Click or drag to change.</span>
              </div>
            ) : (
              <div className="space-y-1">
                <UploadCloud className="w-6 h-6 text-neutral-400 mx-auto" />
                <p className="text-xs text-neutral-500">Click or drag luxury garment photography here</p>
              </div>
            )}
          </div>

          <DialogFooter className="pt-2">
            <Button type="button" variant="outline" size="sm" onClick={onClose}>
              Cancel
            </Button>
            <Button
              type="submit"
              variant="luxury"
              size="sm"
              disabled={createProductMutation.isPending}
              isLoading={createProductMutation.isPending}
            >
              <span>Publish to Catalog</span>
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
};

export default AddProductModal;
