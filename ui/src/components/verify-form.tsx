"use client"

import { useActionState } from "react"

import { verifyAction } from "@/app/actions"
import { SubmitButton } from "@/components/submit-button"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export function VerifyForm({ challenge }: { challenge: string }) {
  const [state, action] = useActionState(verifyAction, { error: null })

  return (
    <form action={action} className="flex flex-col gap-3">
      <input type="hidden" name="challenge" value={challenge} />
      <div className="flex flex-col gap-1.5">
        <Label htmlFor="code">Code</Label>
        <Input
          id="code"
          name="code"
          inputMode="numeric"
          autoComplete="one-time-code"
          required
        />
      </div>
      {state.error ? (
        <Alert variant="destructive">
          <AlertDescription>{state.error}</AlertDescription>
        </Alert>
      ) : null}
      <SubmitButton>Verify</SubmitButton>
    </form>
  )
}
