import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  fetchCatalogProducts,
  createProductFetch,
  updateProductFetch,
  deleteProductFetch,
  CreateProductPayload,
  UpdateProductPayload,
} from '@/slicers/catalog_slicer/catalog_fetch';
import { ProductDto, InventoryItemDto } from '@/shared/types/contracts';
import { useRootStore } from '@/slicers/root_store';
import { useEffect } from 'react';

export function useProducts() {
  const setProducts = useRootStore((s) => s.setProducts);
  const setInventory = useRootStore((s) => s.setInventory);

  const query = useQuery<ProductDto[]>({
    queryKey: ['products'],
    queryFn: async () => {
      return fetchCatalogProducts();
    },
  });

  useEffect(() => {
    if (query.data) {
      setProducts(query.data);

      const mappedInventory: InventoryItemDto[] = query.data.map((p: any, idx: number) => {
        const stockVal = typeof p.stock === 'number' ? p.stock : 120;
        const incomingVal = typeof p.incoming === 'number' ? p.incoming : 50;
        const status = stockVal === 0 ? 'Out of stock' : stockVal < 20 ? 'Low stock' : 'In stock';

        return {
          id: p.id,
          name: p.name,
          category: p.category || 'Apparel',
          sku: p.id || `SKU-${100 + idx}`,
          incoming: incomingVal,
          stock: stockVal,
          price: p.price,
          currency: p.currency || '$',
          status,
          image: p.image,
        };
      });

      setInventory(mappedInventory);
    }
  }, [query.data, setProducts, setInventory]);

  return query;
}

export function useCreateProduct() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (payload: CreateProductPayload) => {
      return createProductFetch(payload);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] });
    },
  });
}

export function useUpdateProduct() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, payload }: { id: string; payload: UpdateProductPayload }) => {
      return updateProductFetch(id, payload);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] });
    },
  });
}

export function useDeleteProduct() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      return deleteProductFetch(id);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] });
    },
  });
}
