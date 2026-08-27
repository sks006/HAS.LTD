import React from 'react';
import { Provider } from 'react-redux';
import { store } from '../shared/store';
import { AuthProvider } from './providers/AuthProvider';
import { QueryClientProvider } from './providers/QueryClientProvider';

interface ProvidersProps {
  children: React.ReactNode;
}

export default function Providers({ children }: ProvidersProps) {
  return (
    <Provider store={store}>
      <QueryClientProvider>
        <AuthProvider>
          {children}
        </AuthProvider>
      </QueryClientProvider>
    </Provider>
  );
}
