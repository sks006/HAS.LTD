import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { ProductDto } from '@/shared/types/contracts';

export interface CreateProductPayload {
  name: string;
  price: number;
  description?: string;
  fabric_type?: string;
  season?: string;
  images?: string[];
  stock?: number;
  incoming?: number;
}

export interface UpdateProductPayload {
  name?: string;
  price?: number;
  description?: string;
  fabric_type?: string;
  season?: string;
  images?: string[];
  stock?: number;
  incoming?: number;
}

export async function fetchCatalogProducts(): Promise<ProductDto[]> {
  try {
    const rawData = await apiFetch<any[]>(ENDPOINTS.PRODUCTS);
    if (!Array.isArray(rawData)) return [];

    return rawData.map((item, idx) => ({
      id: item.id || `prod-${idx}`,
      name: item.name || 'Untitled Piece',
      category: item.category || item.fabric_type || 'no category',
      price: typeof item.price === 'number' ? item.price : (item.price_cents ? item.price_cents / 100 : 1999),
      currency: 'SAR',
      image: (item.images && item.images[0]) || item.image,
      alt: item.name || 'Product Image',
      sale: true,
      rating: item.rating ,
      reviews: item.reviews ,
      stock: typeof item.stock === 'number' ? item.stock : 0,
      incoming: typeof item.incoming === 'number' ? item.incoming : 0,
    }));
  } catch (err) {
    console.warn('Backend fetch notice:', err);
    return [];
  }
}

export async function fetchProductById(id: string): Promise<ProductDto> {
  return apiFetch<ProductDto>(ENDPOINTS.PRODUCT_BY_ID(id));
}

export async function createProductFetch(payload: CreateProductPayload): Promise<ProductDto> {
  return apiFetch<ProductDto>(ENDPOINTS.PRODUCTS, {
    method: 'POST',
    body: payload,
  });
}

export async function updateProductFetch(id: string, payload: UpdateProductPayload): Promise<ProductDto> {
  return apiFetch<ProductDto>(ENDPOINTS.PRODUCT_BY_ID(id), {
    method: 'PUT',
    body: payload,
  });
}

export async function deleteProductFetch(id: string): Promise<void> {
  return apiFetch<void>(ENDPOINTS.PRODUCT_BY_ID(id), {
    method: 'DELETE',
  });
}
