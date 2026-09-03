import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { AddProductModal } from './AddProductModal';
import {
  ShoppingBag,
  ChevronsLeft,
  ChevronsRight,
  Search,
  PlusCircle,
  Monitor,
  Maximize2,
  Mail,
  Bell,
  Settings,
  ChevronDown,
  ChevronRight,
  LayoutDashboard,
  ShieldCheck,
  AppWindow,
  Layers,
  Package,
  PackagePlus,
  Clock,
  TrendingDown,
  FolderTree,
  Tag,
  Award,
  Ruler,
  Sliders,
  FileCheck,
  Barcode,
  QrCode,
  Boxes,
  SlidersHorizontal,
  ArrowLeftRight,
  ShoppingCart,
  FileText,
  RotateCcw,
  FileSpreadsheet,
  Users,
  UserCheck,
  UserMinus,
  FileCode,
  BookOpen,
  MapPin,
  MessageSquare,
  HelpCircle,
  User,
  Lock,
  AlertTriangle,
  File,
  DollarSign,
  Calendar,
  Menu,
  ShieldAlert,
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
  const [collapsed, setCollapsed] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);

  // Collapsible section states
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    Main: true,
    Inventory: true,
    Stock: true,
    Sales: true,
    'User Management': true,
    'Content (CMS)': false,
    Pages: false,
  });

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => ({ ...prev, [section]: !prev[section] }));
  };

  const handleNavClick = (tabLabel: string) => {
    if (tabLabel === 'Create Product' || tabLabel === 'Add Product') {
      setIsAddModalOpen(true);
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

  const sidebarGroups: {
    title: string;
    items: { label: string; icon: any; hasSub?: boolean }[];
  }[] = [
    {
      title: 'Main',
      items: [
        { label: 'Dashboard', icon: LayoutDashboard, hasSub: true },
        { label: 'Orders', icon: ShoppingCart, hasSub: false },
        { label: 'Super Admin', icon: ShieldCheck, hasSub: true },
        { label: 'Moderator Audit', icon: ShieldAlert, hasSub: true },
        { label: 'Application', icon: AppWindow, hasSub: true },
      ],
    },
    {
      title: 'Inventory',
      items: [
        { label: 'Products', icon: Package, hasSub: false },
        { label: 'Create Product', icon: PackagePlus, hasSub: false },
        { label: 'Low Stocks', icon: TrendingDown, hasSub: false },
        { label: 'Category', icon: FolderTree, hasSub: false },
      ],
    },
    {
      title: 'User Management',
      items: [
        { label: 'Users', icon: Users, hasSub: false },
        { label: 'Roles & Permissions', icon: UserCheck, hasSub: false },
      ],
    },
  ];

  const SidebarContent = (
    <div
      className={`flex flex-col h-full bg-white border-r border-gray-200/80 text-gray-700 transition-all duration-300 select-none ${
        collapsed ? 'w-20' : 'w-64'
      }`}
    >
      {/* Brand Header */}
      <div className="p-4 flex items-center justify-between border-b border-gray-100">
        {!collapsed && (
          <div
            className="flex items-center space-x-2.5 cursor-pointer"
            onClick={() => handleNavClick('Products')}
          >
            <div className="w-8 h-8 rounded-xl bg-gradient-to-tr from-[#1e1b4b] to-[#312e81] flex items-center justify-center text-[#f59e0b] shadow-xs">
              <ShoppingBag className="w-5 h-5" />
            </div>
            <div className="flex items-center">
              <span className="font-extrabold text-base text-gray-900 tracking-tight font-serif">
                Ajrah Noor
              </span>
              <span className="font-extrabold text-xs text-[#f59e0b] ml-1.5 uppercase font-serif tracking-wider">
                Atelier
              </span>
            </div>
          </div>
        )}

        {collapsed && (
          <div className="mx-auto w-8 h-8 rounded-xl bg-[#1e1b4b] flex items-center justify-center text-[#f59e0b]">
            <ShoppingBag className="w-5 h-5" />
          </div>
        )}

        {/* Collapse Toggle Button */}
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="hidden md:flex w-7 h-7 rounded-full bg-[#f59e0b] hover:bg-[#d97706] text-white items-center justify-center shadow-xs transition-transform"
        >
          {collapsed ? <ChevronsRight className="w-4 h-4" /> : <ChevronsLeft className="w-4 h-4" />}
        </button>
      </div>

      {/* Sidebar Navigation */}
      <div className="flex-1 overflow-y-auto px-3 py-4 space-y-5 text-xs">
        {sidebarGroups.map((group) => {
          const isExpanded = expandedSections[group.title] ?? true;

          return (
            <div key={group.title} className="space-y-1">
              {!collapsed && (
                <div
                  onClick={() => toggleSection(group.title)}
                  className="flex items-center justify-between px-2 py-1 text-[10px] font-bold text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-600"
                >
                  <span>{group.title}</span>
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
                        className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all ${
                          isActive
                            ? 'bg-[#f59e0b]/10 text-[#d97706] border border-[#f59e0b]/30 shadow-2xs'
                            : 'text-gray-600 hover:bg-gray-100/80 hover:text-gray-900'
                        }`}
                        title={collapsed ? item.label : undefined}
                      >
                        <div className="flex items-center space-x-2.5 truncate">
                          <Icon className={`w-4 h-4 shrink-0 ${isActive ? 'text-[#f59e0b]' : 'text-gray-400'}`} />
                          {!collapsed && <span className="truncate">{item.label}</span>}
                        </div>

                        {!collapsed && item.hasSub && (
                          <ChevronRight className="w-3.5 h-3.5 text-gray-300 shrink-0" />
                        )}
                      </button>
                    );
                  })}
              </nav>
            </div>
          );
        })}
      </div>
    </div>
  );

  return (
    <div className="min-h-screen bg-[#f8fafc] text-gray-800 flex font-sans antialiased">
      <AddProductModal isOpen={isAddModalOpen} onClose={() => setIsAddModalOpen(false)} />

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
        {/* Ajrah Noor Top Header */}
        <header className="bg-white border-b border-gray-200/80 px-4 md:px-6 py-2.5 flex items-center justify-between sticky top-0 z-30 shadow-2xs">
          <div className="flex items-center space-x-3">
            <button
              onClick={() => setMobileOpen(!mobileOpen)}
              className="p-1.5 text-gray-600 md:hidden hover:bg-gray-100 rounded-lg"
            >
              <Menu className="w-5 h-5" />
            </button>

            {/* Global Search Input */}
            <div className="relative hidden sm:block w-64 md:w-72">
              <Search className="w-4 h-4 text-gray-400 absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                placeholder="Search products or orders..."
                className="w-full pl-9 pr-12 py-1.5 text-xs bg-gray-50 border border-gray-200 rounded-lg text-gray-800 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-[#f59e0b]/20 focus:border-[#f59e0b] transition-all"
              />
              <span className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[10px] font-semibold text-gray-400 border border-gray-200 rounded px-1.5 py-0.5 bg-white font-mono shadow-2xs">
                ⌘ K
              </span>
            </div>
          </div>

          {/* Right Header Actions */}
          <div className="flex items-center space-x-2 md:space-x-3 text-xs">
            {/* Store Switcher */}
            <div className="hidden lg:flex items-center space-x-1.5 bg-emerald-50 border border-emerald-200 text-emerald-800 px-2.5 py-1 rounded-lg font-semibold text-[11px] cursor-pointer">
              <span className="w-2 h-2 rounded-full bg-emerald-500" />
              <span>Freshmart POS</span>
              <ChevronDown className="w-3 h-3 text-emerald-600" />
            </div>

            {/* + Add New Button */}
            <button
              onClick={() => setIsAddModalOpen(true)}
              className="bg-[#f59e0b] hover:bg-[#d97706] text-white px-3 py-1.5 rounded-lg font-bold flex items-center space-x-1 shadow-2xs transition-all text-xs"
            >
              <PlusCircle className="w-3.5 h-3.5" />
              <span>Add Product</span>
            </button>

            {/* Storefront Navigation */}
            <button
              onClick={() => navigate('/')}
              className="bg-[#1e1b4b] hover:bg-[#0f172a] text-white px-3 py-1.5 rounded-lg font-bold flex items-center space-x-1.5 shadow-2xs transition-all text-xs"
            >
              <Monitor className="w-3.5 h-3.5 text-[#f59e0b]" />
              <span>Storefront</span>
            </button>
          </div>
        </header>

        {/* Main Content Workspace */}
        <main className="flex-1 p-4 md:p-6 lg:p-8 overflow-y-auto">{children}</main>
      </div>
    </div>
  );
};

export default DashboardLayout;
