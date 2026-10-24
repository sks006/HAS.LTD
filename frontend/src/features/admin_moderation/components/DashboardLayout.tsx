import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { AddProductModal } from './AddProductModal';
import { useAuth } from '@/app/providers/AuthProvider';
import { Button } from '@/shared/ui/button';
import { Badge } from '@/shared/ui/badge';
import {
  ShoppingBag,
  ChevronsLeft,
  ChevronsRight,
  Search,
  PlusCircle,
  Monitor,
  LayoutDashboard,
  ShieldCheck,
  ShieldAlert,
  Package,
  PackagePlus,
  TrendingDown,
  FolderTree,
  ShoppingCart,
  Users,
  UserCheck,
  Menu,
  ChevronDown,
  ChevronRight,
  LogOut,
  User,
  Shield,
} from 'lucide-react';

interface DashboardLayoutProps {
  children: React.ReactNode;
  activeTab?: string;
  onTabChange?: (tab: string) => void;
}

export const DashboardLayout: React.FC<DashboardLayoutProps> = ({
  children,
  activeTab = 'Products',
  onTabChange,
}) => {
  const navigate = useNavigate();
  const { user, role, logout } = useAuth();
  const [collapsed, setCollapsed] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);

  const isSuperAdmin = role === 'SUPER_ADMIN';

  // Collapsible section states
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    Main: true,
    Inventory: true,
    'User Management': true,
  });

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => ({ ...prev, [section]: !prev[section] }));
  };

  const handleNavClick = (tabLabel: string) => {
    if (tabLabel === 'Create Product' || tabLabel === 'Add Product') {
      if (isSuperAdmin) {
        setIsAddModalOpen(true);
      }
      return;
    }
    if (tabLabel === 'Moderator Audit') {
      navigate('/moderator');
      return;
    }
    if (tabLabel === 'Orders') {
      navigate('/dashboard/orders');
      return;
    }
    if (tabLabel === 'Dashboard' || tabLabel === 'Products') {
      navigate('/dashboard');
      return;
    }
    if (onTabChange) {
      onTabChange(tabLabel);
    }
  };

  // Build sidebar groups dynamically based on user role
  const sidebarGroups: {
    title: string;
    adminOnly?: boolean;
    items: { label: string; icon: any; hasSub?: boolean; adminOnly?: boolean }[];
  }[] = [
    {
      title: 'Main',
      items: [
        { label: 'Dashboard', icon: LayoutDashboard, hasSub: false },
        { label: 'Orders', icon: ShoppingCart, hasSub: false },
        ...(isSuperAdmin
          ? [{ label: 'Super Admin', icon: ShieldCheck, hasSub: true, adminOnly: true }]
          : []),
        { label: 'Moderator Audit', icon: ShieldAlert, hasSub: false },
      ],
    },
    {
      title: 'Inventory',
      items: [
        { label: 'Products', icon: Package, hasSub: false },
        ...(isSuperAdmin
          ? [{ label: 'Create Product', icon: PackagePlus, hasSub: false, adminOnly: true }]
          : []),
        { label: 'Low Stocks', icon: TrendingDown, hasSub: false },
        { label: 'Category', icon: FolderTree, hasSub: false },
      ],
    },
    ...(isSuperAdmin
      ? [
          {
            title: 'User Management',
            adminOnly: true,
            items: [
              { label: 'Users', icon: Users, hasSub: false, adminOnly: true },
              { label: 'Roles & Permissions', icon: UserCheck, hasSub: false, adminOnly: true },
            ],
          },
        ]
      : []),
  ];

  const SidebarContent = (
    <div
      className={`flex flex-col h-full bg-white border-r border-neutral-200 text-neutral-700 transition-all duration-300 select-none ${
        collapsed ? 'w-20' : 'w-64'
      }`}
    >
      {/* Brand Header */}
      <div className="p-4 flex items-center justify-between border-b border-neutral-100">
        {!collapsed && (
          <div
            className="flex items-center space-x-2.5 cursor-pointer"
            onClick={() => handleNavClick('Products')}
          >
            <div className="w-8 h-8 rounded-xl bg-neutral-900 flex items-center justify-center text-amber-400 shadow-xs">
              <ShoppingBag className="w-5 h-5" />
            </div>
            <div className="flex flex-col">
              <div className="flex items-center">
                <span className="font-extrabold text-base text-neutral-900 tracking-tight font-serif">
                  Ajrah Noor
                </span>
                <span className="text-[10px] text-amber-700 font-bold ml-1.5 uppercase font-mono tracking-wider">
                  Portal
                </span>
              </div>
              <span className="text-[9px] uppercase font-mono tracking-widest text-neutral-400">
                Staff Control
              </span>
            </div>
          </div>
        )}

        {collapsed && (
          <div className="mx-auto w-8 h-8 rounded-xl bg-neutral-900 flex items-center justify-center text-amber-400">
            <ShoppingBag className="w-5 h-5" />
          </div>
        )}

        {/* Collapse Toggle Button */}
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="hidden md:flex w-7 h-7 rounded-full bg-neutral-100 hover:bg-neutral-200 text-neutral-700 items-center justify-center shadow-xs transition-transform cursor-pointer"
        >
          {collapsed ? <ChevronsRight className="w-4 h-4" /> : <ChevronsLeft className="w-4 h-4" />}
        </button>
      </div>

      {/* Role Indicator Banner */}
      {!collapsed && (
        <div className="px-4 py-2.5 bg-neutral-50/80 border-b border-neutral-100 flex items-center justify-between">
          <div className="flex items-center space-x-2 truncate">
            <User className="w-3.5 h-3.5 text-neutral-400 shrink-0" />
            <span className="text-xs font-semibold text-neutral-700 truncate max-w-[120px]">
              {user?.email?.split('@')[0] || 'Staff'}
            </span>
          </div>
          <Badge
            variant={isSuperAdmin ? 'default' : 'secondary'}
            className={`font-mono text-[9px] uppercase font-bold ${
              isSuperAdmin
                ? 'bg-amber-700 text-white hover:bg-amber-800'
                : 'bg-purple-100 text-purple-800 border-purple-200'
            }`}
          >
            {role || 'Staff'}
          </Badge>
        </div>
      )}

      {/* Sidebar Navigation */}
      <div className="flex-1 overflow-y-auto px-3 py-4 space-y-5 text-xs">
        {sidebarGroups.map((group) => {
          const isExpanded = expandedSections[group.title] ?? true;

          return (
            <div key={group.title} className="space-y-1">
              {!collapsed && (
                <div
                  onClick={() => toggleSection(group.title)}
                  className="flex items-center justify-between px-2 py-1 text-[10px] font-bold text-neutral-400 uppercase tracking-wider cursor-pointer hover:text-neutral-600"
                >
                  <span className="flex items-center space-x-1">
                    <span>{group.title}</span>
                    {group.adminOnly && (
                      <span className="text-[9px] text-amber-700 font-mono">(Admin)</span>
                    )}
                  </span>
                  <ChevronDown
                    className={`w-3 h-3 transition-transform ${isExpanded ? 'rotate-0' : '-rotate-90'}`}
                  />
                </div>
              )}

              <nav className="space-y-0.5">
                {isExpanded &&
                  group.items.map((item) => {
                    const isActive = activeTab === item.label;
                    const Icon = item.icon;

                    return (
                      <button
                        key={item.label}
                        onClick={() => handleNavClick(item.label)}
                        className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all cursor-pointer ${
                          isActive
                            ? 'bg-neutral-900 text-white shadow-2xs'
                            : 'text-neutral-600 hover:bg-neutral-100 hover:text-neutral-900'
                        }`}
                        title={collapsed ? item.label : undefined}
                      >
                        <div className="flex items-center space-x-2.5 truncate">
                          <Icon
                            className={`w-4 h-4 shrink-0 ${
                              isActive ? 'text-amber-400' : 'text-neutral-400'
                            }`}
                          />
                          {!collapsed && <span className="truncate">{item.label}</span>}
                        </div>

                        {!collapsed && item.hasSub && (
                          <ChevronRight className="w-3.5 h-3.5 text-neutral-400 shrink-0" />
                        )}
                      </button>
                    );
                  })}
              </nav>
            </div>
          );
        })}
      </div>

      {/* Footer Controls: Storefront + Logout */}
      <div className="p-3 border-t border-neutral-100 space-y-1.5 bg-neutral-50/50">
        <Button
          variant="outline"
          size="sm"
          onClick={() => navigate('/')}
          className="w-full text-xs font-semibold justify-start rounded-xl h-8 text-neutral-700"
        >
          <Monitor className="w-3.5 h-3.5 mr-2 text-neutral-500" />
          {!collapsed && <span>Storefront</span>}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={logout}
          className="w-full text-xs font-semibold justify-start rounded-xl h-8 text-rose-600 hover:text-rose-700 hover:bg-rose-50"
        >
          <LogOut className="w-3.5 h-3.5 mr-2 text-rose-500" />
          {!collapsed && <span>Sign Out</span>}
        </Button>
      </div>
    </div>
  );

  return (
    <div className="min-h-screen bg-neutral-50 text-neutral-800 flex font-sans antialiased">
      {isSuperAdmin && (
        <AddProductModal isOpen={isAddModalOpen} onClose={() => setIsAddModalOpen(false)} />
      )}

      {/* Desktop Sidebar */}
      <aside className="hidden md:block shrink-0">{SidebarContent}</aside>

      {/* Mobile Drawer Overlay */}
      {mobileOpen && (
        <div className="fixed inset-0 z-50 flex md:hidden">
          <div className="fixed inset-0 bg-black/40 backdrop-blur-xs" onClick={() => setMobileOpen(false)} />
          <div className="relative z-10">{SidebarContent}</div>
        </div>
      )}

      {/* Main Container */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Top Header */}
        <header className="bg-white border-b border-neutral-200 px-4 md:px-6 py-2.5 flex items-center justify-between sticky top-0 z-30 shadow-2xs">
          <div className="flex items-center space-x-3">
            <button
              onClick={() => setMobileOpen(!mobileOpen)}
              className="p-1.5 text-neutral-600 md:hidden hover:bg-neutral-100 rounded-lg"
            >
              <Menu className="w-5 h-5" />
            </button>

            {/* Global Search Input */}
            <div className="relative hidden sm:block w-64 md:w-72">
              <Search className="w-4 h-4 text-neutral-400 absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                placeholder="Search products or orders..."
                className="w-full pl-9 pr-12 py-1.5 text-xs bg-neutral-50 border border-neutral-200 rounded-xl text-neutral-800 placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-amber-500/20 focus:border-amber-500 transition-all"
              />
              <span className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[10px] font-semibold text-neutral-400 border border-neutral-200 rounded px-1.5 py-0.5 bg-white font-mono shadow-2xs">
                ⌘ K
              </span>
            </div>
          </div>

          {/* Right Header Actions */}
          <div className="flex items-center space-x-2 md:space-x-3 text-xs">
            {/* Role Badge in Header */}
            <div className="flex items-center space-x-1.5">
              <Badge
                variant={isSuperAdmin ? 'default' : 'secondary'}
                className={`text-[10px] font-mono uppercase font-bold tracking-wider px-2.5 py-1 ${
                  isSuperAdmin
                    ? 'bg-amber-700 text-white'
                    : 'bg-purple-100 text-purple-800 border-purple-200'
                }`}
              >
                <Shield className="w-3 h-3 mr-1" />
                <span>{role || 'STAFF'}</span>
              </Badge>
            </div>

            {/* + Add Product Button - ONLY for Super Admin */}
            {isSuperAdmin && (
              <Button
                variant="luxury"
                size="sm"
                onClick={() => setIsAddModalOpen(true)}
                className="rounded-xl h-8 text-xs font-bold"
              >
                <PlusCircle className="w-3.5 h-3.5 mr-1 text-amber-400" />
                <span>Add Product</span>
              </Button>
            )}

            {/* Storefront Navigation */}
            <Button
              variant="outline"
              size="sm"
              onClick={() => navigate('/')}
              className="rounded-xl h-8 text-xs font-semibold text-neutral-700"
            >
              <Monitor className="w-3.5 h-3.5 mr-1.5 text-neutral-500" />
              <span>Storefront</span>
            </Button>
          </div>
        </header>

        {/* Main Content Workspace */}
        <main className="flex-1 p-4 md:p-6 lg:p-8 overflow-y-auto">{children}</main>
      </div>
    </div>
  );
};

export default DashboardLayout;
