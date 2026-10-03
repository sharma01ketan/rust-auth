# Task UI

## Problem Statement

A reviewer can already prove the task API with curl: seed Admin and James Bond, sign both in through a login challenge, create five tasks, assign three to James Bond, see staff blocked from creating a task, and see James Bond's my tasks miss the cache and then hit it. The updated brief also asks for a web UI that performs that same flow against this API. There is no UI today. The API has been tested end to end and must stay as it is.

## Solution

A local Next.js app, using the current stable Next.js, React 19, and shadcn/ui, lets a reviewer complete that flow in the browser. The app calls the existing Rust API. Sign-in still stops at a login challenge until the verification code is confirmed. An admin can create tasks and assign a selection of them to James Bond. A staff User sees only my tasks, including whether that response came from cache, and sees a clear message when creating a task is forbidden.

## User Stories

1. As a reviewer, I want the UI to seed Admin and James Bond, so that I can start the validation flow without curl.
2. As a reviewer, I want seeding twice from the UI to succeed, so that a repeated setup step does not fail the run.
3. As a reviewer, I want the seed result to show both Users and their Roles, and no passwords, so that I can see setup worked.
4. As Admin, I want a sign-in form for email and password, so that I can start a login challenge.
5. As Admin, I want a wrong password to show an error and no verification step, so that a bad password is obvious.
6. As Admin, I want an unknown email to show the same kind of error, so that the UI does not reveal which emails exist.
7. As Admin, I want a successful sign-in to show the verification step and keep me signed out, so that a verification code is still required.
8. As a reviewer, I want the UI to read the latest email log and show the verification code, so that I can finish sign-in without curl or an email client.
9. As a reviewer, I want an empty email log to say that no code has been issued, so that I do not paste a stale value.
10. As Admin, I want to submit the verification code and receive a session, so that I can open the admin screen.
11. As Admin, I want an incorrect verification code to show an error and leave me signed out, so that guessing fails in the UI.
12. As Admin, I want an expired or already used verification code to show an error, so that those codes cannot open a session.
13. As James Bond, I want the same sign-in and verification screens, so that I receive my own session.
14. As a signed-in User, I want to sign out, so that I can switch from Admin to James Bond in one sitting.
15. As a reviewer, I want a signed-out visit to an admin or staff screen to return me to sign-in, so that task data is not shown without a session.
16. As Admin, I want an admin screen after verification, so that I can create and assign tasks.
17. As James Bond, I want a staff screen after verification, so that I see my tasks rather than the admin tools.
18. As Admin, I want to create a task with a title, description, and priority, so that work exists to assign.
19. As Admin, I want each created task to appear in the working list with its title, priority, and status, so that I can choose which ones to assign.
20. As Admin, I want to create five tasks in that list, so that the validation count is met.
21. As Admin, I want a blank title or an unknown priority to show an error and add nothing, so that a bad task is not treated as created.
22. As Admin, I want a success message after each created task, so that I know the API accepted it.
23. As Admin, I want the working list to start empty, so that I am not looking at invented tasks.
24. As Admin, I want to select tasks from the ones I created in this session, so that I can assign exactly three to James Bond.
25. As Admin, I want James Bond's email filled in as the Assignee and still editable, so that the validation assignment is the default.
26. As Admin, I want assigning with nothing selected to be blocked in the form, so that I do not send an empty assignment.
27. As Admin, I want a successful assignment to tell me how many tasks were assigned, so that I can see that the count is three.
28. As Admin, I want an unknown Assignee or an unknown task to show an error and leave the working list unchanged, so that a failed assignment is obvious.
29. As James Bond, I want creating a task from the staff screen to be rejected, so that I see that staff cannot create work.
30. As James Bond, I want that rejection to say I cannot create tasks, so that the block is clear and not only a status code.
31. As James Bond, I want my tasks to list only tasks assigned to me, so that I see exactly three and not the two left unassigned.
32. As James Bond, I want each task to show its title, status, priority, and my email as the Assignee, so that the screen matches the API payload.
33. As James Bond, I want the screen to show my email, staff Role, and a total of three, so that the summary matches the list.
34. As James Bond, I want the first my-tasks view to say the cache missed, so that I can see the response came from the database.
35. As James Bond, I want a second view of the same my tasks to say the cache hit, so that I can see the repeat came from cache.
36. As Admin, I want my own my-tasks view to be an empty state when nothing is assigned to me, so that an empty list is explained rather than blank.
37. As a reviewer, I want my tasks to show a loading state before the API responds, so that a slow call is not a frozen screen.
38. As a reviewer, I want a failed API call to show an error and a way to try again, so that a down API is visible.
39. As a reviewer, I want the task data on screen to come from the Rust API, so that a hard-coded list cannot pass the review.
40. As a reviewer, I want to complete sign-in, create, assign, the forbidden create, and both my-tasks reads with the keyboard, so that the flow is usable without a pointer.
41. As a reviewer, I want the layout to remain usable on a narrow screen and on a desktop screen, so that the validation flow is not tied to one window size.
42. As a reviewer, I want setup and run instructions for the UI next to the existing API instructions, so that the submission can be repeated.
43. As a reviewer, I want screenshots of the running sign-in, admin, and staff screens, so that the submission shows the UI without starting it.

## Implementation Decisions

