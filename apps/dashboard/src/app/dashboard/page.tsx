import { createClient } from '@/utils/supabase/server'
import { EmptyState } from '@/components/ui/empty-state'
import { ActivitySession } from '@/lib/types'

export default async function DashboardPage() {
  const supabase = await createClient()

  // Fetch today's activity sessions
  const startOfDay = new Date()
  startOfDay.setHours(0, 0, 0, 0)
  
  const { data: sessions, error } = await supabase
    .from('activity_sessions')
    .select('*')
    .gte('started_at', startOfDay.toISOString())
    .order('duration_ms', { ascending: false })

  if (error || !sessions || sessions.length === 0) {
    return <EmptyState title="No activity today" description="We haven't tracked any activity for today yet. Make sure your desktop client is running." />
  }

  const typedSessions = sessions as ActivitySession[]
  const totalDuration = typedSessions.reduce((acc, curr) => acc + curr.duration_ms, 0)
  const foregroundDuration = typedSessions.reduce((acc, curr) => acc + (curr.foreground_ms || 0), 0)

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-black uppercase tracking-tighter text-[var(--color-zp-black)]">Dashboard</h1>
        <p className="font-bold text-[var(--color-zp-gunmetal)]">Overview of today&apos;s productivity</p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <div className="border-4 border-[var(--color-zp-black)] bg-[var(--color-zp-papaya)] p-6 shadow-[8px_8px_0_0_var(--color-zp-black)]">
          <h2 className="text-xl font-black uppercase">Total Tracked Time</h2>
          <p className="mt-2 text-5xl font-black text-[var(--color-zp-teal)]">
            {Math.round(totalDuration / 1000 / 60)} <span className="text-2xl text-[var(--color-zp-black)]">min</span>
          </p>
        </div>
        
        <div className="border-4 border-[var(--color-zp-black)] bg-white p-6 shadow-[8px_8px_0_0_var(--color-zp-black)]">
          <h2 className="text-xl font-black uppercase">Foreground Time</h2>
          <p className="mt-2 text-5xl font-black text-[var(--color-zp-teal)]">
            {Math.round(foregroundDuration / 1000 / 60)} <span className="text-2xl text-[var(--color-zp-black)]">min</span>
          </p>
        </div>
      </div>

      <div>
        <h2 className="mb-4 text-2xl font-black uppercase tracking-tight">Top Applications</h2>
        <div className="border-4 border-[var(--color-zp-black)] bg-white shadow-[8px_8px_0_0_var(--color-zp-black)]">
          {typedSessions.slice(0, 5).map((session, i) => (
            <div key={session.id} className={`flex justify-between p-4 font-bold ${i !== 0 ? 'border-t-4 border-[var(--color-zp-black)]' : ''}`}>
              <span>{session.application_name || 'Unknown'}</span>
              <span className="text-[var(--color-zp-teal)]">{Math.round(session.duration_ms / 1000 / 60)} min</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
