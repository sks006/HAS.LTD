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
  image?: string;
};

export type OrderState = 'Pending' | 'Processing' | 'Delivered' | 'Returned' | 'Cancelled';

export type AddressDto = {
  recipient_name: string;
  phone: string;
  street_line1: string;
  street_line2?: string;
  city: string;
  state?: string;
  postal_code: string;
  country: string;
  delivery_instructions?: string;
};

export type OrderItemDto = {
  productId: Uuid;
  name: string;
  quantity: number;
  price: number;
};

export type OrderRecord = {
  id: Uuid;
  order_number: string; // e.g. "ID: 70668"
  customer_name: string;
  product_name: string;
  sku: string;
  payment_method: 'Cash on Delivery' | 'Paid via PayPal' | 'Credit Card' | 'Apple Pay';
  amount: number;
  currency: string;
  status: OrderState;
  created_at: string;
  updated_at?: string;
  version?: number;
  idempotency_key?: string;
  shipping_address?: AddressDto;
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
