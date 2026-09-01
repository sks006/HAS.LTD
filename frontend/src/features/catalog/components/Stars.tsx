import { Star } from 'lucide-react';
import type { ProductDto } from '@/shared/types/contracts';

export function renderStars(rating: number) {
  const rounded = Math.round(rating);
  return [1, 2, 3, 4, 5].map((star) => (
    <Star key={star} size={13} strokeWidth={1.5} className={star <= rounded ? 'text-[#e6a817]' : 'text-[#e0e0e0]'} fill={star <= rounded ? 'currentColor' : 'none'} />
  ));
}

export const filters = ['All pieces', 'New arrivals', 'Best sellers', 'Everyday edit'];
