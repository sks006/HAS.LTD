import React, { useState, useRef } from 'react';
import { useRootStore } from '@/slicers/root_store';
import { useCreateProduct } from '@/features/catalog/hooks/useProducts';
import { X, Plus, Package, UploadCloud, Image as ImageIcon, Loader2, Check } from 'lucide-react';

interface AddProductModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AddProductModal: React.FC<AddProductModalProps> = ({ isOpen, onClose }) => {
  const addInventoryItem = useRootStore((s) => s.addInventoryItem);
  const createProductMutation = useCreateProduct();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [name, setName] = useState('');
  const [category, setCategory] = useState('Apparel');
  const [sku, setSku] = useState(`AFZM${Math.floor(100 + Math.random() * 900)}`);
  const [incoming, setIncoming] = useState(100);
  const [stock, setStock] = useState(250);
  const [price, setPrice] = useState(1999);
  const [status, setStatus] = useState<'In stock' | 'Low stock' | 'Out of stock'>('In stock');
  const [image, setImage] = useState('https://images.unsplash.com/photo-1523275335684-37898b6baf30?auto=format&fit=crop&w=120&q=80');
  const [imagePreview, setImagePreview] = useState<string | null>('https://images.unsplash.com/photo-1523275335684-37898b6baf30?auto=format&fit=crop&w=120&q=80');
  const [isDragging, setIsDragging] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  if (!isOpen) return null;

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
      // 1. Send to Rust Axum backend (inserts to PostgreSQL database)
      await createProductMutation.mutateAsync({
        name,
        price: Number(price),
        description: `Stock:${stock}|Incoming:${incoming}`,
        fabric_type: category,
        images: [activeImage],
        stock: Number(stock),
        incoming: Number(incoming),
      });

      // 2. Also update local store state for immediate view sync
      addInventoryItem({
        name,
        category,
        sku,
        incoming: Number(incoming),
        stock: Number(stock),
        price: Number(price),
        currency: '$',
        status,
        image: activeImage,
      });

