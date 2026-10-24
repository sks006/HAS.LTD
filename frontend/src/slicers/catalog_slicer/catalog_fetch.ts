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

interface BackendProduct {
  id: string;
  name: string;
  price_cents: number;
  category: string;
  images: string[];
  stock: number;
  incoming: number;
}

function mapProduct(item: BackendProduct): ProductDto {
  return {
    id: item.id,
    name: item.name,
    category: item.category,
    price: item.price_cents / 100,
    currency: 'SAR',
    image: item.images[0] || '',
    alt: item.name,
    sale: false,
    rating: 0,
    reviews: 0,
    stock: item.stock,
    incoming: item.incoming,
  };
}

export async function fetchCatalogProducts(): Promise<ProductDto[]> {
  const products = await apiFetch<BackendProduct[]>(ENDPOINTS.PRODUCTS);
  return products.map(mapProduct);
}

export async function fetchProductById(id: string): Promise<ProductDto> {
  return mapProduct(await apiFetch<BackendProduct>(ENDPOINTS.PRODUCT_BY_ID(id)));
}

export async function createProductFetch(payload: CreateProductPayload): Promise<ProductDto> {
  return mapProduct(await apiFetch<BackendProduct>(ENDPOINTS.PRODUCTS, {
    method: 'POST',
    body: payload,
  }));
}

export async function updateProductFetch(id: string, payload: UpdateProductPayload): Promise<ProductDto> {
  return mapProduct(await apiFetch<BackendProduct>(ENDPOINTS.PRODUCT_BY_ID(id), {
    method: 'PUT',
    body: payload,
  }));
}

export async function deleteProductFetch(id: string): Promise<void> {
  return apiFetch<void>(ENDPOINTS.PRODUCT_BY_ID(id), {
    method: 'DELETE',
  });
}
