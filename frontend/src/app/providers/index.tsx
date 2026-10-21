import React from 'react';
import { Provider } from 'react-redux';
import { store } from '@/slicers/root_store';
import { QueryClientProvider } from './QueryClientProvider';
import { AuthProvider } from './AuthProvider';

interface ProvidersProps {
  children: React.ReactNode;
}

export const Providers: React.FC<ProvidersProps> = ({ children }) => {
  return (
    <Provider store={store}>
      <QueryClientProvider>
        <AuthProvider>{children}</AuthProvider>
      </QueryClientProvider>
    </Provider>
  );
};

export default Providers;