- The Rust crate stays unchanged. No new routes, no CORS headers, no Postgres, no Redis, no email client. The SQLite ADR and the in-memory cache ADR still stand.
- The UI is a separate Next.js application inside the task API project. It is not part of the dots-and-boxes game, and it does not change the game crates.
- Use the current stable releases at implementation time: Next.js 16 (App Router, Turbopack default), React 19, and shadcn/ui. Scaffold with the shadcn Next.js setup so Tailwind and the shadcn tokens are the design system. Leave the React Compiler off.
- Visual language is the shadcn neutral theme: semantic color tokens, the spacing scale, and composition of shadcn parts (card, form, button, input, select, checkbox, alert, badge, skeleton). Priority and status are labeled in text, not by color alone. No gradient hero, no purple default palette, no stock card grid that is unrelated to the flow.
- Pages are Server Components. A component is a Client Component only when it holds form state or a selection. Presentational pieces receive data. They do not call the API.
- One server-side API module talks to the Rust API. The browser never calls the Rust origin. The base URL comes from the Next server environment and defaults to the API on port 3000. Authenticated calls send `Authorization: Bearer` with the access token the API returned from verify.
- The access token is stored in an httpOnly cookie set when verify succeeds. Sign-out clears that cookie. Role and email are read from the token claims the API already issues (`email`, `role`). `cookies()` is awaited, matching Next.js 16.
- Sign-in, verification, seed, create, assign, and the staff create attempt are React Actions. Pending and result state use `useActionState`. The submit control uses `useFormStatus`. Do not track form pending with a separate `useState`. Forms use the form action. Do not fetch task data in an effect.
- Do not mark my-tasks reads with `"use cache"` or any Next data cache. Next.js 16 runs dynamic work at request time unless caching is opted in. Each my-tasks view must be a real request to the Rust API, so `cache.hit` is the Rust cache and not a Next cache.
- The admin working list is the tasks returned by create during this signed-in visit. The API has no route that lists every task, and this spec does not add one. A refresh clears that working list. Assignment sends the selected ids and the Assignee email to the existing assign route.
- After verify, an admin lands on the admin screen and a staff User lands on the staff screen. The staff screen shows my tasks and a create form whose only successful outcome in the validation flow is the forbidden message.
- The forbidden message is a sentence a reviewer can read. The raw API error can sit beside it. A 401 returns the User to sign-in.
- Loading uses a skeleton with a busy state. Empty my tasks is its own state, not an empty table with no explanation. Errors use an alert. Success after create and after assign is inline confirmation, including the assigned count.
- My tasks renders `user.email`, `user.role`, each task's title, status, priority, and `assigned_to`, `summary.total_assigned_tasks`, and `cache.hit`. Order is the API order.
- The latest email log is a development readout on the verification step. It shows the recipient, the code, and the time. It does not submit the code by itself.
- Reach WCAG 2.1 AA for this flow: labeled controls, keyboard access, visible focus, text for status, and no information that is color alone. Check 320, 768, 1024, and 1440 widths.
- The README gains frontend setup, how to run it against the API, and the validation clicks. `AI_USAGE.md` records the UI work. Screenshots of the running sign-in, admin, and staff screens are part of the submission.
- Sources for the framework choices: [React 19 Actions](https://react.dev/blog/2024/12/05/react-19#actions), [`useActionState`](https://react.dev/reference/react/useActionState), [`useFormStatus`](https://react.dev/reference/react/useFormStatus), [Next.js 16](https://nextjs.org/blog/next-16) (dynamic by default, `"use cache"` is opt-in, `proxy.ts` replaces `middleware.ts`, async `cookies()`), and [shadcn/ui for Next.js](https://ui.shadcn.com/docs/installation/next). UI structure follows the component, state, and accessibility rules in Addy Osmani's frontend UI engineering skill.

## Testing Decisions

A good test drives the UI the way a reviewer would and checks what is on screen. It does not assert component state, hook calls, cookie names, or how the Rust cache map is stored.

The new seam is one browser test against the running Next.js app and the running Rust API. It is the highest seam that can see the updated brief's check: James Bond's screen shows the same three tasks as the API. The existing HTTP tests stay the seam for the API and are not extended for the UI. There is no second frontend seam of component or hook tests.

Prior art is the Rust HTTP suite: one behavioral test walks seed, both sign-ins, five creates, an assignment of three, a forbidden create, and my tasks from cache miss to cache hit. The browser test walks that same story through the screens. It expects the three titles, the staff summary of three, a cache miss and then a cache hit, and the staff forbidden message. Wrong password and an empty my-tasks state are part of that same story, not extra suites.

## Out of Scope

- Any change to the Rust handlers, schema, cache, auth, or error shape
- CORS on the Rust server, or a new list-all-tasks route
- Postgres, Redis, an email client, OpenAPI, and refresh sessions
- The dots-and-boxes game crates
- Production deployment and a shared cache across processes
- React Compiler, Cache Components, and a `"use cache"` layer in front of my tasks
- A design beyond the shadcn neutral theme
- Remembering the admin working list after a refresh

## Further Notes

The backend validation in the session summary already passed, including the live James Bond my-tasks body. This spec adds the UI on top of that API.

The issue tracker for this repo is not configured, so this spec is a local file rather than a GitHub issue. `ready-for-agent` was not applied. Run `/setup-matt-pocock-skills` if later specs should be published to a tracker.

The test seam is the browser flow above. If that seam is wrong, say so before implementation.
