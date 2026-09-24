# Task API

## Problem Statement

A reviewer has 45 minutes to confirm a local task API. They must seed Admin and James Bond, sign both in with a verification code, create five tasks, assign three to James Bond, prove staff cannot create tasks, and show that James Bond's my tasks come from the SQLite file on the first read and from cache on the second.

## Solution

A local HTTP API implements that workflow. Sign-in always stops at a login challenge until the verification code is confirmed. Only an admin can create, update, and assign tasks. My tasks returns the signed-in User's assignments, and the response says whether that payload came from cache.

## User Stories

1. As a reviewer, I want to seed Admin and James Bond, so that the validation flow has one admin and one staff User.
2. As a reviewer, I want seeding twice to succeed, so that a repeated setup step does not fail the run.
3. As a reviewer, I want seed responses to omit passwords, so that the password hashes are not echoed back.
4. As Admin, I want to start sign-in with email and password, so that the API can check who I am.
5. As Admin, I want a wrong password to be rejected, so that a bad password does not start a login challenge.
6. As Admin, I want an unknown email to be rejected the same way as a wrong password, so that the API does not reveal which emails exist.
7. As Admin, I want sign-in to return a login challenge id and no session, so that a verification code is required.
8. As Admin, I want a verification code written to the email log, so that I can finish sign-in without an email client.
9. As a reviewer, I want to read the latest email log, so that I can copy the current verification code.
10. As a reviewer, I want the latest-email endpoint to fail clearly when nothing has been issued, so that an empty log is obvious.
11. As Admin, I want the correct verification code to return a session, so that I can call admin routes.
12. As Admin, I want an incorrect verification code rejected, so that guessing fails.
13. As Admin, I want an expired verification code rejected, so that a code dies after five minutes.
14. As Admin, I want a reused verification code rejected, so that a code works only once.
15. As Admin, I want a new sign-in to expire my unused login challenge, so that only the latest verification code works.
16. As Admin, I want a verification code for a different login challenge rejected, so that codes cannot be swapped across challenges.
17. As James Bond, I want the same sign-in flow, so that I receive my own session.
18. As a caller without a session, I want task routes to reject me, so that tasks stay private.
19. As a caller with a bad session, I want task routes to reject me, so that a forged session fails.
20. As Admin, I want to create a task with a title, description, and priority, so that work exists to assign.
21. As Admin, I want a new task to start as todo with no Assignee, so that assignment is a separate step.
22. As Admin, I want to create five tasks, so that the validation count is met.
23. As Admin, I want a create request with a missing title or an unknown priority rejected, so that bad tasks are not stored.
24. As James Bond, I want creating a task to be forbidden, so that staff cannot create work.
25. As James Bond, I want updating a task to be forbidden, so that staff cannot change work.
26. As James Bond, I want assigning tasks to be forbidden, so that staff cannot change assignment.
27. As Admin, I want to update a task's title, description, status, and priority, so that an update can refresh my tasks.
28. As Admin, I want updating a missing task to fail, so that unknown ids are not silent.
29. As Admin, I want to assign chosen tasks to James Bond by email, so that exactly three tasks become his.
30. As Admin, I want assigning an unknown task or unknown User to fail, so that bad assignment does not partially succeed.
31. As Admin, I want reassignment to replace the previous Assignee, so that a task has one Assignee.
32. As James Bond, I want my tasks to list only tasks assigned to me, so that I see exactly three and not the two left unassigned.
33. As James Bond, I want each of those tasks to show my email as the Assignee, so that the validation payload matches.
34. As James Bond, I want my tasks to include my email, staff role, and a total of three, so that the summary matches the list.
35. As Admin, I want my own my-tasks view to be empty when nothing is assigned to me, so that the list follows the Assignee rather than the creator.
36. As James Bond, I want the first my-tasks read after a change to come from the SQLite file, so that the response shows a cache miss.
37. As James Bond, I want the next identical my-tasks read to come from cache, so that the response shows a cache hit.
38. As James Bond, I want assignment of one of my tasks to someone else to drop my cached my tasks, so that the following read is a miss and no longer includes that task.
39. As James Bond, I want an update to one of my tasks to drop my cached my tasks, so that the following read is a miss and shows the new fields.
40. As a reviewer, I want setup, migrate, run, seed, and the curl validation written down, so that the submission can be repeated.
41. As a reviewer, I want the final James Bond my-tasks response saved in that writeup, so that the validation point is visible without re-running it.

## Implementation Decisions

- The API lives in this `rust/` folder as its own service. The dots-and-boxes game crates stay untouched.
- Persistence is one SQLite file, opened with the SQLx SQLite driver. The connection string is a `sqlite://` URL. See the SQLite ADR.
- My tasks cache is in-process memory. See the in-memory cache ADR.
- No email client. The email log is the only delivery of a verification code.
- HTTP stack is Axum on Tokio. SQL access and migrations use SQLx. Passwords use Argon2. Sessions are HS256 JWTs.
- Identifiers are UUIDs.
- Passwords for the seed Users are fixed and documented: Admin is `admin-password`, James Bond is `bond-password`. They are not loaded from the environment.
- `JWT_SECRET` and `CODE_PEPPER` come from the environment. `DATABASE_URL` is a `sqlite://` URL pointing at the SQLite file.

