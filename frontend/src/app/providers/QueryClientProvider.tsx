import React, { useState } from 'react';
import { QueryClient, QueryClientProvider as ReactQueryClientProvider } from '@tanstack/react-query';

interface QueryClientProviderProps {
  children: React.ReactNode;
}

export const QueryClientProvider: React.FC<QueryClientProviderProps> = ({ children }) => {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            refetchOnWindowFocus: false,
            retry: 1,
            staleTime: 5 * 60 * 1000,
          },
        },
      })
  );

  return (
    <ReactQueryClientProvider client={queryClient}>
      {children}
    </ReactQueryClientProvider>
  );
};

export default QueryClientProvider;
