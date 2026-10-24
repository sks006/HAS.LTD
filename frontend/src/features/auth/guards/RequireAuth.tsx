import React, { type ReactNode } from 'react';
import { Navigate, useLocation, useNavigate } from 'react-router-dom';
import { ShieldAlert, ArrowLeft, LogOut, Loader2, Sparkles, User } from 'lucide-react';
import type { Role, Permission } from '@/shared/types/roles';
import { can } from '@/shared/types/roles';
import { useAuth } from '@/app/providers/AuthProvider';
import { Button } from '@/shared/ui/button';
import { Badge } from '@/shared/ui/badge';
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/shared/ui/card';

type GuardProps = { children: ReactNode; fallback?: ReactNode };

/**
 * Loading screen displayed while backend session hydration is occurring.
 */
function AuthLoadingState() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-neutral-50 font-sans">
      <div className="flex flex-col items-center space-y-4 p-8 bg-white rounded-2xl border border-neutral-200 shadow-sm text-center">
        <div className="w-12 h-12 rounded-full bg-neutral-900 flex items-center justify-center text-amber-400">
          <Loader2 className="w-6 h-6 animate-spin text-amber-400" />
        </div>
        <div className="space-y-1">
          <h4 className="font-serif font-bold text-base text-neutral-900">Ajrah Noor Atelier</h4>
          <p className="text-xs font-mono uppercase tracking-widest text-neutral-400">
            Verifying Role Clearance & Security Credentials...
          </p>
        </div>
      </div>
    </div>
  );
}

/**
 * Access Denied UI matching the luxury haute couture aesthetic.
 */
export function AccessDenied({
  required,
  currentRole,
  userEmail,
}: {
  required: string;
  currentRole?: Role | null;
  userEmail?: string;
}) {
  const navigate = useNavigate();
  const { logout } = useAuth();

  return (
    <div className="min-h-screen flex items-center justify-center bg-neutral-50 p-6 font-sans">
      <Card className="max-w-md w-full bg-white border-neutral-200/90 shadow-xl rounded-3xl overflow-hidden">
        <div className="h-2 bg-gradient-to-r from-amber-700 via-rose-700 to-amber-900" />

        <CardHeader className="text-center pt-8 pb-4">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-rose-50 text-rose-600 border border-rose-200/80 shadow-xs">
            <ShieldAlert size={32} />
          </div>
          <CardTitle className="text-2xl font-serif font-extrabold text-neutral-900">
            Access Restricted
          </CardTitle>
          <CardDescription className="text-xs text-neutral-500 max-w-xs mx-auto mt-1 leading-relaxed">
            The Ajrah Noor Atelier Portal requires verified Staff clearance.
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4 pt-2">
          <div className="rounded-2xl bg-neutral-50 p-4 border border-neutral-200/80 space-y-2.5 text-xs">
            <div className="flex items-center justify-between text-neutral-500">
              <span>Required Role:</span>
              <Badge variant="outline" className="font-mono text-[10px] uppercase font-bold border-amber-300 text-amber-800 bg-amber-50">
                {required}
              </Badge>
            </div>

            <div className="flex items-center justify-between text-neutral-500">
              <span>Active Account:</span>
              <span className="font-semibold text-neutral-900 truncate max-w-[180px]">
                {userEmail || 'Authenticated User'}
              </span>
            </div>

            <div className="flex items-center justify-between text-neutral-500">
              <span>Assigned Clearance:</span>
              <Badge variant="secondary" className="font-mono text-[10px] uppercase">
                {currentRole || 'CUSTOMER'}
              </Badge>
            </div>
          </div>

          <p className="text-[11px] text-neutral-400 text-center leading-relaxed">
            Your current account credentials do not possess administrative permissions for this atelier console.
          </p>
        </CardContent>

        <CardFooter className="flex flex-col sm:flex-row items-center gap-2.5 pt-2 pb-6 px-6">
          <Button
            variant="luxury"
            size="sm"
            onClick={() => navigate('/')}
            className="w-full sm:flex-1 h-10 rounded-xl text-xs font-bold"
          >
            <ArrowLeft size={14} className="mr-1.5" />
            <span>Boutique Storefront</span>
          </Button>

          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              logout();
              navigate('/login');
            }}
            className="w-full sm:flex-1 h-10 rounded-xl text-xs font-semibold text-neutral-700"
          >
            <LogOut size={14} className="mr-1.5 text-rose-600" />
            <span>Switch Staff Account</span>
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}

/**
 * Requires the user to be logged in with ANY role.
 */
export function RequireAuth({ children, fallback }: GuardProps) {
  const location = useLocation();
  const { user, isAuthenticated, isLoading } = useAuth();

  if (isLoading) return <AuthLoadingState />;
  if (!isAuthenticated || !user) {
    return fallback ? <>{fallback}</> : <Navigate to="/login" state={{ from: location.pathname }} replace />;
  }
  return <>{children}</>;
}

/**
 * Requires user to have Staff clearance (SUPER_ADMIN or MODERATOR).
 * Customer accounts are denied access.
 */
export function RequireStaff({ children, fallback }: GuardProps) {
  const location = useLocation();
  const { user, role, isAuthenticated, isLoading } = useAuth();

  if (isLoading) return <AuthLoadingState />;

  if (!isAuthenticated || !user) {
    return fallback ? <>{fallback}</> : <Navigate to="/login" state={{ from: location.pathname }} replace />;
  }

  const isStaff = role === 'SUPER_ADMIN' || role === 'MODERATOR';
  if (!isStaff) {
    return (
      fallback ? (
        <>{fallback}</>
      ) : (
        <AccessDenied required="SUPER_ADMIN or MODERATOR" currentRole={role} userEmail={user.email} />
      )
    );
  }

  return <>{children}</>;
}

/**
 * Requires user to have MODERATOR or SUPER_ADMIN access.
 */
export function RequireModerator({ children, fallback }: GuardProps) {
  return <RequireStaff fallback={fallback}>{children}</RequireStaff>;
}

/**
 * Requires user to have SUPER_ADMIN access specifically.
 * Moderator and Customer accounts are denied.
 */
export function RequireAdmin({ children, fallback }: GuardProps) {
  const location = useLocation();
  const { user, role, isAuthenticated, isLoading } = useAuth();

  if (isLoading) return <AuthLoadingState />;

  if (!isAuthenticated || !user) {
    return fallback ? <>{fallback}</> : <Navigate to="/login" state={{ from: location.pathname }} replace />;
  }

  if (role !== 'SUPER_ADMIN') {
    return (
      fallback ? (
        <>{fallback}</>
      ) : (
        <AccessDenied required="SUPER_ADMIN" currentRole={role} userEmail={user.email} />
      )
    );
  }

  return <>{children}</>;
}

/**
 * Requires user to possess a specific fine-grained permission.
 */
export function RequirePermission({
  permission,
  children,
  fallback,
}: {
  permission: Permission;
  children: ReactNode;
  fallback?: ReactNode;
}) {
  const location = useLocation();
  const { user, role, isAuthenticated, isLoading } = useAuth();

  if (isLoading) return <AuthLoadingState />;

  if (!isAuthenticated || !user) {
    return fallback ? <>{fallback}</> : <Navigate to="/login" state={{ from: location.pathname }} replace />;
  }

  if (!role || !can(role, permission)) {
    return (
      fallback ? (
        <>{fallback}</>
      ) : (
        <AccessDenied required={`Permission: ${permission}`} currentRole={role} userEmail={user.email} />
      )
    );
  }

  return <>{children}</>;
}

export default RequireAuth;
