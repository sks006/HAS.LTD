import { useMutation } from '@tanstack/react-query';
import { loginFetch } from '@/slicers/auth_slicer/auth_fetch';
import { useRootStore } from '@/slicers/root_store';

export function useLogin() {
  const setAuth = useRootStore((s) => s.setAuth);

  return useMutation({
    mutationFn: async (credentials: { email: string; password_hash: string }) => {
      return loginFetch(credentials.email, credentials.password_hash);
    },
    onSuccess: (data) => {
      if (data.token) {
        setAuth(data.user, data.token);
      }
    },
  });
}
