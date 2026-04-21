# ADR-0002 — Atomic `tasks` Table with Extensible Satellites

- **Status:** Accepted
- **Date:** 2026-04-20

## Context

Personal backlogs grow in unforeseen dimensions: estimations, area/service, effort, plan anchors, PR links, external references. Modeling everything as columns in `tasks` forces a migration for every new idea. Modeling everything as EAV (Entity-Attribute-Value) from the start dilutes the performance of common queries and complicates filtering.

The user's explicit requirement: evolutionary architecture — **start simple, allow extension without modification**.

## Decision

`tasks` is the central table with the minimum set every task has: `id`, `project_id`, `title`, `body`, `status`, `priority`, `type`, `parent_id`, `archived_at`, `completed_at`, `created_at`, `updated_at`.

Every dimension beyond this core goes into satellite tables:

- **`tags` + `task_tags`** — free labeling per tenant.
- **`task_attributes(task_id, key, value)`** — EAV for ad-hoc fields (estimates, service, area, external references).
- **`task_links(from_id, to_id, kind)`** — typed relationships (`blocks`, `relates`, `duplicates`, `spawned-from-plan`).
- **`task_events(task_id, ts, kind, payload)`** — append-only log of changes (status change, tag added, AI suggested, etc.). `payload` is `TEXT` holding **serialized JSON** (schema is free per `kind`), never an arbitrary string; consumers can inspect it via SQLite's `json_extract()` without needing to promote dimensions.

Promotion Rule: when a key in `task_attributes` appears in ≥80% of active tasks, or becomes a recurring filter criterion, it is migrated to a column in `tasks` via a new migration. Promotion is a conscious decision, not an automatic one.

## Consequences

**Positive:**
- Adding a new dimension is zero-migration (new key in `task_attributes`).
- A small core keeps common queries (`list`, `show`) fast and indices simple.
- `task_events` provides auditing and a foundation for future metrics (lead time, throughput) without retrofitting.
- Independent satellites evolve at their own pace.

**Negative:**
- EAV penalizes queries that filter by rare keys (scan of `task_attributes`). Mitigation: create a partial index when it hurts; promote to a column when it proves its weight.
- More joins in queries that need to gather everything (accepted — `show` performs 4-5 joins, runs locally on SQLite and responds in ms).
- Two extension mechanisms (attributes vs. tags) can be confusing — **rule of thumb:** tags are free categorical filters; attributes are key-value pairs. Document in the README.

## Alternatives Considered

- **JSON in a single column** (`tasks.metadata JSON`) — rejected: SQLite supports JSON1 but indexing is fragile; pure EAV provides better diagnostics and future migration.
- **Strict schema-first (every dimension becomes a column)** — rejected: violates the evolutionary premise; every new idea costs a migration + release.
- **Document store (e.g., file per task)** — rejected: loses relational queries and the entire point of SQLite.

## Related

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — all satellites inherit `project_id` via `task_id`.
- [ADR-0003 — Inline Migrations](0003-migrations-inline.md) — EAV → column promotion is a new migration.
