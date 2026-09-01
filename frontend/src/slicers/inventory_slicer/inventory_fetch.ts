import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { InventoryItemDto } from '@/shared/types/contracts';

export async function fetchInventory(): Promise<InventoryItemDto[]> {
  return apiFetch<InventoryItemDto[]>(ENDPOINTS.INVENTORY);
}

export async function updateProductInventory(
  id: string,
  patch: Partial<InventoryItemDto>
): Promise<InventoryItemDto> {
  return apiFetch<InventoryItemDto>(`${ENDPOINTS.ADMIN_INVENTORY}/${id}`, {
    method: 'PUT',
    body: patch,
  });
}

export async function deleteInventoryProduct(id: string): Promise<void> {
  return apiFetch<void>(`${ENDPOINTS.ADMIN_INVENTORY}/${id}`, {
    method: 'DELETE',
  });
}
