import { createClient } from '@/utils/supabase/server'
import { EmptyState } from '@/components/ui/empty-state'
import { Device } from '@/lib/types'
import { format } from 'date-fns'
import { Monitor } from 'lucide-react'

export default async function DevicesPage() {
  const supabase = await createClient()

  const { data: devices, error } = await supabase
    .from('devices')
    .select('*')
    .order('created_at', { ascending: false })

  if (error || !devices || devices.length === 0) {
    return <EmptyState title="No devices registered" description="Log into the desktop application to register your first device." />
  }

  const typedDevices = devices as Device[]

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-black uppercase tracking-tighter text-[var(--color-zp-black)]">Devices</h1>
        <p className="font-bold text-[var(--color-zp-gunmetal)]">Manage your connected tracking devices</p>
      </div>

      <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
        {typedDevices.map(device => (
          <div key={device.id} className="flex flex-col justify-between border-4 border-[var(--color-zp-black)] bg-white p-6 shadow-[8px_8px_0_0_var(--color-zp-black)]">
            <div>
              <div className="mb-4 flex items-center gap-3">
                <Monitor className="h-8 w-8 text-[var(--color-zp-teal)]" />
                <h2 className="text-2xl font-black uppercase">{device.device_name}</h2>
              </div>
              <div className="space-y-2 font-bold text-[var(--color-zp-gunmetal)]">
                <p>Platform: <span className="text-[var(--color-zp-black)]">{device.platform}</span></p>
                <p>Version: <span className="text-[var(--color-zp-black)]">{device.app_version || 'Unknown'}</span></p>
                <p>Mode: <span className="text-[var(--color-zp-black)]">{device.settings_mode}</span></p>
              </div>
            </div>
            
            <div className="mt-6 border-t-4 border-[var(--color-zp-black)] pt-4">
              <p className="text-sm font-bold text-[var(--color-zp-gunmetal)]">
                Last Sync: {device.last_sync_at ? format(new Date(device.last_sync_at), 'MMM d, HH:mm') : 'Never'}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
