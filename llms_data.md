# Handoff for the next LLM

Local Rust task API for a timed coding assignment. The work lives in this `rust/` folder inside the dots-and-boxes git repo. The game crates are a different product. Do not change them.

Commit `a689117` on branch `main-hinting-v1` contains this API. Other game files were already staged and were left staged. Do not commit those unless the user asks.

## What the user wants

Pass the assignment checks. Quality improvements only if time is left. The user is the solver and must be able to explain the code.

Hard constraints from the user:

- This folder only. Do not touch `core`, `server`, `web`, `wasm`, or `wasm-az`.
- SQLite only. Connection string is a `sqlite://` URL. Do not add Postgres or enable the sqlx Postgres feature.
- In-memory cache only. Do not add Redis.
- No email client. The email log is how a reviewer reads a verification code.
- Time box was about 45 minutes. The validation flow matters more than extra polish.

The original brief is `assessment.md`. The agreed spec is `.scratch/task-api/spec.md`. Glossary is `CONTEXT.md`. Decision records are `docs/adr/0001-sqlite.md` and `docs/adr/0002-in-memory-cache.md`.

## Decisions already agreed

1. Self-contained crate in `rust/`. Its `Cargo.toml` has an empty `[workspace]` table so Cargo does not merge it into the parent workspace.
2. Axum, Tokio, SQLx (SQLite only), Argon2 passwords, HS256 JWTs.
3. `PATCH /tasks/{id}` exists so an update can drop the assignee cache. It changes title, description, status, and priority. It does not change the assignee. Assignment stays on `POST /tasks/assign`.
4. Seed users, passwords fixed in code and documented, not read from the environment:
   - Admin, `admin@example.com`, role `admin`, password `admin-password`
   - James Bond, `jamesbond@example.com`, role `staff`, password `bond-password`
   - `POST /seed/users` is safe to call twice. Existing rows stay as they are. Responses omit passwords.
5. Login returns `{ "login_challenge_id" }` only. No JWT yet. A 6-digit code is stored on the challenge as `SHA-256(pepper || code)`. A new login expires unused challenges for that user. Codes last 5 minutes and work once. Wrong, expired, reused, and swapped codes return 401 and do not consume a still-valid challenge. Unknown email and wrong password both return 401 and write no email log.
6. The email log stores the plaintext code. `GET /dev/email-logs/latest` is unauthenticated and returns `{ "to", "code", "created_at" }`, or 404 when empty.
7. `POST /auth/verify-2fa` with `{ "login_challenge_id", "code" }` returns `{ "access_token" }`. Claims are `sub` (user id), `email`, `role`, and an 8-hour `exp`. Expiry is checked against the app clock, not the host clock, so tests can move time.
8. `POST /tasks` body is `title`, `description`, `priority` (`low`, `medium`, `high`). Status starts as `todo`. `assigned_to` is null. Admin only. Staff get 403. Blank title or unknown priority is 400.
9. `POST /tasks/assign` body is `task_ids` plus `assignee_email`. Admin only. Empty `task_ids` is 400. Unknown user or any unknown task is 404 and changes nothing. Reassignment replaces the previous assignee.
10. `GET /tasks/view-my-tasks` returns tasks for the signed-in user, oldest first. Shape:

```json
{
  "user": { "email": "...", "role": "staff" },
  "tasks": [
    { "id": "...", "title": "...", "status": "todo", "priority": "high", "assigned_to": "jamesbond@example.com" }
  ],
  "summary": { "total_assigned_tasks": 3 },
  "cache": { "hit": false }
}
```

11. One cache entry per user id. No TTL. First read loads from SQLite and returns `cache.hit: false`. The next identical read returns `cache.hit: true`. Assign forgets the previous assignee and the new assignee. Update forgets the current assignee when there is one. Creating a task does not forget anyone, because a new task has no assignee.
12. Error bodies from the handlers are `{ "error": "<short message>" }`. Status codes: 401 unauthorized, 403 wrong role, 404 missing, 400 bad input, 201 create, 200 everything else that succeeds.
13. Ids are UUID strings. Times are RFC3339 millisecond strings from the app clock.

## Language

Use these words: User, Role (`admin` or `staff`), Task, Assignee, Assignment, login challenge, verification code, email log, my tasks. "Admin" is the seed person. `admin` is the role.

## Layout

- `src/main.rs` loads `.env`, opens SQLite, runs migrations, listens.
- `src/lib.rs` holds the clock, `AppState`, router, `ApiError`, and `TestApp`.
- `src/identity.rs` seeds users, login, verify, email log, password hashing, JWT.
- `src/tasks.rs` creates, updates, assigns, lists my tasks, and updates the cache map.
- `migrations/0001_init.sql` creates `users`, `tasks`, `login_challenges`, `email_logs`.
- `tests/http_api.rs` is the only test seam: HTTP against a temp SQLite file and `ManualClock`.
- `.env.example` has `DATABASE_URL=sqlite://dev.db`, `JWT_SECRET`, `CODE_PEPPER`, `PORT`.
- `dev.db` is a local file from the validation run. It is gitignored.

Tests construct the app through `TestApp::new()`. They advance `app.clock` to expire codes. They do not query SQL.

## Status

Done and checked:

- `cargo test` — 5 tests passed.
- `cargo clippy --all-targets -- -D warnings` passed.
- Live curl on port 3456: seed, Admin 2FA, five tasks, assign three to James Bond, James Bond 2FA, create as James Bond returned 403, first my-tasks `cache.hit` false with three tasks (high, medium, low), second call `cache.hit` true.
- That first JSON body is in `README.md`. Ids in that sample are from that run and will differ next time.
- sqlx is built with `default-features = false` and only `macros`, `migrate`, `runtime-tokio`, `sqlite`. `cargo tree -i sqlx-postgres` printed nothing.

## Review still open

These were noted and not fixed. The validation flow already passes.

- Spec asked for a separate my-tasks cache module. The map lives on `AppState` and is updated in `tasks.rs`.
- Spec put session extraction with HTTP. `current_user` is in `identity.rs`.
- A JSON body missing required fields is Axum's 422, not the `{ "error" }` 400 used for a blank title.
- `PORT` is read even though the spec only named the other three env vars.
- Update rejects a blank title with 400. Titles are trimmed. Login challenges also store `created_at`.

## How to run

```bash
cd rust
cp .env.example .env
cargo run
cargo test
```

Migrations run on startup. Server default port is 3000.

## What not to do

- Do not add Postgres, Redis, SMTP, a frontend, OpenAPI, or refresh tokens.
- Do not edit the parent game crates to make this crate a workspace member.
- Do not commit unless the user asks. The implementation commit already exists.
