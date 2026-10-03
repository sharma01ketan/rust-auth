# Session summary

This file records the work done after the original handoff. The handoff text below is the context from the first message, kept as it was given. It lives on disk as `llms_data.md`.

The first request was: take all of the data in `llms_data.md`, and run the test cases defined in the codebase and in `assessment.md`, step by step.

## Original context

Local Rust task API for a timed coding assignment. The work lives in this `rust/` folder inside the dots-and-boxes git repo. The game crates are a different product. Do not change them.

Commit `a689117` on branch `main-hinting-v1` contains this API. Other game files were already staged and were left staged. Do not commit those unless the user asks.

### What the user wants

Pass the assignment checks. Quality improvements only if time is left. The user is the solver and must be able to explain the code.

Hard constraints from the user:

- This folder only. Do not touch `core`, `server`, `web`, `wasm`, or `wasm-az`.
- SQLite only. Connection string is a `sqlite://` URL. Do not add Postgres or enable the sqlx Postgres feature.
- In-memory cache only. Do not add Redis.
- No email client. The email log is how a reviewer reads a verification code.
- Time box was about 45 minutes. The validation flow matters more than extra polish.

The original brief is `assessment.md`. The agreed spec is `.scratch/task-api/spec.md`. Glossary is `CONTEXT.md`. Decision records are `docs/adr/0001-sqlite.md` and `docs/adr/0002-in-memory-cache.md`.

### Decisions already agreed

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

### Language

Use these words: User, Role (`admin` or `staff`), Task, Assignee, Assignment, login challenge, verification code, email log, my tasks. "Admin" is the seed person. `admin` is the role.

### Layout

- `src/main.rs` loads `.env`, opens SQLite, runs migrations, listens.
- `src/lib.rs` holds the clock, `AppState`, router, `ApiError`, and `TestApp`.
- `src/identity.rs` seeds users, login, verify, email log, password hashing, JWT.
- `src/tasks.rs` creates, updates, assigns, lists my tasks, and updates the cache map.
- `migrations/0001_init.sql` creates `users`, `tasks`, `login_challenges`, `email_logs`.
- `tests/http_api.rs` is the only test seam: HTTP against a temp SQLite file and `ManualClock`.
- `.env.example` has `DATABASE_URL=sqlite://dev.db`, `JWT_SECRET`, `CODE_PEPPER`, `PORT`.
- `dev.db` is a local file from the validation run. It is gitignored.

Tests construct the app through `TestApp::new()`. They advance `app.clock` to expire codes. They do not query SQL.

### Status at the start of the session

Done and checked before this session:

- `cargo test` — 5 tests passed.
- `cargo clippy --all-targets -- -D warnings` passed.
- Live curl on port 3456: seed, Admin 2FA, five tasks, assign three to James Bond, James Bond 2FA, create as James Bond returned 403, first my-tasks `cache.hit` false with three tasks (high, medium, low), second call `cache.hit` true.
- That first JSON body is in `README.md`. Ids in that sample are from that run and will differ next time.
- sqlx is built with `default-features = false` and only `macros`, `migrate`, `runtime-tokio`, `sqlite`. `cargo tree -i sqlx-postgres` printed nothing.

### Review still open

These were noted and not fixed. The validation flow already passes.

- Spec asked for a separate my-tasks cache module. The map lives on `AppState` and is updated in `tasks.rs`.
- Spec put session extraction with HTTP. `current_user` is in `identity.rs`.
- A JSON body missing required fields is Axum's 422, not the `{ "error" }` 400 used for a blank title.
- `PORT` is read even though the spec only named the other three env vars.
- Update rejects a blank title with 400. Titles are trimmed. Login challenges also store `created_at`.

### How to run

```bash
cd rust
cp .env.example .env
cargo run
cargo test
```

Migrations run on startup. Server default port is 3000.

### What not to do

- Do not add Postgres, Redis, SMTP, a frontend, OpenAPI, or refresh tokens.
- Do not edit the parent game crates to make this crate a workspace member.
- Do not commit unless the user asks. The implementation commit already exists.

## What this session did

No application code was changed. The session re-ran the checks, walked the `assessment.md` flow live, then committed and published when asked.

### Codebase tests

`cargo test -- --test-threads=1` passed: 5 tests, 0 failed, about 6 seconds.

