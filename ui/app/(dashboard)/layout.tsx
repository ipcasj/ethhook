'use client';

import { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Button } from '@/components/ui/button';
import { clearAuthToken, getAuthToken, api } from '@/lib/api-client';
import { User } from '@/lib/types';
import { 
  LayoutDashboard, 
  Box, 
  Webhook, 
  Activity, 
  Settings, 
  LogOut, 
  Menu, 
  X,
  User as UserIcon
} from 'lucide-react';
import { toast } from 'sonner';

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Applications', href: '/dashboard/applications', icon: Box },
  { name: 'Endpoints', href: '/dashboard/endpoints', icon: Webhook },
  { name: 'Events', href: '/dashboard/events', icon: Activity },
  { name: 'Settings', href: '/dashboard/settings', icon: Settings },
];

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const pathname = usePathname();
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);

  // Fetch user profile
  const { data: user } = useQuery<User>({
    queryKey: ['user-profile'],
    queryFn: () => api.get<User>('/users/me'),
    retry: false,
  });

  useEffect(() => {
    const token = getAuthToken();
    if (!token) {
      router.push('/login');
    }
  }, [router]);

  const handleLogout = () => {
    clearAuthToken();
    toast.success('Logged out successfully');
    router.push('/login');
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50">
      {/* Mobile sidebar backdrop */}
      {isSidebarOpen && (
        <div
          className="fixed inset-0 bg-indigo-900/20 backdrop-blur-sm z-40 lg:hidden"
          onClick={() => setIsSidebarOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside
        className={`
          fixed top-0 left-0 z-50 h-screen w-64 bg-white/80 backdrop-blur-xl border-r border-indigo-100
          transform transition-transform duration-200 ease-in-out lg:translate-x-0 shadow-xl
          ${isSidebarOpen ? 'translate-x-0' : '-translate-x-full'}
        `}
      >
        <div className="flex flex-col h-full">
          {/* Logo */}
          <div className="flex items-center justify-between h-16 px-6 border-b border-indigo-100">
            <h1 className="text-xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
              EthHook
            </h1>
            <button
              onClick={() => setIsSidebarOpen(false)}
              className="lg:hidden text-slate-500 hover:text-indigo-600 transition-colors"
            >
              <X className="w-6 h-6" />
            </button>
          </div>

          {/* Navigation */}
          <nav className="flex-1 px-4 py-6 space-y-1 overflow-y-auto">
            {navigation.map((item) => {
              const isActive = pathname === item.href;
              return (
                <Link
                  key={item.name}
                  href={item.href}
                  className={`
                    flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all
                    ${
                      isActive
                        ? 'bg-gradient-to-r from-blue-600 to-indigo-600 text-white shadow-lg shadow-indigo-500/30'
                        : 'text-slate-700 hover:bg-indigo-50 hover:text-indigo-700'
                    }
                  `}
                  onClick={() => setIsSidebarOpen(false)}
                >
                  <item.icon className="w-5 h-5" />
                  {item.name}
                </Link>
              );
            })}
          </nav>

          {/* User info & Logout */}
          <div className="p-4 border-t border-indigo-100 space-y-3">
            {/* User profile section */}
            {user && (
              <div className="flex items-center gap-3 px-3 py-2 bg-indigo-50 rounded-lg">
                <div className="w-10 h-10 rounded-full bg-gradient-to-r from-blue-600 to-indigo-600 flex items-center justify-center text-white font-semibold">
                  {user.name.charAt(0).toUpperCase()}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-slate-900 truncate">
                    {user.name}
                  </p>
                  <p className="text-xs text-slate-600 truncate">
                    {user.email}
                  </p>
                </div>
              </div>
            )}

            {/* Logout button */}
            <Button
              variant="ghost"
              className="w-full justify-start gap-3 text-rose-600 hover:text-rose-700 hover:bg-rose-50 transition-colors"
              onClick={handleLogout}
            >
              <LogOut className="w-5 h-5" />
              Logout
            </Button>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <div className="lg:pl-64">
        {/* Top header */}
        <header className="sticky top-0 z-30 h-16 bg-white/80 backdrop-blur-xl border-b border-indigo-100 shadow-sm">
          <div className="flex items-center justify-between h-full px-6">
            <button
              onClick={() => setIsSidebarOpen(true)}
              className="lg:hidden text-slate-500 hover:text-indigo-600 transition-colors"
            >
              <Menu className="w-6 h-6" />
            </button>
            <div className="flex-1 lg:flex-none"></div>
          </div>
        </header>

        {/* Page content */}
        <main className="p-6">
          {children}
        </main>
      </div>
    </div>
  );
}
