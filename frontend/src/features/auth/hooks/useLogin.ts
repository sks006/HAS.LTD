import { useMutation } from '@tanstack/react-query';
import { apiClient } from '../../../shared/api/client';
import { API_ENDPOINTS } from '../../../shared/api/endpoints';
import { useAuthStore } from '../store/authSlice';

export function useLogin() {
  const setAuth = useAuthStore((state) => state.setAuth);

  return useMutation({
    mutationFn: async (credentials: any) => {
      const response = await apiClient.post(API_ENDPOINTS.login, credentials);
      return response.data;
    },
    onSuccess: (data) => {
      if (data.token) {
        setAuth(data.token, data.role || 'CUSTOMER');
      }
    },
  });
}
