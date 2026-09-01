import type { Role } from './roles';

export type Uuid = string;

export type ProductDto = {
  id: Uuid;
  name: string;
  category: string;
  price: number;
  currency: string;
  image: string;
  alt: string;
  sale: boolean;
  rating: number;
  reviews: number;
};

export type InventoryItemDto = {
  id: Uuid;
  name: string;
  category: string;
  sku: string;
  incoming: number;
  stock: number;
  price: number;
  currency: string;
  status: 'In stock' | 'Low stock' | 'Out of stock';
};

export type OrderItemDto = {
  productId: Uuid;
  name: string;
  quantity: number;
  price: number;
};

export type OrderDto = {
  id: Uuid;
  items: OrderItemDto[];
  total: number;
  currency: string;
  status: 'pending' | 'confirmed' | 'shipped' | 'delivered' | 'refunded';
  createdAt: string;
  customerId?: Uuid;
};

export type AuthUser = {
  id: Uuid;
  email: string;
  role: Role;
};
