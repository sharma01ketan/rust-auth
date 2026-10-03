import { Skeleton } from "@/components/ui/skeleton"

export default function Loading() {
  return (
    <div aria-busy="true" aria-label="Loading tasks" className="flex flex-col gap-3">
      <Skeleton className="h-8 w-40" />
      <Skeleton className="h-24 w-full" />
      <Skeleton className="h-24 w-full" />
    </div>
  )
}
