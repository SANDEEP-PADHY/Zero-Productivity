import { Loader2 } from "lucide-react"

export function LoadingState() {
  return (
    <div className="flex h-[400px] w-full flex-col items-center justify-center border-4 border-[var(--color-zp-black)] bg-white shadow-[8px_8px_0_0_var(--color-zp-black)]">
      <Loader2 className="h-12 w-12 animate-spin text-[var(--color-zp-teal)]" />
      <p className="mt-4 font-bold text-[var(--color-zp-gunmetal)]">Loading data...</p>
    </div>
  )
}
