import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { ProductDto } from '@/shared/types/contracts';

export async function fetchCatalogProducts(): Promise<ProductDto[]> {
  return apiFetch<ProductDto[]>(ENDPOINTS.PRODUCTS);
}

export async function fetchProductById(id: string): Promise<ProductDto> {
  return apiFetch<ProductDto>(ENDPOINTS.PRODUCT_BY_ID(id));
}
