import type { Metadata } from "next"
import { Geist, Geist_Mono } from "next/font/google"

import { signOutAction } from "@/app/actions"
import { SubmitButton } from "@/components/submit-button"
import { getSession } from "@/lib/session"
import "./globals.css"

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
})

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
})

export const metadata: Metadata = {
  title: "Task UI",
  description: "Sign in, assign tasks, and view the tasks assigned to you.",
}

export default async function RootLayout({ children }: LayoutProps<"/">) {
  const session = await getSession()

  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col">
        <header className="border-b">
          <div className="mx-auto flex w-full max-w-xl items-center justify-between gap-3 px-4 py-3">
            <p className="text-sm font-medium">Task UI</p>
            {session ? (
              <div className="flex items-center gap-3">
                <p className="text-sm text-muted-foreground">
                  {session.email} · {session.role}
                </p>
                <form action={signOutAction}>
                  <SubmitButton variant="outline">Sign out</SubmitButton>
                </form>
              </div>
            ) : null}
          </div>
        </header>
        <main className="mx-auto flex w-full max-w-xl flex-1 flex-col gap-6 px-4 py-8">
          {children}
        </main>
      </body>
    </html>
  )
}
