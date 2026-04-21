# ADR-0001 — Strict Project-Based Tenancy

- **Status:** Accepted
- **Date:** 2026-04-20
- **Author:** Cleidson Oliveira

## Context

`local-backlog` uses a single SQLite database, located at `~/.local-backlog/backlog.db`, to aggregate multiple user projects. Two approaches were considered:

1. **A shared database with optional filters:** Commands display the current project by default, but flags such as `--all-projects` enable an aggregated view.
2. **Strict tenancy:** The project inferred from the Current Working Directory (CWD) is the only scope visible in any data operation. No interface exists to aggregate tasks, tags, or links across projects.

While Option 1 may seem pragmatic, it introduces potential bugs such as `backlog list` displaying tasks from the wrong repository, tag collisions between projects, accidental linking of unrelated tasks, and AI models receiving unintended context during exports.

## Decision

Adopt strict project-based tenancy:

- Every query for tasks, tags, attributes, links, and events must include `project_id = :current`, which is inferred from the CWD via the registry.
- The combination of `(project_id, name)` in the `tags` table is unique; for example, an `#auth` tag in two different projects will neither collide nor share a record.
- Parent/child, task/tag, and task/task link relationships must remain within the same `project_id`, a constraint enforced by SQL triggers during both insertions and updates.
- **There is no `--all-projects` flag** in data commands (e.g., `list`, `show`, `export`).
- The only interface that spans multiple tenants is the `backlog projects ...` metadata namespace (e.g., `list`, `show`, `archive`, `relink`). This namespace never exposes task or tag content; it only provides registry metadata.
- The `backlog doctor` command identifies inconsistencies, such as orphaned tasks or cross-project parent-tag-link relationships, as part of its health check.

## Consequences

**Positive:**
- It is impossible for data to leak between projects due to flag errors.
- Tags have a natural namespace per tenant, allowing for simple name reuse (e.g., `#bug`, `#auth`) without extra configuration.
- AI context exports are secure by design, as the generated JSON or Markdown files contain data from only the current tenant.
- The mental model aligns with Git: each repository is its own universe.

**Negative:**
- There is no aggregated view for all pending tasks across all projects. Mitigation: The `backlog projects list` command provides task counters for each project. Cross-tenant dashboards are considered out of scope; users requiring specialized reports can query the SQLite database directly.
- There is no native mechanism for sharing context between two different projects (e.g., related microservices). The recommended workaround is to model them as a single project and use tags such as `#service-a` and `#service-b` to differentiate them.
- The use of SQL triggers increases the testing surface; migrations must verify that triggers correctly block invalid insertions and updates.

## Alternatives Considered

- **An `--all-projects` opt-in flag** (Rejected): This would introduce cross-tenancy to the public interface, and a well-intentioned flag used in a script could lead to permanent data leakage.
- **A per-project database located in `<repo>/.local-backlog.db`** (Rejected): This violates the premise of a portable, zero-clutter tool. Furthermore, users could lose their history if they fail to add the database file to their `.gitignore`.
- **Applying filters only at the application layer without using triggers** (Rejected): A query bug that omits the `WHERE project_id` clause would silently compromise tenancy. Triggers provide an essential layer of defense-in-depth.

## Related

- [ADR-0005 — Global Registry](0005-registry-global.md) defines how the tenant is resolved from the CWD.
