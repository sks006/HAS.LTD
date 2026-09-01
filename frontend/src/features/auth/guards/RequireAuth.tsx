import React, { type ReactNode } from 'react';
import { Lock } from 'lucide-react';
import type { Role, Permission } from '@/shared/types/roles';
import { can } from '@/shared/types/roles';
import { useRootStore } from '@/slicers/root_store';

type GuardProps = { children: ReactNode; fallback?: ReactNode };

function useCurrentRole(): Role | null {
  return useRootStore((s) => s.user?.role ?? null);
}

export function RequireAuth({ children, fallback }: GuardProps) {
  const user = useRootStore((s) => s.user);
  if (!user) return <>{fallback ?? <AccessDenied required="authenticated" />}</>;
  return <>{children}</>;
}

export function RequireModerator({ children, fallback }: GuardProps) {
  const role = useCurrentRole();
  if (role !== 'MODERATOR' && role !== 'SUPER_ADMIN') {
    return <>{fallback ?? <AccessDenied required="MODERATOR or SUPER_ADMIN" />}</>;
  }
  return <>{children}</>;
}

export function RequireAdmin({ children, fallback }: GuardProps) {
  const role = useCurrentRole();
  if (role !== 'SUPER_ADMIN') {
    return <>{fallback ?? <AccessDenied required="SUPER_ADMIN" />}</>;
  }
  return <>{children}</>;
}

export function RequirePermission({
  permission,
  children,
  fallback,
}: {
  permission: Permission;
  children: ReactNode;
  fallback?: ReactNode;
}) {
  const role = useCurrentRole();
  if (!role || !can(role, permission)) {
    return <>{fallback ?? <AccessDenied required={permission} />}</>;
  }
  return <>{children}</>;
}

function AccessDenied({ required }: { required: string }) {
  return (
    <div className="flex flex-col items-center justify-center rounded-xl border border-slate-800 bg-slate-900 px-6 py-16 text-center shadow-2xl">
      <div className="mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-rose-500/10 text-rose-400">
        <Lock size={24} />
      </div>
      <h3 className="text-xl font-bold text-slate-100">Access Restricted</h3>
      <p className="mt-2 max-w-sm text-sm text-slate-400">
        This area requires <strong className="font-semibold text-slate-200">{required}</strong> access.
      </p>
    </div>
  );
}

export default RequireAuth;
