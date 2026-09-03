import { createClient } from '@/utils/supabase/server'
import { EmptyState } from '@/components/ui/empty-state'
import { ActivitySession } from '@/lib/types'
import { format } from 'date-fns'

export default async function ActivityPage() {
  const supabase = await createClient()

  const { data: sessions, error } = await supabase
    .from('activity_sessions')
    .select('*')
    .order('started_at', { ascending: false })
    .limit(50)

  if (error || !sessions || sessions.length === 0) {
    return <EmptyState title="No activity found" description="There is no activity history available yet." />
  }

  const typedSessions = sessions as ActivitySession[]

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-black uppercase tracking-tighter text-[var(--color-zp-black)]">Activity Timeline</h1>
        <p className="font-bold text-[var(--color-zp-gunmetal)]">Your most recent 50 tracked sessions</p>
      </div>

      <div className="border-4 border-[var(--color-zp-black)] bg-white shadow-[8px_8px_0_0_var(--color-zp-black)]">
        <table className="w-full text-left">
          <thead className="bg-[var(--color-zp-gunmetal)] text-white">
            <tr>
              <th className="p-4 font-black uppercase">Time</th>
              <th className="p-4 font-black uppercase">Application</th>
              <th className="p-4 font-black uppercase">Duration</th>
              <th className="p-4 font-black uppercase">Type</th>
            </tr>
          </thead>
          <tbody>
            {typedSessions.map((session, i) => (
              <tr key={session.id} className={i % 2 === 0 ? 'bg-white' : 'bg-[var(--color-zp-papaya)]'}>
                <td className="border-t-4 border-[var(--color-zp-black)] p-4 font-bold">
                  {format(new Date(session.started_at), 'MMM d, HH:mm')}
                </td>
                <td className="border-t-4 border-[var(--color-zp-black)] p-4 font-bold">
                  {session.title || session.application_name || session.domain || 'Unknown'}
                </td>
                <td className="border-t-4 border-[var(--color-zp-black)] p-4 font-bold text-[var(--color-zp-teal)]">
                  {Math.round(session.duration_ms / 1000)}s
                </td>
                <td className="border-t-4 border-[var(--color-zp-black)] p-4 font-bold">
                  {session.classification || 'Neutral'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
