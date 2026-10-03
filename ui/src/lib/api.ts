export type PublicUser = {
  id: string
  full_name: string
  email: string
  role: string
}

export type Task = {
  id: string
  title: string
  description: string
  status: string
  priority: string
  assigned_to: string | null
}

export type MyTask = {
  id: string
  title: string
  status: string
  priority: string
  assigned_to: string
}

export type MyTasks = {
  user: { email: string; role: string }
  tasks: MyTask[]
  summary: { total_assigned_tasks: number }
  cache: { hit: boolean }
}

export type EmailLog = {
  to: string
  code: string
  created_at: string
}

export class ApiFailure extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message)
    this.name = "ApiFailure"
  }
}

function apiUrl(path: string) {
  const base = process.env.TASK_API_URL ?? "http://127.0.0.1:3000"
  return `${base.replace(/\/$/, "")}${path}`
}

async function call<T>(
  path: string,
  init: RequestInit & { token?: string } = {},
): Promise<T> {
  const { token, ...request } = init
  const headers = new Headers(request.headers)
  if (request.body) headers.set("content-type", "application/json")
  if (token) headers.set("authorization", `Bearer ${token}`)
  let response: Response
  try {
    response = await fetch(apiUrl(path), {
      ...request,
      headers,
      cache: "no-store",
    })
  } catch {
    throw new ApiFailure(0, "The API request failed.")
  }
  const text = await response.text()
  const data = text ? (JSON.parse(text) as { error?: string }) : null
  if (!response.ok) {
    throw new ApiFailure(response.status, data?.error ?? "The API request failed.")
  }
  return data as T
}

export function seedUsers() {
  return call<{ users: PublicUser[] }>("/seed/users", {
    method: "POST",
    body: "{}",
  })
}

export function login(email: string, password: string) {
  return call<{ login_challenge_id: string }>("/auth/login", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  })
}

export function latestEmail() {
  return call<EmailLog>("/dev/email-logs/latest")
}

export function verifyCode(loginChallengeId: string, code: string) {
  return call<{ access_token: string }>("/auth/verify-2fa", {
    method: "POST",
    body: JSON.stringify({ login_challenge_id: loginChallengeId, code }),
  })
}

export function createTask(
  token: string,
  body: { title: string; description: string; priority: string },
) {
  return call<Task>("/tasks", {
    method: "POST",
    token,
    body: JSON.stringify(body),
  })
}

export function assignTasks(
  token: string,
  taskIds: string[],
  assigneeEmail: string,
) {
  return call<{ assigned: number }>("/tasks/assign", {
    method: "POST",
    token,
    body: JSON.stringify({ task_ids: taskIds, assignee_email: assigneeEmail }),
  })
}

export function viewMyTasks(token: string) {
  return call<MyTasks>("/tasks/view-my-tasks", { token })
}
