import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { AuthUser } from '@/shared/types/contracts';

export async function loginFetch(email: string, password: string): Promise<{ user: AuthUser; token: string }> {
  return apiFetch(ENDPOINTS.LOGIN, {
    method: 'POST',
    body: { email, password },
  });
}

export async function registerFetch(email: string, password: string): Promise<{ user: AuthUser; token: string }> {
  return apiFetch(ENDPOINTS.REGISTER, {
    method: 'POST',
    body: { email, password },
  });
}

export async function refreshTokenFetch(token: string): Promise<{ token: string }> {
  return apiFetch(ENDPOINTS.REFRESH, {
    method: 'POST',
    token,
  });
}

export async function logoutFetch(token: string): Promise<void> {
  return apiFetch(ENDPOINTS.LOGOUT, {
    method: 'POST',
    token,
  });
}
