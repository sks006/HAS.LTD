export const API_ENDPOINTS = {
  login: '/auth/login',
  register: '/auth/register',
  logout: '/auth/logout',
  products: '/products',
  checkout: '/checkout',
  orders: (id: string) => `/orders/${id}`,
  orderStatus: (id: string) => `/orders/${id}/status`,
  inventory: '/admin/inventory',
  metrics: '/admin/metrics',
  syncCart: '/cart/sync',
};
