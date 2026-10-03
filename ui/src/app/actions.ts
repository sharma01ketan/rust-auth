"use server"

import { refresh } from "next/cache"
import { redirect } from "next/navigation"

import {
  ApiFailure,
  assignTasks,
  createTask,
  login,
  seedUsers,
  verifyCode,
  type PublicUser,
  type Task,
} from "@/lib/api"
import { clearSession, getSession, readClaims, setSession } from "@/lib/session"

export type SeedState = { users: PublicUser[] | null; error: string | null }
export type LoginState = { error: string | null }
export type VerifyState = { error: string | null }
export type CreateState = {
  tasks: Task[]
  message: string | null
  error: string | null
}
export type AssignState = { message: string | null; error: string | null }
export type StaffCreateState = { error: string | null }

export async function seedUsersAction(): Promise<SeedState> {
  try {
    const result = await seedUsers()
    return { users: result.users, error: null }
  } catch {
    return { users: null, error: "Could not seed users." }
  }
}

export async function loginAction(
  _prev: LoginState,
  formData: FormData,
): Promise<LoginState> {
  const email = String(formData.get("email") ?? "")
  const password = String(formData.get("password") ?? "")
  let challengeId: string
  try {
    const result = await login(email, password)
    challengeId = result.login_challenge_id
  } catch (error) {
    if (error instanceof ApiFailure && error.status === 401) {
      return { error: "Email or password is wrong." }
    }
    return { error: "The API request failed." }
  }
  redirect(`/verify?challenge=${encodeURIComponent(challengeId)}`)
}

export async function verifyAction(
  _prev: VerifyState,
  formData: FormData,
): Promise<VerifyState> {
  const challengeId = String(formData.get("challenge") ?? "")
  const code = String(formData.get("code") ?? "").trim()
  let token: string
  try {
    const result = await verifyCode(challengeId, code)
    token = result.access_token
  } catch (error) {
    if (error instanceof ApiFailure && error.status === 401) {
      return { error: "That verification code was rejected." }
    }
    return { error: "The API request failed." }
  }
  const claims = readClaims(token)
  if (!claims) return { error: "The API request failed." }
  await setSession(token)
  redirect(claims.role === "admin" ? "/admin" : "/staff")
}

export async function signOutAction() {
  await clearSession()
  redirect("/")
}

export async function createTaskAction(
  prev: CreateState,
  formData: FormData,
): Promise<CreateState> {
  const session = await getSession()
  if (!session) redirect("/")
  const title = String(formData.get("title") ?? "")
  const description = String(formData.get("description") ?? "")
  const priority = String(formData.get("priority") ?? "")
  try {
    const task = await createTask(session.token, { title, description, priority })
    return {
      tasks: [...prev.tasks, task],
      message: `Created ${task.title}.`,
      error: null,
    }
  } catch (error) {
    const message =
      error instanceof ApiFailure && error.status === 400
        ? error.message
        : "The API request failed."
    return { tasks: prev.tasks, message: null, error: message }
  }
}

export async function assignTasksAction(
  _prev: AssignState,
  formData: FormData,
): Promise<AssignState> {
  const session = await getSession()
  if (!session) redirect("/")
  const taskIds = formData.getAll("task_id").map(String).filter(Boolean)
  if (taskIds.length === 0) {
    return { message: null, error: "Select at least one task." }
  }
  const assigneeEmail = String(formData.get("assignee_email") ?? "")
  try {
    const result = await assignTasks(session.token, taskIds, assigneeEmail)
    return { message: `Assigned ${result.assigned} tasks.`, error: null }
  } catch (error) {
    if (error instanceof ApiFailure && error.status === 404) {
      return { message: null, error: "That assignee or task was not found." }
    }
    if (error instanceof ApiFailure && error.status === 400) {
      return { message: null, error: error.message }
    }
    return { message: null, error: "The API request failed." }
  }
}

export async function staffCreateAction(
  _prev: StaffCreateState,
  formData: FormData,
): Promise<StaffCreateState> {
  const session = await getSession()
  if (!session) redirect("/")
  try {
    await createTask(session.token, {
      title: String(formData.get("title") ?? ""),
      description: String(formData.get("description") ?? ""),
      priority: String(formData.get("priority") ?? ""),
    })
    return { error: null }
  } catch (error) {
    if (error instanceof ApiFailure && error.status === 403) {
      return { error: "You cannot create tasks. Only an admin can." }
    }
    if (error instanceof ApiFailure && error.status === 400) {
      return { error: error.message }
    }
    return { error: "The API request failed." }
  }
}

export async function reloadMyTasksAction() {
  refresh()
}
