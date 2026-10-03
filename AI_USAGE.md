# AI usage

Cursor, with Grok, was used for this assignment.

The design was settled in a grilling pass before any API code was written. The constraints I set in that pass, and that the implementation follows, are:

- Keep the API in this `rust/` folder.
- Use SQLite, not another database.
- Cache my tasks in memory.
- Do not use an email client. The email log is how a reviewer reads a verification code.
- Seed Admin (`admin@example.com` / `admin-password`) and James Bond (`jamesbond@example.com` / `bond-password`).
- Add `PATCH /tasks/{id}` so a task update can drop the assignee's cache.

The agent then wrote the spec in `.scratch/task-api/spec.md`, the glossary in `CONTEXT.md`, and the two decision records in `docs/adr/`. Implementation followed test-first on the HTTP API: each failing test was run, then the code to pass it was added. I did not hand-edit the Rust after that.

What I still need to be able to explain: the login challenge is stored as a hash, the plaintext code lives only in the email log, sessions are issued only after verify, staff receive 403 on create, and the second my-tasks read is served from the in-memory cache until assign or update drops that user's entry.

The UI was added after a second grilling pass. It is a Next.js app in `ui/` using React 19 and shadcn/ui. The Rust API was not changed. The browser test in `ui/e2e` walks seed, both sign-ins, five creates, assigning three tasks, the forbidden create, and my tasks from cache miss to cache hit.
