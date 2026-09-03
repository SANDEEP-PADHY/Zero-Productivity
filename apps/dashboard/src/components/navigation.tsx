'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { cn } from '@/components/ui/button';
import { LayoutDashboard, Activity, BarChart, Monitor, Settings, LogOut } from 'lucide-react';
import { createClient } from '@/utils/supabase/client';
import { useRouter } from 'next/navigation';

const navItems = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Activity', href: '/activity', icon: Activity },
  { name: 'Analytics', href: '/analytics', icon: BarChart },
  { name: 'Devices', href: '/devices', icon: Monitor },
  { name: 'Settings', href: '/settings', icon: Settings },
];

export function Navigation() {
  const pathname = usePathname();
  const router = useRouter();

  // Don't render navigation on the login page
  if (pathname === '/login') return null;

  const handleLogout = async () => {
    const supabase = createClient();
    await supabase.auth.signOut();
    router.push('/login');
  };

  return (
    <nav className="fixed inset-y-0 left-0 w-64 border-r-4 border-[var(--color-zp-black)] bg-[var(--color-zp-papaya)] p-6 z-10 flex flex-col">
      <div className="mb-12">
        <h1 className="text-2xl font-black uppercase tracking-tighter text-[var(--color-zp-black)] leading-tight">
          Zero <br/> Productivity
        </h1>
      </div>
      
      <ul className="space-y-4 flex-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = pathname === item.href || (pathname === '/' && item.href === '/dashboard');
          
          return (
            <li key={item.name}>
              <Link
                href={item.href}
                className={cn(
                  "flex items-center gap-3 border-4 p-3 font-bold transition-all shadow-[4px_4px_0_0_var(--color-zp-black)] hover:translate-x-1 hover:translate-y-1 hover:shadow-none",
                  isActive 
                    ? "bg-[var(--color-zp-teal)] text-white border-[var(--color-zp-black)]" 
                    : "bg-white border-[var(--color-zp-black)] text-[var(--color-zp-gunmetal)]"
                )}
              >
                <Icon size={20} />
                {item.name}
              </Link>
            </li>
          );
        })}
      </ul>

      <div className="mt-auto">
        <button
          onClick={handleLogout}
          className="flex w-full items-center justify-center gap-2 border-4 border-[var(--color-zp-black)] bg-[var(--color-zp-gunmetal)] p-3 font-bold text-white shadow-[4px_4px_0_0_var(--color-zp-black)] transition-all hover:translate-x-1 hover:translate-y-1 hover:shadow-none"
        >
          <LogOut size={20} />
          Sign Out
        </button>
      </div>
    </nav>
  );
}
