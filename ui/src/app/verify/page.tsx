import { ApiFailure, latestEmail } from "@/lib/api"
import { VerifyForm } from "@/components/verify-form"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export default async function VerifyPage({
  searchParams,
}: {
  searchParams: Promise<{ challenge?: string }>
}) {
  const { challenge } = await searchParams
  if (!challenge) {
    return (
      <>
        <h1 className="text-2xl font-semibold tracking-tight">Verification code</h1>
        <p className="text-sm text-muted-foreground">
          Start sign-in before entering a verification code.
        </p>
      </>
    )
  }

  let log: { to: string; code: string; created_at: string } | null = null
  let empty = false
  try {
    log = await latestEmail()
  } catch (error) {
    if (error instanceof ApiFailure && error.status === 404) empty = true
    else throw error
  }

  return (
    <>
      <h1 className="text-2xl font-semibold tracking-tight">Verification code</h1>
      <Card>
        <CardHeader>
          <CardTitle>Email log</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {empty || !log ? (
            <p role="status" className="text-sm text-muted-foreground">
              No verification code has been issued.
            </p>
          ) : (
            <p className="text-sm">
              Latest code for {log.to} at {log.created_at}:{" "}
              <strong>{log.code}</strong>
            </p>
          )}
          <VerifyForm challenge={challenge} />
        </CardContent>
      </Card>
    </>
  )
}
