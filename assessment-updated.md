**Full Stack Rust Developer**  
**Coding Assignment**

Task Management App with Rust API, Auth, 2FA, Role-Based Access, Caching, and a Simple Frontend

| Time expectation | Primary stack | Validation point |
| ----- | ----- | ----- |
| 60 min | Frontend \+ Rust web API | UI \+ GET /tasks/view-my-tasks |

# **Objective**

Build a small full-stack task management application. The goal is to test frontend integration, Rust backend development, authentication, two-factor login, role-based permissions, task assignment, caching, clean data flow, and maintainable project structure.

**AI tools are allowed.** We do not mind how the candidate builds it, but they must understand and explain the submitted code.

# **Full Stack Scope**

* Frontend: Build a simple web UI using React, Next.js, Vue, or another modern frontend framework.  
* Backend: Use Rust and a production-appropriate web API framework.  
* The frontend must consume the candidate's Rust API rather than using hard-coded task data.  
* Keep the UI simple. Functional behavior and clean integration matter more than visual polish.

# **Required Workflow**

When the application is running locally, we should be able to complete this flow using the frontend and/or curl, Postman, or Swagger/OpenAPI documentation.

1. Create two users: Admin and James Bond.  
2. Start login as Admin using email and password. The API should trigger a two-factor email code and return a login\_challenge\_id, not a JWT.  
3. Retrieve the verification code from a local development email log or console output.  
4. Verify Admin 2FA and receive an Admin JWT token.  
5. Create exactly 5 tasks as Admin.  
6. Assign exactly 3 of those tasks to James Bond.  
7. Start login as James Bond and retrieve his two-factor verification code.  
8. Verify James Bond 2FA and receive a James Bond JWT token.  
9. Attempt to create a task as James Bond. This must return 403 Forbidden.  
10. View James Bond's tasks in the frontend. It must show exactly 3 assigned tasks.  
11. Call GET /tasks/view-my-tasks again. The response should come from cache and show cache.hit \= true.

# **Frontend Requirements**

* Login screen for email/password and 2FA verification.  
* Admin screen to create tasks and assign selected tasks to James Bond.  
* Staff screen to show the logged-in user's assigned tasks.  
* Basic loading, error, empty, and success states.  
* Use the JWT returned by the Rust API for authenticated requests.  
* Show a clear message when James Bond is blocked from creating a task.  
* Keep components and API calls separated cleanly; no need for advanced styling.

# **Expected API Shape**

The exact route names can vary slightly, but the following capabilities must exist.

| Endpoint | Purpose |
| :---- | :---- |
| POST /seed/users | Create Admin and James Bond users for validation. |
| POST /auth/login | Validate email/password, create a 2FA challenge, and trigger an email code. |
| GET /dev/email-logs/latest | Development-only endpoint to view the latest sent verification code. |
| POST /auth/verify-2fa | Verify the code and return a JWT access token. |
| POST /tasks | Create a task. Admin only. |
| POST /tasks/assign | Assign selected tasks to James Bond. Admin only. |
| GET /tasks/view-my-tasks | Return tasks assigned to the logged-in user, with cache metadata. |

# **Final Validation Response**

The main validation point remains the final James Bond response. If this response is correct and the frontend displays the same three tasks, we can quickly confirm that the core full-stack flow works.

GET /tasks/view-my-tasks  
Authorization: Bearer JAMES\_BOND\_TOKEN

Expected response:  
{  
  "user": {  
    "email": "jamesbond@example.com",  
    "role": "staff"  
  },  
  "tasks": \[  
    {"id":"...","title":"...","status":"todo","priority":"high","assigned\_to":"jamesbond@example.com"},  
    {"id":"...","title":"...","status":"todo","priority":"medium","assigned\_to":"jamesbond@example.com"},  
    {"id":"...","title":"...","status":"todo","priority":"low","assigned\_to":"jamesbond@example.com"}  
  \],  
  "summary": {"total\_assigned\_tasks": 3},  
  "cache": {"hit": false}  
}

Calling the same endpoint again should return cache.hit \= true.

# **Minimum Data Model**

* User  
* Task  
* LoginChallenge or TwoFactorChallenge  
* EmailLog or equivalent development email record

A user should include: id, full\_name, email, hashed\_password, role, created\_at, updated\_at.

A task should include: id, title, description, status, priority, created\_by\_id, assigned\_to\_id, created\_at, updated\_at.

# **Business Rules**

* Users have roles: admin or staff.  
* Only Admin can create tasks.  
* Only Admin can assign tasks.  
* James Bond must not be able to create tasks.  
* James Bond must only see tasks assigned to him.  
* Exactly 5 tasks must be created during the validation flow.  
* Exactly 3 tasks must be assigned to James Bond.  
* The view-my-tasks response must be generated from the database, not hardcoded.  
* The frontend must render task data from the Rust API, not from local hard-coded arrays.

