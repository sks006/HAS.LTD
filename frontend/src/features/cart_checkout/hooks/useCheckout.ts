import { useMutation } from '@tanstack/react-query';
import { apiClient } from '../../../shared/api/client';
import { API_ENDPOINTS } from '../../../shared/api/endpoints';

export function useCheckout() {
  return useMutation({
    mutationFn: async (checkoutPayload: any) => {
      const response = await apiClient.post(API_ENDPOINTS.checkout, checkoutPayload, {
        headers: {
          'idempotency-key': `checkout-${Date.now()}`,
        },
      });
      return response.data;
    },
  });
}
