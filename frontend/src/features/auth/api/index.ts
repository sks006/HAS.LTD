import {
  loginFetch,
  logoutFetch,
  refreshTokenFetch,
  registerFetch,
  type LoginResponse,
  type RegisterResponse,
} from '@/slicers/auth_slicer/auth_fetch';

export function login(email: string, password: string): Promise<LoginResponse> {
  return loginFetch(email, password);
}

export function register(email: string, password: string, name?: string): Promise<RegisterResponse> {
  return registerFetch(email, password, name);
}

export function refreshToken(): Promise<{ access_token: string }> {
  return refreshTokenFetch();
}

export function logout(token: string): Promise<{ success: boolean }> {
  return logoutFetch(token);
}