# **Two-Factor Authentication Requirement**

Login must use email-based two-factor authentication. The first login request should not return a JWT immediately. It should create a challenge, generate a one-time code, and trigger an email event.

* Verification codes expire after 5 minutes.  
* Verification codes can only be used once.  
* Incorrect and expired codes must be rejected.  
* A JWT is issued only after successful verification.  
* Real email delivery is optional. SMTP, Mailtrap, console logging, or an email\_logs table are acceptable.  
* If persisted, the verification code should not be stored in plain text.

# **Caching Requirement**

* The GET /tasks/view-my-tasks endpoint must use caching per user.  
* The first request should load from the database and return cache.hit \= false.  
* The second identical request should return from cache and return cache.hit \= true.  
* When tasks are assigned or updated, the affected user task cache must be invalidated.  
* Redis is preferred. In-memory cache is acceptable if the limitation is documented.

# **Rust Technical Requirements**

* Use Rust stable edition 2021 or later.  
* Use Axum or Actix Web, or another suitable Rust web framework.  
* Use SQLx, SeaORM, Diesel, or equivalent.  
* PostgreSQL preferred; SQLite acceptable for local development.  
* Use JWT authentication.  
* Use Argon2 or bcrypt password hashing.  
* Use Serde for request/response models.  
* Use Tokio async runtime.  
* Use migrations with sqlx-cli, refinery, Diesel migrations, or equivalent.  
* Use cargo test plus integration tests where appropriate.  
* Optional: Docker Compose, Redis, OpenAPI generation, tracing/logging, rustfmt, clippy, GitHub Actions.

# **Frontend Technical Requirements**

* Use React, Next.js, Vue, or another modern frontend framework.  
* Use reusable components and separate API/service logic from UI components.  
* Handle authentication state and attach the JWT to protected API requests.  
* Handle loading, empty, success, and error states.  
* No advanced design system is required; a simple responsive interface is enough.

# **Testing Expectations**

* Admin and James Bond can be created.  
* Login creates a 2FA challenge and does not immediately return a JWT.  
* Correct 2FA code returns a JWT.  
* Incorrect, expired, or reused 2FA codes are rejected.  
* Admin can create 5 tasks.  
* Admin can assign exactly 3 tasks to James Bond.  
* James Bond cannot create a task.  
* James Bond can view exactly 3 assigned tasks.  
* Calling view-my-tasks twice shows cache.hit false, then true.  
* Task assignment/update invalidates the affected cache.  
* The frontend can complete login/2FA and display the API-driven task list.  
* The frontend displays appropriate loading/error states.

# **Submission Requirements**

* GitHub repository link containing frontend and backend projects.  
* README.md with frontend setup, backend setup, migration, run, seed, validation, and test instructions.  
* AI\_USAGE.md explaining which AI tools were used and what was manually changed.  
* .env.example for required environment variables.  
* Application code, migrations, and tests.  
* The final GET /tasks/view-my-tasks response pasted into the README.  
* 2-3 screenshots showing the running frontend and relevant code.

# **Assumptions Candidates Can Make**

* This is a local development assignment, not a deployed production service.  
* A simple seed endpoint or seed script is acceptable for creating Admin and James Bond.  
* Real email delivery is not required if email events can be validated locally.  
* In-memory caching is acceptable if Redis is not used, but it must be documented.  
* Task statuses and priorities can be simple enums or constrained strings.  
* The frontend can be simple and functional; visual design is not the primary evaluation area.  
* The API can use UUIDs or integer IDs.  
* The exact folder structure is up to the candidate, as long as it is clean and maintainable.  
* It is acceptable to make small route-name changes if the README clearly documents the validation workflow.

# **Evaluation Criteria**

| Skill | Rating | Criteria |
| :---- | :---: | :---- |
| Rust Backend & API | 25 | Correct API behavior, async handling, clean endpoints, validation and error handling. |
| Authentication & Security | 15 | 2FA flow, JWT handling, password hashing and role-based permissions. |
| Frontend / Full Stack Integration | 20 | Frontend consumes the Rust API, supports auth flow and renders task data correctly. |
| Data Handling & Caching | 15 | Database-backed models, DTOs, cache hit behavior and invalidation. |
| Code Quality | 15 | Idiomatic Rust, modular frontend/backend code, readable architecture and naming. |
| Testing & Explainability | 10 | Core workflow tests plus ability to explain design and AI usage. |
| **Total** | **100** |  |

