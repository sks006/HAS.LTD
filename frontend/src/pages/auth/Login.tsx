import React from 'react';
import { LoginForm } from '@/features/auth/components/LoginForm';
import { useNavigate } from 'react-router-dom';

export const Login: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-white flex flex-col justify-center items-center p-6 relative font-sans">
      <button
        onClick={() => navigate('/')}
        className="absolute top-6 left-6 text-xs font-semibold uppercase tracking-wider text-slateblue hover:text-navy"
      >
        ← Return to Storefront
      </button>

      <div className="w-full max-w-md space-y-6">
        <div className="text-center space-y-2">
          <div className="w-14 h-14 rounded-2xl bg-navy mx-auto flex items-center justify-center font-serif text-white text-2xl font-black shadow-md">
            AN
          </div>
          <h1 className="text-3xl font-extrabold text-navy font-serif tracking-tight">Ajrah Noor Portal</h1>
          <p className="text-xs text-slateblue font-medium">Sign in to access executive atelier & inventory services</p>
        </div>

        <LoginForm onSuccess={() => navigate('/dashboard/admin')} />
      </div>
    </div>
  );
};

export default Login;
