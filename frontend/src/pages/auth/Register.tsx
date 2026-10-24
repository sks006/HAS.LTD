import React from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { RegisterForm } from '@/features/auth/components/RegisterForm';
import { ArrowLeft } from 'lucide-react';
import { Button } from '@/shared/ui/button';

export const Register: React.FC = () => {
  const navigate = useNavigate();

  return (
    <main className="min-h-screen bg-[#fafafa] flex flex-col justify-center items-center p-6 relative font-sans text-neutral-800 select-none">
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
        <header className="text-center space-y-2">
          <div className="w-14 h-14 rounded-2xl bg-[#1c1917] text-amber-300 mx-auto flex items-center justify-center text-xl font-serif font-black shadow-md border border-neutral-800">
            AN
          </div>
          <h1 className="text-2xl font-extrabold text-neutral-900 font-serif">
            Create Atelier Account
          </h1>
          <p className="text-xs text-neutral-500 font-medium max-w-sm mx-auto">
            Register with your email address and verify with a 6-digit OTP code to activate your account.
          </p>
        </header>

        <RegisterForm onSuccess={() => navigate('/login')} />

        <p className="text-center text-xs text-neutral-500">
          Already registered?{' '}
          <Link to="/login" className="font-semibold text-amber-800 hover:text-neutral-900 underline">
            Sign in to portal
          </Link>
        </p>
      </div>
    </main>
  );
};

export default Register;
