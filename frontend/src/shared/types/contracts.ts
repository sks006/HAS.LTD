import { UserRole } from './roles';

export interface ProductDto {
  id: string;
  name: string;
  price_cents: number;
}

export interface AddressDto {
  recipient_name: string;
  phone: string;
  street_line1: string;
  street_line2?: string;
  city: string;
  state?: string;
  postal_code: string;
  country: string;
  delivery_instructions?: string;
}

export interface CheckoutItemDto {
  variant_id: string;
  quantity: number;
}

export interface OrderDto {
  id: string;
  user_id: string | null;
  state: 'Pending' | 'Reserved' | 'Paid' | 'Shipped' | 'Cancelled';
  currency: string;
  total_minor_units: number;
  version: number;
}
