import React, { useEffect, useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { apiClient } from '../../shared/api/client';
import { API_ENDPOINTS } from '../../shared/api/endpoints';
import { Button } from '../../shared/ui/Button';
import { useAuthStore } from '../../features/auth/store/authSlice';

export function DashboardPage() {
  const [metrics, setMetrics] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const { clearAuth } = useAuthStore();
  const navigate = useNavigate();

  useEffect(() => {
    apiClient.get(API_ENDPOINTS.metrics)
      .then((res) => {
        setMetrics(res.data);
        setIsLoading(false);
      })
      .catch(() => {
        setIsLoading(false);
      });
  }, []);

  const handleAdjustInventory = () => {
    apiClient.post(API_ENDPOINTS.inventory, {
      quantity_change: 10,
      reason: 'Restocking system',
    })
      .then((res) => {
        alert(res.data.message);
      })
      .catch((err) => {
        alert(`Failed to adjust inventory: ${err.message}`);
      });
  };

  const handleLogout = () => {
    clearAuth();
    navigate('/login');
  };

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col">
      <header className="border-b border-slate-900 bg-slate-950/80 backdrop-blur-md px-6 py-4 flex justify-between items-center">
        <div className="flex items-center space-x-3">
          <span className="font-extrabold text-xl tracking-wider bg-clip-text bg-gradient-to-r from-indigo-400 to-purple-400">
            HAS.LTD
          </span>
          <span className="bg-amber-950/40 border border-amber-900/50 text-[10px] text-amber-400 px-2 py-0.5 rounded font-mono uppercase">
            Admin Console
          </span>
        </div>

        <nav className="flex items-center space-x-6">
          <Link to="/" className="text-slate-300 hover:text-white font-medium text-sm transition-colors">
            View Storefront
          </Link>
          <button 
            onClick={handleLogout} 
            className="text-slate-400 hover:text-white font-medium text-sm transition-colors"
          >
            Logout
          </button>
        </nav>
      </header>

      <main className="flex-1 max-w-7xl w-full mx-auto px-6 py-10 space-y-8">
        <div>
          <h2 className="text-3xl font-extrabold tracking-tight text-white">System Performance</h2>
          <p className="text-slate-400 text-sm mt-1">Real-time health statistics and operational logs.</p>
        </div>

        {/* Metrics Grid */}
        <section className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl shadow-lg">
            <span className="text-slate-400 text-xs font-semibold uppercase tracking-wider block">Database Status</span>
            <span className="text-2xl font-bold text-emerald-400 block mt-2">
              {isLoading ? 'Loading...' : metrics?.database_status || 'Offline'}
            </span>
          </div>

          <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl shadow-lg">
            <span className="text-slate-400 text-xs font-semibold uppercase tracking-wider block">Active Connections</span>
            <span className="text-2xl font-bold text-white block mt-2">
              {isLoading ? 'Loading...' : metrics?.active_connections || '0'}
            </span>
          </div>

          <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl shadow-lg">
            <span className="text-slate-400 text-xs font-semibold uppercase tracking-wider block">CPU Load</span>
            <span className="text-2xl font-bold text-indigo-400 block mt-2">
              {isLoading ? 'Loading...' : `${metrics?.cpu_usage_pct || '0'}%`}
            </span>
          </div>

          <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl shadow-lg">
            <span className="text-slate-400 text-xs font-semibold uppercase tracking-wider block">Cache Server</span>
            <span className="text-2xl font-bold text-emerald-400 block mt-2">
              {isLoading ? 'Loading...' : metrics?.redis_status || 'Offline'}
            </span>
          </div>
        </section>

        {/* Administration Actions */}
        <section className="bg-slate-900 border border-slate-800 rounded-xl p-8 shadow-xl">
          <h3 className="text-xl font-bold tracking-tight text-white mb-2">Inventory Management</h3>
          <p className="text-slate-400 text-sm mb-6">Restock catalog items and update warehousing ledgers.</p>
          
          <Button variant="primary" onClick={handleAdjustInventory}>
            Perform Mock Restock (+10 units)
          </Button>
        </section>
      </main>
    </div>
  );
}
