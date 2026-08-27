import { useQuery } from '@tanstack/react-query';
import { apiClient } from '../../../shared/api/client';
import { API_ENDPOINTS } from '../../../shared/api/endpoints';
import { ProductDto } from '../../../shared/types/contracts';

export function useProducts() {
  return useQuery<ProductDto[]>({
    queryKey: ['products'],
    queryFn: async () => {
      const response = await apiClient.get(API_ENDPOINTS.products);
      return response.data;
    },
  });
}
