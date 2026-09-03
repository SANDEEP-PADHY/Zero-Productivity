import { createClient } from '@/utils/supabase/server'
import { Setting } from '@/lib/types'

export default async function SettingsPage() {
  const supabase = await createClient()

  // Fetch account-level settings (device_id is null)
  const { data: settings, error } = await supabase
    .from('settings')
    .select('*')
    .is('device_id', null)
    .single()

  const hasSettings = !error && settings;
  const typedSettings = settings as Setting;

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-black uppercase tracking-tighter text-[var(--color-zp-black)]">Settings</h1>
        <p className="font-bold text-[var(--color-zp-gunmetal)]">Account-wide defaults and tracking rules</p>
      </div>

      <div className="border-4 border-[var(--color-zp-black)] bg-white p-6 shadow-[8px_8px_0_0_var(--color-zp-black)]">
        <h2 className="mb-4 text-2xl font-black uppercase">Account Configuration</h2>
        
        {hasSettings ? (
          <div className="font-bold text-[var(--color-zp-gunmetal)]">
            <p className="mb-2">Settings Version: <span className="text-[var(--color-zp-black)]">{typedSettings.version}</span></p>
            <div className="mt-4 bg-[var(--color-zp-papaya)] p-4 border-2 border-[var(--color-zp-black)]">
              <pre className="whitespace-pre-wrap font-mono text-sm text-[var(--color-zp-black)]">
                {JSON.stringify(typedSettings.payload, null, 2)}
              </pre>
            </div>
          </div>
        ) : (
          <div className="bg-[var(--color-zp-papaya)] p-6 border-4 border-[var(--color-zp-black)] text-center">
            <p className="font-bold text-[var(--color-zp-gunmetal)]">No account-wide settings have been synced yet. Configure them from your desktop client.</p>
          </div>
        )}
      </div>
    </div>
  )
}
