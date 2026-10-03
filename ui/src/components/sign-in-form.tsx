"use client"

import { useActionState } from "react"

import { loginAction, seedUsersAction } from "@/app/actions"
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

export function SignInForm() {
  const [seed, seedAction] = useActionState(seedUsersAction, {
    users: null,
    error: null,
  })
  const [login, loginFormAction] = useActionState(loginAction, { error: null })

  return (
    <div className="flex flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>Seed users</CardTitle>
          <CardDescription>
            Creates Admin and James Bond when they are missing.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form action={seedAction} className="flex flex-col gap-3">
            <SubmitButton variant="outline">Seed Admin and James Bond</SubmitButton>
            {seed.error ? (
              <Alert variant="destructive">
                <AlertDescription>{seed.error}</AlertDescription>
              </Alert>
            ) : null}
            {seed.users ? (
              <ul className="flex flex-col gap-1 text-sm" role="list">
                {seed.users.map((user) => (
                  <li key={user.email}>
                    {user.full_name} · {user.email} · {user.role}
                  </li>
                ))}
              </ul>
            ) : null}
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Sign in</CardTitle>
          <CardDescription>
            Email and password start a login challenge. A verification code comes next.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form action={loginFormAction} className="flex flex-col gap-3">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="email">Email</Label>
              <Input id="email" name="email" type="email" autoComplete="username" required />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                name="password"
                type="password"
                autoComplete="current-password"
                required
              />
            </div>
            {login.error ? (
              <Alert variant="destructive">
                <AlertDescription>{login.error}</AlertDescription>
              </Alert>
            ) : null}
            <SubmitButton>Sign in</SubmitButton>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
