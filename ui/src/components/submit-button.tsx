"use client"

import { useFormStatus } from "react-dom"

import { Button } from "@/components/ui/button"

export function SubmitButton({
  children,
  disabled,
  variant = "default",
}: {
  children: React.ReactNode
  disabled?: boolean
  variant?: "default" | "outline" | "secondary"
}) {
  const { pending } = useFormStatus()
  return (
    <Button type="submit" variant={variant} disabled={pending || disabled}>
      {pending ? "Working…" : children}
    </Button>
  )
}