      onClose();
    } catch (err: any) {
      console.warn('Backend create product fallback notice:', err);
      addInventoryItem({
        name,
        category,
        sku,
        incoming: Number(incoming),
        stock: Number(stock),
        price: Number(price),
        currency: '$',
        status,
        image: activeImage,
      });
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div className="fixed inset-0 bg-black/50 backdrop-blur-xs" onClick={onClose} />

      {/* Modal Card */}
      <div className="relative bg-white rounded-2xl shadow-xl w-full max-w-lg overflow-hidden border border-gray-100 z-10 animate-in fade-in zoom-in-95 duration-200">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
          <div className="flex items-center space-x-2">
            <div className="w-8 h-8 rounded-lg bg-purple-100 flex items-center justify-center text-purple-600">
              <Package className="w-4 h-4" />
            </div>
            <h2 className="text-base font-bold text-gray-900">Add Product to Database</h2>
          </div>
          <button onClick={onClose} className="p-1 rounded-lg text-gray-400 hover:text-gray-600 hover:bg-gray-100">
            <X className="w-5 h-5" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4 text-xs">
          {errorMsg && (
            <div className="p-3 bg-red-50 text-red-700 rounded-xl font-medium text-xs">
              {errorMsg}
            </div>
          )}

          <div>
            <label className="block text-gray-700 font-semibold mb-1">Product Name</label>
            <input
              type="text"
              required
              placeholder="e.g. LuminaSilk Top"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500/20 focus:border-purple-500 text-xs"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-gray-700 font-semibold mb-1">Category</label>
              <select
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500/20 focus:border-purple-500 text-xs bg-white"
              >
                <option value="Electronics">Electronics</option>
                <option value="Apparel">Apparel</option>
                <option value="Wellness">Wellness</option>
                <option value="Home & Living">Home & Living</option>
                <option value="Velvet & Silk">Velvet & Silk</option>
              </select>
            </div>

            <div>
              <label className="block text-gray-700 font-semibold mb-1">SKU Code</label>
              <input
                type="text"
                required
                value={sku}
                onChange={(e) => setSku(e.target.value)}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500/20 focus:border-purple-500 text-xs font-mono"
              />
            </div>
          </div>

          <div className="grid grid-cols-3 gap-3">
            <div>
              <label className="block text-gray-700 font-semibold mb-1">Incoming</label>
              <input
                type="number"
                min="0"
                value={incoming}
                onChange={(e) => setIncoming(Number(e.target.value))}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500/20 focus:border-purple-500 text-xs"
              />
            </div>

            <div>
              <label className="block text-gray-700 font-semibold mb-1">Stock</label>
              <input
                type="number"
                min="0"
                value={stock}
                onChange={(e) => setStock(Number(e.target.value))}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500/20 focus:border-purple-500 text-xs"
              />
            </div>

            <div>
              <label className="block text-gray-700 font-semibold mb-1">Price ($)</label>
              <input
                type="number"
                min="0"
                value={price}
                onChange={(e) => setPrice(Number(e.target.value))}
                className="w-full px-3 py-2 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500/20 focus:border-purple-500 text-xs"
              />
            </div>
          </div>

          {/* Image Upload Area */}
          <div>
            <label className="block text-gray-700 font-semibold mb-1">Product Image Upload</label>
            
            <input
              type="file"
              ref={fileInputRef}
              accept="image/*"
              className="hidden"
              onChange={(e) => e.target.files?.[0] && handleFileChange(e.target.files[0])}
            />

            <div
              onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
              onDragLeave={() => setIsDragging(false)}
              onDrop={handleDrop}
              onClick={() => fileInputRef.current?.click()}
              className={`border-2 border-dashed rounded-xl p-4 text-center cursor-pointer transition-all flex items-center justify-center gap-4 ${
                isDragging ? 'border-purple-500 bg-purple-50' : 'border-gray-200 hover:border-purple-300 hover:bg-gray-50'
              }`}
            >
              {imagePreview ? (
                <div className="flex items-center gap-3 w-full">
                  <img
                    src={imagePreview}
                    alt="Preview"
                    className="w-12 h-12 rounded-lg object-cover border border-gray-200 shadow-xs"
                  />
                  <div className="text-left flex-1 truncate">
                    <p className="font-semibold text-gray-800 text-xs flex items-center gap-1">
                      <Check className="w-3.5 h-3.5 text-emerald-600" /> Image Selected
                    </p>
                    <p className="text-[10px] text-gray-500">Click or drag to replace image</p>
                  </div>
                </div>
              ) : (
                <div className="space-y-1">
                  <UploadCloud className="w-6 h-6 text-purple-500 mx-auto" />
                  <p className="text-xs text-gray-600 font-medium">
                    Click to upload image or drag & drop file
                  </p>
                  <p className="text-[10px] text-gray-400">PNG, JPG, WEBP supported</p>
                </div>
              )}
            </div>

            <div className="mt-2">
              <input
                type="text"
                placeholder="Or paste image URL directly..."
                value={image}
                onChange={(e) => {
                  setImage(e.target.value);
                  setImagePreview(e.target.value);
                }}
                className="w-full px-3 py-1.5 border border-gray-200 rounded-lg text-[11px] text-gray-600 placeholder:text-gray-400 focus:outline-none focus:border-purple-400"
              />
            </div>
          </div>

          <div className="pt-4 flex items-center justify-end space-x-3 border-t border-gray-100">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-xl font-medium"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={createProductMutation.isPending}
              className="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-xl font-semibold shadow-sm flex items-center space-x-1.5 disabled:opacity-50"
            >
              {createProductMutation.isPending ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Plus className="w-4 h-4" />
              )}
              <span>Add Product</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default AddProductModal;
