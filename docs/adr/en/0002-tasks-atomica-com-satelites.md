# ADR-0002 — Atomic `tasks` Table with Extensible Satellites

- **Status:** Accepted
- **Date:** 2026-04-20

## Context

Personal backlogs expand in unpredictable ways, incorporating estimations, areas, services, levels of effort, plan anchors, pull request links, and external references. Representing every new attribute as a column in the `tasks` table would necessitate a database migration for each addition. Conversely, using an Entity-Attribute-Value (EAV) model for everything from the outset would degrade query performance and complicate filtering.

The user's requirement is for an evolutionary architecture: start simply and allow for extensions without modifying the core structure.

## Decision

The `tasks` table is the central entity, containing the minimum set of attributes common to every task: `id`, `project_id`, `title`, `body`, `status`, `priority`, `type`, `parent_id`, `archived_at`, `completed_at`, `created_at`, and `updated_at`.

Additional attributes are stored in satellite tables:

- **`tags` + `task_tags`**: Provides flexible labeling for each tenant.
- **`task_attributes(task_id, key, value)`**: An EAV model for ad-hoc fields, such as estimates, services, areas, and external references.
- **`task_links(from_id, to_id, kind)`**: Defines typed relationships, including `blocks`, `relates`, `duplicates`, and `spawned-from-plan`.
- **`task_events(task_id, ts, kind, payload)`**: An append-only log of changes, such as status updates, tag additions, and AI-suggested modifications. The `payload` is a `TEXT` field containing serialized JSON with a kind-specific schema, rather than an arbitrary string. This allows users to inspect data using SQLite's `json_extract()` without requiring dimension promotion.

**Promotion Rule:** If a key in `task_attributes` appears in 80% or more of active tasks, or if it becomes a frequent filter criterion, it will be migrated to a column in the `tasks` table through a new migration. Promotion is an intentional decision, not an automatic process.

## Consequences

**Positive:**
- Introducing a new dimension does not require a migration, as it simply involves adding a new key to `task_attributes`.
- Maintaining a small core ensures that common queries, such as `list` and `show`, remain fast and that indices stay simple.
- The `task_events` table provides an audit trail and serves as a foundation for future metrics, such as lead time and throughput, without needing retrofits.
- Satellite tables can evolve independently.

**Negative:**
- The EAV model can penalize queries filtering by rare keys, as it requires scanning `task_attributes`. Mitigation: Create partial indices when performance issues arise, or promote the attribute to a column when its usage justifies the change.
- Queries that must aggregate all data will require more joins. This is acceptable, as the `show` command performs four to five joins and runs locally on SQLite with millisecond response times.
- Having two extension mechanisms (attributes and tags) could be confusing. As a rule of thumb: tags are flexible categorical filters, while attributes are key-value pairs. This distinction will be documented in the README.

## Alternatives Considered

- **Storing JSON in a single column (`tasks.metadata JSON`)** (Rejected): Although SQLite supports JSON1, indexing can be fragile. A pure EAV model provides superior diagnostics and simpler future migrations.
- **A strict, schema-first approach** (Rejected): This violates the evolutionary design premise, as every new idea would necessitate a migration and a new release.
- **A document-oriented store (e.g., a file per task)** (Rejected): This would sacrifice relational queries and the benefits of using SQLite.

## Appendix: `task_events` payload schema

Each event has a `kind` and a JSON `payload`. The table below documents the set emitted by the CLI in the current version. Consumers must tolerate unknown fields (forward-compat).

| `kind`           | Emitted by                          | Payload                                                      |
|------------------|-------------------------------------|--------------------------------------------------------------|
| `created`        | `backlog add`                       | `{ "title": string, "type": string\|null, "priority": integer }` |
| `status_changed` | `backlog done`                      | `{ "from": string, "to": string }`                           |
| `archived`       | `backlog archive`                   | `{}`                                                         |
| `field_changed`  | `backlog edit` (one event per changed field) | `{ "field": string, "from": any\|null, "to": any\|null }` |
| `tag_added`      | `backlog tag add`                   | `{ "tag": string }`                                          |
| `tag_removed`    | `backlog tag remove`                | `{ "tag": string }`                                          |
| `link_added`     | `backlog link ... --kind X`         | `{ "to": integer, "kind": string }`                          |
| `link_removed`   | `backlog link ... --kind X --remove`| `{ "to": integer, "kind": string }`                          |
| `attr_set`       | `backlog attr set`                  | `{ "key": string, "from": string\|null, "to": string }`      |
| `attr_unset`     | `backlog attr unset`                | `{ "key": string }`                                          |

Invariants:

- `payload` is always a JSON object (never a top-level scalar or array).
- `from`/`to` fields in `field_changed` and `attr_set` may be `null` (field cleared or previously absent).
- `kind` names are stable: changing one breaks external consumers and requires a new ADR superseding this one.
- New `kind`s are additive — `backlog events` ignores unknown kinds without error.

## Related

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — all satellites inherit `project_id` via `task_id`.
- [ADR-0003 — Inline Migrations](0003-migrations-inline.md) — EAV → column promotion is a new migration.
