import { redirect } from "next/navigation"

import { reloadMyTasksAction } from "@/app/actions"
import { ApiFailure, viewMyTasks } from "@/lib/api"
import { getSession } from "@/lib/session"
import { MyTasksView } from "@/components/my-tasks"
import { StaffCreateForm } from "@/components/staff-create-form"
import { SubmitButton } from "@/components/submit-button"

export const dynamic = "force-dynamic"

export default async function StaffPage() {
  const session = await getSession()
  if (!session) redirect("/")
  if (session.role !== "staff") redirect("/admin")

  let mine
  try {
    mine = await viewMyTasks(session.token)
  } catch (error) {
    if (error instanceof ApiFailure && error.status === 401) redirect("/")
    throw error
  }

  return (
    <>
      <h1 className="text-2xl font-semibold tracking-tight">My tasks</h1>
      <MyTasksView data={mine} />
      <form action={reloadMyTasksAction}>
        <SubmitButton variant="outline">Load my tasks again</SubmitButton>
      </form>
      <StaffCreateForm />
    </>
  )
}
