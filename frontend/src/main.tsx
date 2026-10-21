import React, { Component, ErrorInfo, ReactNode } from 'react';
import ReactDOM from 'react-dom/client';
import Providers from '@/app/providers';
import { AppRouter } from '@/app/router';
import '@/app/globals.css';

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class GlobalErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  public state: ErrorBoundaryState = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Unhandled Global UI Error:', error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
          <div className="max-w-md w-full p-8 bg-slate-900 border border-rose-500/30 rounded-2xl text-center space-y-4 shadow-2xl">
            <span className="text-4xl">⚠️</span>
            <h2 className="text-xl font-bold text-rose-400">Application Error</h2>
            <p className="text-xs text-slate-400 font-mono bg-slate-950 p-3 rounded-lg overflow-x-auto text-left">
              {this.state.error?.toString()}
            </p>
            <button
              onClick={() => window.location.reload()}
              className="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white font-medium text-sm rounded-lg transition-colors"
            >
              Reload Application
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <GlobalErrorBoundary>
      <Providers>
        <AppRouter />
      </Providers>
    </GlobalErrorBoundary>
  </React.StrictMode>
);
