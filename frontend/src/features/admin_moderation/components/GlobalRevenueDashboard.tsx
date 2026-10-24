import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Activity, Cpu, Database, Radio, RefreshCw } from 'lucide-react';
import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import { Card, CardContent } from '@/shared/ui/card';
import { Badge } from '@/shared/ui/badge';
import { Button } from '@/shared/ui/button';

interface SystemMetrics {
  active_connections: number;
  database_status: string;
  redis_status: string;
  cpu_usage_pct: number;
}

const metricsConfig = [
  { key: 'active_connections', label: 'Active Connections', icon: Activity, color: 'text-indigo-600', bg: 'bg-indigo-50' },
  { key: 'database_status', label: 'PostgreSQL 16', icon: Database, color: 'text-emerald-600', bg: 'bg-emerald-50' },
  { key: 'redis_status', label: 'Redis Cluster', icon: Radio, color: 'text-amber-600', bg: 'bg-amber-50' },
  { key: 'cpu_usage_pct', label: 'Axum CPU Load', icon: Cpu, color: 'text-rose-600', bg: 'bg-rose-50' },
] as const;

export function GlobalRevenueDashboard() {
  const query = useQuery({
    queryKey: ['admin-system-metrics'],
    queryFn: () => apiFetch<SystemMetrics>(ENDPOINTS.ADMIN_METRICS),
    refetchInterval: 15_000,
  });

  if (query.isPending) {
    return (
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4 animate-pulse">
        {[1, 2, 3, 4].map((i) => (
          <div key={i} className="h-28 rounded-2xl bg-neutral-100 border border-neutral-200" />
        ))}
      </div>
    );
  }

  if (query.isError) {
    return (
      <div role="alert" className="rounded-2xl border border-rose-200 bg-rose-50 p-4 text-xs text-rose-700 flex items-center justify-between">
        <span>Unable to load backend system telemetry: {query.error instanceof Error ? query.error.message : 'Backend unreachable.'}</span>
        <Button variant="outline" size="sm" onClick={() => query.refetch()} className="h-7 text-xs bg-white">
          Retry
        </Button>
      </div>
    );
  }

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      {metricsConfig.map(({ key, label, icon: Icon, color, bg }) => {
        const rawValue = query.data?.[key];
        const displayValue = key === 'cpu_usage_pct' ? `${rawValue}%` : rawValue;
        const isOnline = rawValue === 'Online' || (typeof rawValue === 'number' && rawValue >= 0);

        return (
          <Card key={key} className="border-neutral-200/80 shadow-2xs bg-white hover:shadow-xs transition-shadow">
            <CardContent className="p-5 flex items-center justify-between">
              <div className="space-y-1">
                <span className="text-[10px] font-bold uppercase tracking-wider text-neutral-400 block font-mono">
                  {label}
                </span>
                <h4 className="text-2xl font-extrabold text-neutral-900 font-sans tracking-tight">
                  {displayValue}
                </h4>
                <Badge
                  variant={isOnline ? 'secondary' : 'destructive'}
                  className="text-[9px] font-mono px-1.5 py-0"
                >
                  {key.includes('status') ? 'Healthy' : 'Active Ingress'}
                </Badge>
              </div>

              <div className={`w-12 h-12 rounded-xl flex items-center justify-center ${bg} ${color}`}>
                <Icon size={22} />
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}

export default GlobalRevenueDashboard;