### Modules

- **Identity**: seed Users, check passwords, create and consume login challenges, write and read the email log, issue sessions.
- **Tasks**: create, update, assign, and list my tasks from the SQLite file.
- **My-tasks cache**: remember one my-tasks payload per User, and forget the Users affected by an assignment or an update.
- **HTTP**: routes, session extraction, and status codes. This is the only test seam.

### Data

- A User has id, full name, email, password hash, role, created time, and updated time. Email is unique. Role is `admin` or `staff`.
- A Task has id, title, description, status, priority, creator id, assignee id, created time, and updated time. Assignee id is empty until assignment. Status is `todo`, `in_progress`, or `done`. Priority is `low`, `medium`, or `high`.
- A login challenge has id, user id, a hash of the verification code, an expiry time, and a used time. The hash is SHA-256 of the pepper plus the code. The plaintext code is not stored on the challenge.
- An email log entry has id, recipient email, the plaintext verification code, and a created time. The dev read returns the newest entry.

### HTTP contract

JSON fields are snake_case.

- `POST /seed/users` with an empty body. `200` and `{ "users": [{ "id", "full_name", "email", "role" }] }`. Inserts a User only when that email is missing. A second call leaves existing rows unchanged.
- `POST /auth/login` with `{ "email", "password" }`. `200` and `{ "login_challenge_id" }` only. No session field. `401` when the email is unknown or the password is wrong, and no email log entry is written. A successful login expires any unused login challenge for that User, stores a 6-digit code hash that expires in five minutes, and appends an email log entry.
- `GET /dev/email-logs/latest` with no session. `200` and `{ "to", "code", "created_at" }`. `404` when the log is empty.
- `POST /auth/verify-2fa` with `{ "login_challenge_id", "code" }`. `200` and `{ "access_token" }`. `401` when the challenge is unknown, the code is wrong, the challenge is expired, or the code was already used. A success marks the challenge used. The session claims are the User id (`sub`), email, role, and an eight-hour expiry.
- `POST /tasks` with a session and `{ "title", "description", "priority" }`. `201` and the task, status `todo`, `assigned_to` null. `401` without a valid session. `403` for staff. `400` when the title is blank or the priority is unknown.
- `PATCH /tasks/{id}` with a session and any of `title`, `description`, `status`, `priority`. `200` and the updated task. `401` without a valid session. `403` for staff. `404` when the task does not exist. `400` when a present status or priority is unknown. This route does not change the Assignee.
- `POST /tasks/assign` with a session and `{ "task_ids", "assignee_email" }`. `200` and `{ "assigned": <count> }`. `401` without a valid session. `403` for staff. `404` when the assignee email or any task id does not exist, and no task is changed. `400` when `task_ids` is empty. Reassignment replaces the previous Assignee.
- `GET /tasks/view-my-tasks` with a session. `200` and the validation shape: `user.email`, `user.role`, `tasks` (id, title, status, priority, `assigned_to` as the Assignee email), `summary.total_assigned_tasks`, and `cache.hit`. Tasks are ordered by created time, oldest first. `401` without a valid session.

### Cache

The first my-tasks read for a User after a cold start, an assignment that names them, or an update of a task currently assigned to them loads from the SQLite file and returns `cache.hit` false. The next identical read returns `cache.hit` true. There is no time-to-live. Assignment forgets both the previous Assignee and the new Assignee. An update forgets the current Assignee when the task has one. Creating a task forgets nothing, because a new task has no Assignee.

### Errors

Error bodies are `{ "error": "<short message>" }`. Tests assert the status code. The message is for humans reading curl output.

## Testing Decisions

A good test exercises the HTTP API and checks status codes and JSON. It does not assert password hashes, code hashes, or how the cache map is stored.

The only seam is HTTP. Tests build the app against a temporary SQLite file and a clock the test controls, so an expired verification code is a verify call after the clock has moved past five minutes, not a real wait. Cache behavior is asserted only through `cache.hit` and the task list.

There is no prior test suite in this folder. The tests cover the brief's list: seed both Users, login returns a challenge and no session, the correct code returns a session, wrong or expired or reused codes are rejected, Admin can create five tasks, Admin can assign three to James Bond, James Bond receives `403` on create, James Bond sees exactly those three tasks, two reads go from cache miss to cache hit, and both assignment and update make the next read a miss.

## Out of Scope

- An email client, SMTP, or any delivery besides the email log
- Redis, Docker, and OpenAPI
- Any database other than the SQLite file
- Refresh sessions, password reset, and extra Roles
- A frontend
- Changes to the dots-and-boxes game crates
- Production deployment

## Further Notes

The time box is 45 minutes. The bar is the validation flow and the tests above. The README must include the James Bond my-tasks body from a real run, with `cache.hit` false on the first call. The issue tracker for this repo was not configured, so this spec lives as a local file rather than a GitHub issue on the game repository.
