# ADR-0001 — Strict Project-Based Tenancy

- **Status:** Accepted
- **Date:** 2026-04-20
- **Author:** Cleidson Oliveira

## Context

`local-backlog` maintains a single SQLite database in `~/.local-backlog/backlog.db` aggregating N of the user's projects. Two approaches were possible:

1. **Shared database with optional filters** — commands by default show the current project, but flags like `--all-projects` allow an aggregated view.
2. **Strict tenancy** — the project inferred from the CWD (Current Working Directory) is the only scope visible in every data operation. There is no surface that aggregates tasks/tags/links across projects.

Option 1 seems pragmatic but introduces persistent bug classes: `backlog list` in one repo showing tasks from another, tags colliding between projects, accidental linking between unrelated tasks, and AI receiving leaked context during export.

## Decision

Adopt strict project-based tenancy:

- Every query for task, tag, attribute, link, and event includes `project_id = :current` inferred via CWD → registry.
- `tags.(project_id, name)` is unique; `#auth` in two different projects does not collide or share a record.
- Parent/child, task/tag, and task/task link relationships must stay inside the same `project_id` — enforced by SQL triggers on both insert and update.
- **There is no `--all-projects` flag** in data commands (`list`, `show`, `export`, etc.).
- The only cross-tenant surface is the meta namespace `backlog projects ...` (list, show, archive, relink). This namespace never exposes task/tag content — only registry metadata.
- `backlog doctor` checks for inconsistencies (orphan tasks, cross-project parents/tags/links) as part of the health check.

## Consequences

**Positive:**
- Impossible to leak data from one project to another due to flag errors.
- Tags have a natural namespace per tenant — trivial name reuse (e.g., `#bug`, `#auth`) without extra configuration.
- AI context export is secure by design: the emitted JSON/Markdown contains only the current tenant.
- Mental model aligns with Git: each repo is its own universe.

**Negative:**
- No aggregated "everything I have pending" view. Mitigation: `backlog projects list` shows counters per project; cross-tenant dashboards are out of scope (users can use SQL directly on the `.db` if an exceptional report is needed).
- No native mechanism for two different projects that want to share context (e.g., related microservices). Accepted workaround: model them as a single project using `#service-a` and `#service-b` tags.
- SQL triggers add testing surface (migrations must validate that triggers block invalid inserts and updates).

## Alternatives Considered

- **`--all-projects` as an opt-in flag** — rejected: introduces cross-tenant mode into the public surface, and a well-intentioned flag in a script becomes a permanent leak.
- **Per-project database in `<repo>/.local-backlog.db`** — rejected: breaks the "portable tool, one `cargo install`, zero clutter in the repo" premise; users would lose history if they forgot to add it to `.gitignore`.
- **Applying filters only at the application layer, without triggers** — rejected: a query bug that forgets the `WHERE project_id` silently breaks tenancy. Triggers are the defense-in-depth.

## Related

- [ADR-0005 — Global Registry](0005-registry-global.md) defines how the tenant is resolved from the CWD.
