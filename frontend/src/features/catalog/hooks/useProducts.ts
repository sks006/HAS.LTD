import { useQuery } from '@tanstack/react-query';
import { fetchCatalogProducts } from '@/slicers/catalog_slicer/catalog_fetch';
import { ProductDto } from '@/shared/types/contracts';

export function useProducts() {
  return useQuery<ProductDto[]>({
    queryKey: ['products'],
    queryFn: async () => {
      return fetchCatalogProducts();
    },
  });
}
