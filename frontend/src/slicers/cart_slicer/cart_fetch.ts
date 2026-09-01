import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { OrderDto } from '@/shared/types/contracts';

export async function syncCartFetch(items: { productId: string; quantity: number }[]): Promise<void> {
  return apiFetch<void>(ENDPOINTS.CART_SYNC, {
    method: 'POST',
    body: { items },
  });
}

export async function submitOrderFetch(
  items: { productId: string; quantity: number }[],
  idempotencyKey: string
): Promise<OrderDto> {
  return apiFetch<OrderDto>(ENDPOINTS.ORDERS, {
    method: 'POST',
    body: { items, idempotency_key: idempotencyKey },
  });
}
