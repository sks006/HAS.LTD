import { fetchCatalogProducts, fetchProductById } from '@/slicers/catalog_slicer/catalog_fetch';
import { ProductDto } from '@/shared/types/contracts';

export const catalogApi = {
  fetchProducts: async (): Promise<ProductDto[]> => {
    return fetchCatalogProducts();
  },

  fetchProductById: async (id: string): Promise<ProductDto> => {
    return fetchProductById(id);
  },
};
