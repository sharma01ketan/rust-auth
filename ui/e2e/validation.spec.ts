import { expect, test, type Page } from "@playwright/test"

const tasks = [
  ["Alpha", "Work item", "high"],
  ["Bravo", "Work item", "medium"],
  ["Charlie", "Work item", "low"],
  ["Delta", "Work item", "high"],
  ["Echo", "Work item", "low"],
] as const

async function signIn(page: Page, email: string, password: string) {
  await page.goto("/")
  await page.getByLabel("Email").fill(email)
  await page.getByLabel("Password").fill(password)
  await page.getByRole("button", { name: "Sign in" }).click()
  await expect(page.getByRole("heading", { name: "Verification code" })).toBeVisible()
  const readout = await page.getByText(/Latest code for/).innerText()
  const code = readout.match(/\b(\d{6})\b/)?.[1]
  expect(code, readout).toBeTruthy()
  await page.getByRole("textbox", { name: "Code" }).fill(code!)
  await page.getByRole("button", { name: "Verify" }).click()
}

test("reviewer completes the validation flow in the browser", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: "Seed Admin and James Bond" }).click()
  await expect(page.getByRole("listitem").filter({ hasText: "admin@example.com" })).toContainText(
    "admin",
  )
  await expect(
    page.getByRole("listitem").filter({ hasText: "jamesbond@example.com" }),
  ).toContainText("staff")
  await page.getByRole("button", { name: "Seed Admin and James Bond" }).click()
  await expect(
    page.getByRole("listitem").filter({ hasText: "jamesbond@example.com" }),
  ).toBeVisible()

  await page.getByLabel("Email").fill("admin@example.com")
  await page.getByLabel("Password").fill("wrong-password")
  await page.getByRole("button", { name: "Sign in" }).click()
  await expect(page.getByText("Email or password is wrong.")).toBeVisible()
  await expect(page.getByRole("heading", { name: "Verification code" })).toHaveCount(0)

  await signIn(page, "admin@example.com", "admin-password")
  await expect(page.getByRole("heading", { name: "Create tasks" })).toBeVisible()
  await expect(page.getByText("No tasks created yet.")).toBeVisible()
  await expect(page.getByText("No tasks assigned.")).toBeVisible()

  for (const [title, description, priority] of tasks) {
    await page.getByLabel("Title").fill(title)
    await page.getByLabel("Description").fill(description)
    await page.getByLabel("Priority").selectOption(priority)
    await page.getByRole("button", { name: "Create task" }).click()
    await expect(page.getByText(`Created ${title}.`)).toBeVisible()
  }

  for (const title of ["Alpha", "Bravo", "Charlie"]) {
    await page.getByRole("checkbox", { name: title }).check()
  }
  await page.getByRole("button", { name: "Assign selected" }).click()
  await expect(page.getByText("Assigned 3 tasks.")).toBeVisible()

  await page.getByRole("button", { name: "Sign out" }).click()
  await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible()

  await signIn(page, "jamesbond@example.com", "bond-password")
  await expect(page.getByRole("heading", { name: "My tasks", level: 1 })).toBeVisible()
  await expect(
    page.getByRole("region", { name: "My tasks" }).getByText("jamesbond@example.com · staff"),
  ).toBeVisible()
  await expect(page.getByText("3 assigned tasks")).toBeVisible()
  await expect(page.getByText("Cache miss")).toBeVisible()
  for (const title of ["Alpha", "Bravo", "Charlie"]) {
    await expect(page.getByRole("listitem").filter({ hasText: title })).toContainText(
      "Assigned to jamesbond@example.com",
    )
  }
  await expect(page.getByRole("listitem").filter({ hasText: "Delta" })).toHaveCount(0)
  await expect(page.getByRole("listitem").filter({ hasText: "Echo" })).toHaveCount(0)

  await page.getByRole("button", { name: "Load my tasks again" }).click()
  await expect(page.getByText("Cache hit")).toBeVisible()
  await expect(page.getByText("Cache miss")).toHaveCount(0)

  await page.getByLabel("Title").fill("Should fail")
  await page.getByLabel("Description").fill("Staff cannot create this")
  await page.getByLabel("Priority").selectOption("low")
  await page.getByRole("button", { name: "Create task" }).click()
  await expect(page.getByText("You cannot create tasks. Only an admin can.")).toBeVisible()
})
