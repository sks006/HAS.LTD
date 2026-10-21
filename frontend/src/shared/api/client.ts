import { ENDPOINTS } from './endpoints';

const API_BASE =
  (typeof import.meta !== 'undefined' &&
    ((import.meta as any).env?.VITE_API_URL || (import.meta as any).env?.NEXT_PUBLIC_API_URL)) ||
  (typeof process !== 'undefined' && process.env?.API_URL) ||
  'http://localhost:3000';

export class ApiError extends Error {
  constructor(
    public status: number,
    public statusText: string,
    public data: unknown,
    public url?: string,
    public method?: string
  ) {
    super(`${status} ${statusText}`);
    this.name = 'ApiError';
  }
}

export type RequestOptions = {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  body?: unknown;
  token?: string | null;
  params?: Record<string, string | number | boolean>;
  headers?: Record<string, string>;
  timeout?: number;
};

function buildUrl(path: string, params?: Record<string, string | number | boolean>): string {
  const cleanBase = API_BASE.endsWith('/') ? API_BASE : `${API_BASE}/`;
  const cleanPath = path.startsWith('/') ? path.slice(1) : path;
  const url = path.startsWith('http://') || path.startsWith('https://')
    ? new URL(path)
    : new URL(cleanPath, cleanBase);

  if (params) {
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        url.searchParams.append(key, String(value));
      }
    });
  }
  return url.toString();
}

export async function apiFetch<T>(path: string, opts: RequestOptions = {}): Promise<T> {
  const {
    method = 'GET',
    body,
    token,
    params,
    headers: customHeaders,
    timeout = 15000,
  } = opts;

  const url = buildUrl(path, params);
  const headers: Record<string, string> = { ...customHeaders };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  let requestBody: BodyInit | undefined;
  if (body instanceof FormData || body instanceof Blob || body instanceof URLSearchParams) {
    requestBody = body;
  } else if (body) {
    headers['Content-Type'] = 'application/json';
    requestBody = JSON.stringify(body);
  }

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);

  try {
    const res = await fetch(url, {
      method,
      headers,
      body: requestBody,
      signal: controller.signal,
    });

    if (!res.ok) {
      let errorData: unknown;
      const contentType = res.headers.get('content-type');
      if (contentType?.includes('application/json')) {
        errorData = await res.json().catch(() => null);
      } else {
        errorData = await res.text().catch(() => null);
      }
      throw new ApiError(res.status, res.statusText, errorData, url, method);
    }

    return await res.json() as T;
  } catch (err) {
    if (err instanceof ApiError) throw err;
    if (err instanceof DOMException && err.name === 'AbortError') {
      throw new Error(`Request timed out after ${timeout}ms`);
    }
    throw new Error(`Network error: ${(err as Error).message}`);
  } finally {
    clearTimeout(timer);
  }
}

export { ENDPOINTS };
