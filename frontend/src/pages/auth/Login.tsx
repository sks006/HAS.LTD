import React from 'react';
import { LoginForm } from '@/features/auth/components/LoginForm';
import { useNavigate, Link } from 'react-router-dom';
import { ArrowLeft, Sparkles } from 'lucide-react';
import { Button } from '@/shared/ui/button';

export const Login: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-[#fafafa] flex flex-col justify-center items-center p-6 relative font-sans text-neutral-800 select-none">
      <Button
        variant="outline"
        size="sm"
        onClick={() => navigate('/')}
        className="absolute top-6 left-6 text-xs bg-white border-neutral-200"
      >
        <ArrowLeft className="w-3.5 h-3.5 mr-1.5" />
        <span>Return to Storefront</span>
      </Button>

      <div className="w-full max-w-md space-y-6">
        <div className="text-center space-y-2">
          <div className="w-14 h-14 rounded-2xl bg-[#1c1917] text-amber-300 mx-auto flex items-center justify-center text-xl font-serif font-black shadow-md border border-neutral-800">
            AN
          </div>
          <h1 className="text-2xl font-extrabold text-neutral-900 font-serif tracking-tight">
            Ajrah Noor Atelier Portal
          </h1>
          <p className="text-xs text-neutral-500 font-medium max-w-sm mx-auto">
            Secure client and staff access for order tracking, bespoke tailoring, and catalog management.
          </p>
        </div>

        <LoginForm
          onSuccess={(role) =>
            navigate(
              role === 'SUPER_ADMIN'
                ? '/admin'
                : role === 'MODERATOR'
                ? '/moderator'
                : '/'
            )
          }
        />

        <p className="text-center text-xs text-neutral-500">
          New client?{' '}
          <Link to="/register" className="font-semibold text-amber-800 hover:text-neutral-900 underline">
            Create an account
          </Link>
        </p>
      </div>
    </div>
  );
};

export default Login;