| Test | What it checks |
| --- | --- |
| `seed_creates_admin_and_james_bond_and_can_run_twice` | Admin and James Bond are created. A second seed keeps the same ids and returns no passwords. |
| `login_returns_a_challenge_and_records_the_verification_code` | Login returns only `login_challenge_id`. The email log then has a 6-digit code. An empty log is 404. |
| `verify_issues_a_session_and_rejects_bad_codes` | Wrong password, unknown email, wrong code, expired code, reused code, a newer login replacing an old one, and a swapped code all return 401. A correct code returns a three-part JWT. |
| `admin_creates_and_updates_tasks_and_staff_cannot` | Missing or forged tokens are 401. James Bond creating or updating a task is 403. Blank title, unknown priority, and unknown status are 400. Admin creates five tasks at 201, then updates one. |
| `james_bond_sees_three_assigned_tasks_and_the_second_read_hits_cache` | Staff cannot assign. Empty ids are 400. Unknown user or task is 404 and changes nothing. First my-tasks read is three tasks with `cache.hit` false. The second is true. An unassigned create does not drop the cache. Updating Alpha does. Reassigning Charlie to Admin drops both caches. |

Wrong, expired, and reused codes, plus cache invalidation, were covered here. They were not repeated on the live server, because that server uses the real clock.

### Live server for the interview

Port 3000 was already taken by an SSH listener, so it was left alone. One API process was kept, on port 3456, with a fresh database `sqlite://demo.db` so James Bond would see exactly three tasks. `.env` was copied from `.env.example` and pointed at that port and database for the demo. `.env` and `*.db` are gitignored.

The required `assessment.md` flow was then completed in Hoppscotch and curl, in order:

1. `POST /seed/users` returned 200. Admin `3d23a277-1b55-410c-9fbd-1cea6e8e21c6` (`admin`) and James Bond `6c3cd822-f926-42a4-a4e4-5bc8a2b14ea9` (`staff`). No passwords in the body.
2. `POST /auth/login` as Admin (`admin-password`) returned only `login_challenge_id` `b5c6a35f-9af0-4880-8bb9-0bb358c3c1b2`.
3. `GET /dev/email-logs/latest` returned `to: admin@example.com`, code `363157`.
4. `POST /auth/verify-2fa` returned an Admin JWT.
5. Five `POST /tasks` calls as Admin returned 201, status `todo`, `assigned_to` null:
   - Alpha `017d74cd-1fd7-49c2-a230-6007e3ff119f` high
   - Bravo `a65398b9-1777-4c7c-b6dd-f3fd5fe3436b` medium
   - Charlie `82c486f6-8cef-4cc7-b2c9-5ab1d2d12964` low
   - Delta `ba9b430b-58dc-4ef7-9042-fdb30d37dd0d` high
   - Echo `db0e5f67-02be-401e-b896-0956a7c077c5` low
6. `POST /tasks/assign` of Alpha, Bravo, and Charlie to `jamesbond@example.com` returned `{"assigned": 3}`.
7. James Bond login (`bond-password`) returned challenge `bc4326a5-292b-4a03-9b64-fdcb48e062c3`. The email log then returned code `228982` for `jamesbond@example.com`.
8. Verify returned James Bond's JWT. His `sub` is `6c3cd822-f926-42a4-a4e4-5bc8a2b14ea9` and his role is `staff`.
9. `POST /tasks` as James Bond returned 403 `{"error":"forbidden"}`.
10. First `GET /tasks/view-my-tasks` returned Alpha, Bravo, and Charlie, `total_assigned_tasks` 3, `cache.hit` false.
11. The same call again returned the same three tasks with `cache.hit` true.

That is the validation point in `assessment.md`.

### Two Hoppscotch tabs

Login was sent from one Hoppscotch tab and `GET /dev/email-logs/latest` from another. That is expected. Login inserts the plaintext code into the shared SQLite `email_logs` table. The latest-log route reads the newest row from that table. It is unauthenticated and is not tied to the client that logged in. A curl from a third client returned the same James Bond row (`228982`). A later login by anyone would replace what "latest" shows. A new login for the same user also expires that user's unused challenge.

### Commit

The user asked for one new commit of everything that was staged. `.env` and the SQLite files were not included.

Commit `a02d2b5` on `main-hinting-v1`: "Hand off decomposed AlphaZero endgames to Perfect or CGT."

Six files: the AlphaZero endgame handoff in `core`, the spec and architecture note, the rebuilt WASM package, and `rust/llms_data.md`. After the commit the branch was 2 commits ahead of `origin/main-hinting-v1` (`a689117` for the task API, then `a02d2b5`). It was not pushed to that remote.

### Public repository

The user asked for a new public GitHub repository named `rust-auth`.

https://github.com/sharma01ketan/rust-auth

It is public. It contains only the `rust/` task API (auth, two-factor login, tasks, tests, and the README), published as its own `main` history. The dots-and-boxes game stayed in the original repo. `.env` and the database files were not pushed.
