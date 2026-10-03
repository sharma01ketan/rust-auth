"use client"

import { useActionState, useState } from "react"

import { assignTasksAction, createTaskAction } from "@/app/actions"
import { SubmitButton } from "@/components/submit-button"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

const fieldClass =
  "h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"

export function AdminBoard() {
  const [created, createAction] = useActionState(createTaskAction, {
    tasks: [],
    message: null,
    error: null,
  })
  const [assigned, assignAction] = useActionState(assignTasksAction, {
    message: null,
    error: null,
  })
  const [selected, setSelected] = useState<string[]>([])

  return (
    <div className="flex flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>Create a task</CardTitle>
          <CardDescription>New tasks start as todo with no assignee.</CardDescription>
        </CardHeader>
        <CardContent>
          <form action={createAction} className="flex flex-col gap-3">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="title">Title</Label>
              <Input id="title" name="title" />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="description">Description</Label>
              <Input id="description" name="description" />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="priority">Priority</Label>
              <select id="priority" name="priority" defaultValue="medium" className={fieldClass}>
                <option value="low">low</option>
                <option value="medium">medium</option>
                <option value="high">high</option>
              </select>
            </div>
            {created.error ? (
              <Alert variant="destructive">
                <AlertDescription>{created.error}</AlertDescription>
              </Alert>
            ) : null}
            {created.message ? (
              <Alert role="status">
                <AlertDescription>{created.message}</AlertDescription>
              </Alert>
            ) : null}
            <SubmitButton>Create task</SubmitButton>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Tasks created this visit</CardTitle>
          <CardDescription>Select the tasks to assign. This list clears on refresh.</CardDescription>
        </CardHeader>
        <CardContent>
          {created.tasks.length === 0 ? (
            <p role="status" className="text-sm text-muted-foreground">
              No tasks created yet.
            </p>
          ) : (
            <form action={assignAction} className="flex flex-col gap-3">
              <ul className="flex flex-col gap-2" role="list">
                {created.tasks.map((task) => (
                  <li key={task.id}>
                    <label className="flex items-center gap-3 text-sm">
                      <input
                        type="checkbox"
                        name="task_id"
                        value={task.id}
                        checked={selected.includes(task.id)}
                        onChange={(event) => {
                          setSelected((current) =>
                            event.target.checked
                              ? [...current, task.id]
                              : current.filter((id) => id !== task.id),
                          )
                        }}
                        className="size-4 accent-primary"
                      />
                      <span>{task.title}</span>
                      <Badge variant="outline">{task.priority}</Badge>
                      <span className="text-muted-foreground">{task.status}</span>
                    </label>
                  </li>
                ))}
              </ul>
              <div className="flex flex-col gap-1.5">
                <Label htmlFor="assignee">Assignee email</Label>
                <Input
                  id="assignee"
                  name="assignee_email"
                  type="email"
                  defaultValue="jamesbond@example.com"
                />
              </div>
              {assigned.error ? (
                <Alert variant="destructive">
                  <AlertDescription>{assigned.error}</AlertDescription>
                </Alert>
              ) : null}
              {assigned.message ? (
                <Alert role="status">
                  <AlertDescription>{assigned.message}</AlertDescription>
                </Alert>
              ) : null}
              <SubmitButton disabled={selected.length === 0}>Assign selected</SubmitButton>
            </form>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
