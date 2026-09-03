import { AlertCircle } from "lucide-react"
import { Button } from "./button"

interface ErrorStateProps {
  message?: string;
  retry?: () => void;
}

export function ErrorState({ message = "Something went wrong loading this data.", retry }: ErrorStateProps) {
  return (
    <div className="flex h-[400px] w-full flex-col items-center justify-center border-4 border-[var(--color-zp-black)] bg-red-50 p-6 text-center shadow-[8px_8px_0_0_var(--color-zp-black)]">
      <AlertCircle className="mb-4 h-12 w-12 text-red-600" />
      <h3 className="mb-2 text-xl font-black uppercase text-[var(--color-zp-black)]">Error Loading Data</h3>
      <p className="mb-6 font-bold text-red-800">{message}</p>
      {retry && (
        <Button onClick={retry} className="border-4 border-[var(--color-zp-black)] bg-red-600 text-white hover:bg-red-700">
          Try Again
        </Button>
      )}
    </div>
  )
}
