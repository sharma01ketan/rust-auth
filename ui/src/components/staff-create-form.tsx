"use client"

import { useActionState } from "react"

import { staffCreateAction } from "@/app/actions"
import { SubmitButton } from "@/components/submit-button"
import { Alert, AlertDescription } from "@/components/ui/alert"
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

export function StaffCreateForm() {
  const [state, action] = useActionState(staffCreateAction, { error: null })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Create a task</CardTitle>
        <CardDescription>Staff cannot create tasks.</CardDescription>
      </CardHeader>
      <CardContent>
        <form action={action} className="flex flex-col gap-3">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="staff-title">Title</Label>
            <Input id="staff-title" name="title" />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="staff-description">Description</Label>
            <Input id="staff-description" name="description" />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="staff-priority">Priority</Label>
            <select id="staff-priority" name="priority" defaultValue="low" className={fieldClass}>
              <option value="low">low</option>
              <option value="medium">medium</option>
              <option value="high">high</option>
            </select>
          </div>
          {state.error ? (
            <Alert variant="destructive">
              <AlertDescription>{state.error}</AlertDescription>
            </Alert>
          ) : null}
          <SubmitButton>Create task</SubmitButton>
        </form>
      </CardContent>
    </Card>
  )
}
