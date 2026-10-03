import { cookies } from "next/headers"

const COOKIE = "access_token"

export type Session = {
  token: string
  email: string
  role: "admin" | "staff"
}

export function readClaims(token: string): Omit<Session, "token"> | null {
  const segment = token.split(".")[1]
  if (!segment) return null
  try {
    const claims = JSON.parse(
      Buffer.from(segment, "base64url").toString("utf8"),
    ) as { email?: unknown; role?: unknown; exp?: unknown }
    if (claims.role !== "admin" && claims.role !== "staff") return null
    if (typeof claims.email !== "string") return null
    if (typeof claims.exp === "number" && claims.exp * 1000 <= Date.now()) {
      return null
    }
    return { email: claims.email, role: claims.role }
  } catch {
    return null
  }
}

export async function getSession(): Promise<Session | null> {
  const jar = await cookies()
  const token = jar.get(COOKIE)?.value
  if (!token) return null
  const claims = readClaims(token)
  if (!claims) return null
  return { token, ...claims }
}

export async function setSession(token: string) {
  const jar = await cookies()
  jar.set(COOKIE, token, {
    httpOnly: true,
    sameSite: "lax",
    path: "/",
    maxAge: 60 * 60 * 8,
  })
}

export async function clearSession() {
  const jar = await cookies()
  jar.delete(COOKIE)
}
