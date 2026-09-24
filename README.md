# Task API

Local task API for the assignment. Users, tasks, login challenges, and email logs are stored in one SQLite file. My tasks are cached in process memory. Verification codes are written to the email log. There is no email client.

## Setup

```bash
cd rust
cp .env.example .env
```

Seed passwords are fixed:

- Admin: `admin@example.com` / `admin-password`
- James Bond: `jamesbond@example.com` / `bond-password`

## Migrations

`cargo run` applies the SQLx migrations in `migrations/` before it listens.

## Run

```bash
cargo run
```

The server listens on `PORT` (default `3000`).

## Seed

```bash
curl -s -X POST http://127.0.0.1:3000/seed/users \
  -H 'content-type: application/json' \
  -d '{}'
```

Calling seed again is safe. Existing users are left unchanged.

## Validation

1. Start login as Admin. The response is a `login_challenge_id` and does not include a session.

```bash
curl -s -X POST http://127.0.0.1:3000/auth/login \
  -H 'content-type: application/json' \
  -d '{"email":"admin@example.com","password":"admin-password"}'
```

2. Read the verification code.

```bash
curl -s http://127.0.0.1:3000/dev/email-logs/latest
```

3. Verify Admin and keep the `access_token`.

```bash
curl -s -X POST http://127.0.0.1:3000/auth/verify-2fa \
  -H 'content-type: application/json' \
  -d '{"login_challenge_id":"<id>","code":"<code>"}'
```

4. Create five tasks as Admin. Repeat with different titles and priorities (`high`, `medium`, `low`).

```bash
curl -s -X POST http://127.0.0.1:3000/tasks \
  -H "authorization: Bearer <admin token>" \
  -H 'content-type: application/json' \
  -d '{"title":"Alpha","description":"Work item","priority":"high"}'
```

5. Assign three of those task ids to James Bond.

```bash
curl -s -X POST http://127.0.0.1:3000/tasks/assign \
  -H "authorization: Bearer <admin token>" \
  -H 'content-type: application/json' \
  -d '{"task_ids":["<id>","<id>","<id>"],"assignee_email":"jamesbond@example.com"}'
```

6. Sign James Bond in the same way (login, latest email log, verify).

7. Creating a task as James Bond returns `403`.

8. `GET /tasks/view-my-tasks` with James Bond's token returns the three assigned tasks. The first response has `cache.hit` false. The same call again has `cache.hit` true.

## Tests

```bash
cargo test
```

The tests call the HTTP API against a temporary SQLite file. A test-controlled clock expires verification codes without waiting five minutes.

## Cache

The my-tasks cache is an in-memory map, one entry per user, with no expiry. It is not shared across processes. Assignment and task updates drop the affected users' entries.

## Final `GET /tasks/view-my-tasks` response

This body is from a local run as James Bond after three tasks were assigned. Ids change on each run. `cache.hit` was `false` on this first read and `true` on the immediate repeat.

```json
{
  "user": {
    "email": "jamesbond@example.com",
    "role": "staff"
  },
  "tasks": [
    {
      "id": "d538d6c0-b20a-4ec0-8efc-107b7ea31e88",
      "title": "Alpha",
      "status": "todo",
      "priority": "high",
      "assigned_to": "jamesbond@example.com"
    },
    {
      "id": "f06727b4-7e11-4416-8960-1c97f9721a8e",
      "title": "Bravo",
      "status": "todo",
      "priority": "medium",
      "assigned_to": "jamesbond@example.com"
    },
    {
      "id": "a74e1f80-dc3e-4e1b-b8a1-9db4c503bab0",
      "title": "Charlie",
      "status": "todo",
      "priority": "low",
      "assigned_to": "jamesbond@example.com"
    }
  ],
  "summary": {
    "total_assigned_tasks": 3
  },
  "cache": {
    "hit": false
  }
}
```
