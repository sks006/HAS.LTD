import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { InventoryItemDto } from '@/shared/types/contracts';

interface BackendProduct {
  id: string;
  name: string;
  price_cents: number;
  category: string;
  images: string[];
  stock: number;
  incoming: number;
}

function toInventoryItem(product: BackendProduct): InventoryItemDto {
  return {
    id: product.id,
    name: product.name,
    category: product.category,
    sku: product.id,
    incoming: product.incoming,
    stock: product.stock,
    price: product.price_cents / 100,
    currency: 'SAR',
    status: product.stock === 0 ? 'Out of stock' : product.stock < 20 ? 'Low stock' : 'In stock',
    image: product.images[0],
  };
}

export async function fetchInventory(): Promise<InventoryItemDto[]> {
  const products = await apiFetch<BackendProduct[]>(ENDPOINTS.PRODUCTS);
  return products.map(toInventoryItem);
}

export async function updateProductInventory(
  id: string,
  patch: Partial<InventoryItemDto>
): Promise<InventoryItemDto> {
  const product = await apiFetch<BackendProduct>(ENDPOINTS.PRODUCT_BY_ID(id), {
    method: 'PUT',
    body: {
      name: patch.name,
      price: patch.price,
      fabric_type: patch.category,
      stock: patch.stock,
      incoming: patch.incoming,
      images: patch.image ? [patch.image] : undefined,
    },
  });
  return toInventoryItem(product);
}

export async function deleteInventoryProduct(id: string): Promise<void> {
  return apiFetch<void>(ENDPOINTS.PRODUCT_BY_ID(id), {
    method: 'DELETE',
  });
}
