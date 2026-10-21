import React, { useState } from 'react';
import { useRootStore } from '@/slicers/root_store';
import { loginFetch } from '@/slicers/auth_slicer/auth_fetch';
import type { Role } from '@/shared/types/roles';
import Button from '@/shared/ui/Button';

export const LoginForm: React.FC<{ onSuccess?: () => void }> = ({ onSuccess }) => {
  const [email, setEmail] = useState('admin@ajrahnoor.ltd');
  const [password, setPassword] = useState('admin123');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const setAuth = useRootStore((s) => s.setAuth);

  const performLoginWithRole = (targetEmail: string, role: Role) => {
    setAuth(
      {
        id: `user-${role.toLowerCase()}`,
        email: targetEmail,
        role: role,
      },
      `token-${Date.now()}`
    );
    onSuccess?.();
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const { user, token } = await loginFetch(email, password);
      setAuth(user, token);
      onSuccess?.();
    } catch {
      // Offline / Demo fallback based on entered email
      let userRole: Role = 'CUSTOMER';
      if (email.toLowerCase().includes('admin')) {
        userRole = 'SUPER_ADMIN';
      } else if (email.toLowerCase().includes('mod')) {
        userRole = 'MODERATOR';
      }
      performLoginWithRole(email, userRole);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-sm w-full mx-auto space-y-4">
      <form
        onSubmit={handleSubmit}
        className="space-y-4 p-6 bg-white border border-lightgray rounded-2xl shadow-sm"
      >
        <h2 className="text-xl font-extrabold text-navy text-center font-serif">Sign In to Portal</h2>
        {error && <p className="text-xs text-rose-600 font-medium">{error}</p>}

        <div>
          <label className="block text-xs font-bold uppercase tracking-wider text-slateblue mb-1">
            Email Address
          </label>
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
            className="w-full px-3.5 py-2.5 bg-white border border-lightgray rounded-xl text-navy placeholder-slateblue/40 focus:outline-none focus:border-navy text-xs font-medium"
            placeholder="admin@ajrahnoor.ltd"
          />
        </div>

        <div>
          <label className="block text-xs font-bold uppercase tracking-wider text-slateblue mb-1">
            Password
          </label>
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            className="w-full px-3.5 py-2.5 bg-white border border-lightgray rounded-xl text-navy placeholder-slateblue/40 focus:outline-none focus:border-navy text-xs font-medium"
            placeholder="••••••••"
          />
        </div>

        <Button type="submit" variant="primary" className="w-full rounded-xl" isLoading={loading}>
          Sign In
        </Button>
      </form>

      {/* Quick Demo Role Selectors */}
      <div className="p-4 bg-cream/50 border border-lightgray rounded-2xl space-y-2 text-xs">
        <p className="font-bold text-navy text-center uppercase tracking-wider text-[10px]">
          Quick One-Click Demo Logins:
        </p>
        <div className="grid grid-cols-3 gap-2 pt-1">
          <button
            type="button"
            onClick={() => {
              setEmail('admin@ajrahnoor.ltd');
              setPassword('admin123');
              performLoginWithRole('admin@ajrahnoor.ltd', 'SUPER_ADMIN');
            }}
            className="px-2 py-1.5 bg-navy text-white text-[10px] font-bold rounded-lg hover:bg-navy/90 transition-colors"
          >
            Admin
          </button>
          <button
            type="button"
            onClick={() => {
              setEmail('moderator@ajrahnoor.ltd');
              setPassword('mod123');
              performLoginWithRole('moderator@ajrahnoor.ltd', 'MODERATOR');
            }}
            className="px-2 py-1.5 bg-taupe text-white text-[10px] font-bold rounded-lg hover:bg-taupe/90 transition-colors"
          >
            Moderator
          </button>
          <button
            type="button"
            onClick={() => {
              setEmail('customer@ajrahnoor.ltd');
              setPassword('user123');
              performLoginWithRole('customer@ajrahnoor.ltd', 'CUSTOMER');
            }}
            className="px-2 py-1.5 bg-white border border-lightgray text-navy text-[10px] font-bold rounded-lg hover:bg-cream transition-colors"
          >
            Customer
          </button>
        </div>
      </div>
    </div>
  );
};

export default LoginForm;
