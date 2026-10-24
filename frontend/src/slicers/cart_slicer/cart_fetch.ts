import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { AddressDto } from '@/shared/types/contracts';

export interface CheckoutPayload {
  currency: string;
  shipping_address: AddressDto;
  billing_address?: AddressDto;
  items: {
    variant_id?: string;
    product_id?: string;
    price_minor_units?: number;
    product_name?: string;
    sku?: string;
    quantity: number;
  }[];
}

export interface CheckoutResponse {
  ok: boolean;
  order_id: string;
  message: string;
}

export async function syncCartFetch(items: { variant_id: string; quantity_change: number }[]) {
  return apiFetch<void>(ENDPOINTS.CART_SYNC, {
    method: 'POST',
    body: items,
  });
}

export async function submitOrderFetch(
  payload: CheckoutPayload,
  idempotencyKey: string
): Promise<CheckoutResponse> {
  return apiFetch<CheckoutResponse>(ENDPOINTS.CHECKOUT, {
    method: 'POST',
    headers: { 'x-idempotency-key': idempotencyKey },
    body: payload,
  });
}
