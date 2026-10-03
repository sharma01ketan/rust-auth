import { defineConfig } from "@playwright/test"

export default defineConfig({
  testDir: "./e2e",
  timeout: 120_000,
  expect: { timeout: 15_000 },
  workers: 1,
  use: { baseURL: "http://127.0.0.1:3100" },
  webServer: [
    {
      command: "rm -f e2e.db e2e.db-wal e2e.db-shm && cargo run",
      cwd: "..",
      port: 3099,
      env: {
        DATABASE_URL: "sqlite://e2e.db",
        JWT_SECRET: "e2e-jwt-secret",
        CODE_PEPPER: "e2e-pepper",
        PORT: "3099",
      },
      reuseExistingServer: false,
      timeout: 180_000,
    },
    {
      command: "npm run dev -- --port 3100",
      port: 3100,
      env: { TASK_API_URL: "http://127.0.0.1:3099" },
      reuseExistingServer: false,
      timeout: 180_000,
    },
  ],
})
