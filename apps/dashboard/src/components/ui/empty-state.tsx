import { LayoutDashboard } from "lucide-react"

interface EmptyStateProps {
  title: string;
  description: string;
}

export function EmptyState({ title, description }: EmptyStateProps) {
  return (
    <div className="flex h-[400px] w-full flex-col items-center justify-center border-4 border-[var(--color-zp-black)] bg-white p-6 text-center shadow-[8px_8px_0_0_var(--color-zp-black)]">
      <LayoutDashboard className="mb-4 h-16 w-16 text-gray-300" />
      <h3 className="mb-2 text-2xl font-black uppercase text-[var(--color-zp-black)]">{title}</h3>
      <p className="max-w-md font-bold text-[var(--color-zp-gunmetal)]">{description}</p>
    </div>
  )
}
