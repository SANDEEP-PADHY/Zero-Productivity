import { createClient } from '@/utils/supabase/server'
import { EmptyState } from '@/components/ui/empty-state'
import { ActivitySession } from '@/lib/types'

export default async function AnalyticsPage() {
  const supabase = await createClient()

  // Fetch last 7 days of activity
  const startOfWeek = new Date()
  startOfWeek.setDate(startOfWeek.getDate() - 7)
  
  const { data: sessions, error } = await supabase
    .from('activity_sessions')
    .select('*')
    .gte('started_at', startOfWeek.toISOString())

  if (error || !sessions || sessions.length === 0) {
    return <EmptyState title="No analytics data" description="Keep the tracker running for a few days to see week-over-week analytics." />
  }

  const typedSessions = sessions as ActivitySession[]
  const totalDuration = typedSessions.reduce((acc, curr) => acc + curr.duration_ms, 0)
  
  const classificationBreakdown = typedSessions.reduce((acc, curr) => {
    const cls = curr.classification || 'Neutral'
    acc[cls] = (acc[cls] || 0) + curr.duration_ms
    return acc
  }, {} as Record<string, number>)

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-black uppercase tracking-tighter text-[var(--color-zp-black)]">Analytics</h1>
        <p className="font-bold text-[var(--color-zp-gunmetal)]">Your productivity trends over the last 7 days</p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <div className="border-4 border-[var(--color-zp-black)] bg-[var(--color-zp-papaya)] p-6 shadow-[8px_8px_0_0_var(--color-zp-black)]">
          <h2 className="text-xl font-black uppercase">Weekly Total</h2>
          <p className="mt-2 text-5xl font-black text-[var(--color-zp-teal)]">
            {Math.round(totalDuration / 1000 / 60 / 60)} <span className="text-2xl text-[var(--color-zp-black)]">hrs</span>
          </p>
        </div>
        
        <div className="border-4 border-[var(--color-zp-black)] bg-white p-6 shadow-[8px_8px_0_0_var(--color-zp-black)]">
          <h2 className="mb-4 text-xl font-black uppercase">Classification</h2>
          <div className="space-y-4">
            {Object.entries(classificationBreakdown).map(([cls, duration]) => (
              <div key={cls} className="flex justify-between font-bold">
                <span>{cls}</span>
                <span className="text-[var(--color-zp-teal)]">{Math.round(duration / 1000 / 60)} min</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
