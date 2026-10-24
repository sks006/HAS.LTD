import { useMutation } from '@tanstack/react-query';
import { loginFetch, normalizeBackendUser } from '@/slicers/auth_slicer/auth_fetch';
import { useRootStore } from '@/slicers/root_store';
import { setApiToken } from '@/shared/api/client';

export function useLogin() {
  const setAuth = useRootStore((s) => s.setAuth);

  return useMutation({
    mutationFn: async (credentials: { email: string; password: string }) => {
      const response = await loginFetch(credentials.email, credentials.password);
      if (!response.access_token || !response.user) {
        throw new Error('Additional MFA verification is required. Use the sign-in form to continue.');
      }
      return { user: normalizeBackendUser(response.user), token: response.access_token };
    },
    onSuccess: (data) => {
      setApiToken(data.token);
      setAuth(data.user, data.token);
    },
  });
}
