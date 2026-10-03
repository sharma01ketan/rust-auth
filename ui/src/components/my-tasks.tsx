import type { MyTasks } from "@/lib/api"
import { Badge } from "@/components/ui/badge"

export function MyTasksView({ data }: { data: MyTasks }) {
  return (
    <section className="flex flex-col gap-3" aria-labelledby="my-tasks-heading">
      <h2 id="my-tasks-heading" className="text-lg font-semibold">
        My tasks
      </h2>
      <p className="text-sm">
        {data.user.email} · {data.user.role}
      </p>
      <p className="text-sm">{data.summary.total_assigned_tasks} assigned tasks</p>
      <p className="text-sm">{data.cache.hit ? "Cache hit" : "Cache miss"}</p>
      {data.tasks.length === 0 ? (
        <p role="status" className="text-sm text-muted-foreground">
          No tasks assigned.
        </p>
      ) : (
        <ul className="flex flex-col gap-2" role="list">
          {data.tasks.map((task) => (
            <li key={task.id} className="flex flex-wrap items-center gap-2 text-sm">
              <span>{task.title}</span>
              <Badge variant="outline">{task.priority}</Badge>
              <span>{task.status}</span>
              <span className="text-muted-foreground">Assigned to {task.assigned_to}</span>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}
