'use client';

import { createClient } from '@/utils/supabase/client';
import { Button } from '@/components/ui/button';
import { useState, Suspense } from 'react';
import { useSearchParams } from 'next/navigation';

function LoginContent() {
  const [loading, setLoading] = useState(false);
  const searchParams = useSearchParams();
  const error = searchParams.get('error');

  const handleLogin = async () => {
    setLoading(true);
    const supabase = createClient();
    await supabase.auth.signInWithOAuth({
      provider: 'google',
      options: {
        redirectTo: `${window.location.origin}/auth/callback`,
      },
    });
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-[var(--color-zp-white)]">
      <div className="w-full max-w-md border-4 border-[var(--color-zp-black)] bg-[var(--color-zp-papaya)] p-8 shadow-[8px_8px_0_0_var(--color-zp-black)]">
        <div className="mb-8 text-center">
          <h1 className="text-4xl font-black text-[var(--color-zp-black)] uppercase tracking-tighter">
            Zero Productivity
          </h1>
          <p className="mt-2 font-bold text-[var(--color-zp-gunmetal)]">
            Log in to access your dashboard
          </p>
        </div>

        {error && (
          <div className="mb-6 border-2 border-red-500 bg-red-100 p-4 text-center font-bold text-red-700">
            Authentication failed. Please try again.
          </div>
        )}

        <Button
          onClick={handleLogin}
          disabled={loading}
          className="w-full border-4 border-[var(--color-zp-black)] bg-[var(--color-zp-teal)] py-6 text-xl font-bold text-white shadow-[4px_4px_0_0_var(--color-zp-black)] transition-all hover:translate-x-1 hover:translate-y-1 hover:shadow-none disabled:opacity-50"
        >
          {loading ? 'Connecting...' : 'Continue with Google'}
        </Button>
      </div>
    </div>
  );
}

export default function LoginPage() {
  return (
    <Suspense fallback={<div className="flex min-h-screen items-center justify-center font-bold">Loading...</div>}>
      <LoginContent />
    </Suspense>
  );
}
