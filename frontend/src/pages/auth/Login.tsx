import React from 'react';
import { LoginForm } from '@/features/auth/components/LoginForm';
import { useNavigate } from 'react-router-dom';
import { ShieldCheck, ShieldAlert, ArrowLeft } from 'lucide-react';

export const Login: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-[#f8fafc] flex flex-col justify-center items-center p-6 relative font-sans text-gray-800">
      <button
        onClick={() => navigate('/')}
        className="absolute top-6 left-6 text-xs font-semibold text-gray-600 hover:text-gray-900 flex items-center gap-1.5 px-3 py-2 bg-white border border-gray-200 rounded-xl shadow-2xs transition-all"
      >
        <ArrowLeft className="w-4 h-4 text-gray-500" />
        <span>Return to Storefront</span>
      </button>

      <div className="w-full max-w-md space-y-6">
        <div className="text-center space-y-2">
          <div className="w-14 h-14 rounded-2xl bg-gradient-to-tr from-[#1e1b4b] to-[#312e81] mx-auto flex items-center justify-center text-[#f59e0b] text-2xl font-black shadow-md">
            AN
          </div>
          <h1 className="text-2xl font-extrabold text-gray-900 font-serif tracking-tight">
            Ajrah Noor Management Portal
          </h1>
          <p className="text-xs text-gray-500 font-medium">
            Sign in to access Super Admin Dashboard or Moderator Audit Console
          </p>
        </div>

        <LoginForm onSuccess={() => navigate('/dashboard')} />

        <div className="pt-2 text-center flex items-center justify-center gap-4 text-xs font-semibold text-gray-500">
          <button
            onClick={() => navigate('/dashboard')}
            className="flex items-center gap-1 text-[#f59e0b] hover:underline"
          >
            <ShieldCheck className="w-3.5 h-3.5" />
            <span>Admin Dashboard</span>
          </button>
          <span>•</span>
          <button
            onClick={() => navigate('/moderator')}
            className="flex items-center gap-1 text-purple-600 hover:underline"
          >
            <ShieldAlert className="w-3.5 h-3.5" />
            <span>Moderator Zone</span>
          </button>
        </div>
      </div>
    </div>
  );
};

export default Login;
