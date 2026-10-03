import { redirect } from "next/navigation"

import { ApiFailure, viewMyTasks } from "@/lib/api"
import { getSession } from "@/lib/session"
import { AdminBoard } from "@/components/admin-board"
import { MyTasksView } from "@/components/my-tasks"

export const dynamic = "force-dynamic"

export default async function AdminPage() {
  const session = await getSession()
  if (!session) redirect("/")
  if (session.role !== "admin") redirect("/staff")

  let mine
  try {
    mine = await viewMyTasks(session.token)
  } catch (error) {
    if (error instanceof ApiFailure && error.status === 401) redirect("/")
    throw error
  }

  return (
    <>
      <h1 className="text-2xl font-semibold tracking-tight">Create tasks</h1>
      <AdminBoard />
      <MyTasksView data={mine} />
    </>
  )
}
