# In-memory cache for my tasks

The brief prefers Redis. My tasks are cached in process memory, one entry per User, with no expiry. Assignment and task updates drop the affected Users' entries. A shared cache is unnecessary for a single-process local API.

## Considered options

- Redis
- In-process memory
