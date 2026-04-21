# ADR-0003 — Inline Migrations via `rusqlite_migration`

- **Status:** Accepted
- **Date:** 2026-04-20

## Context

The CLI is distributed as a single binary via `cargo install`. Schema migrations must meet several requirements:

1. They must run automatically upon first use and subsequent upgrades.
2. They must not depend on an external CLI, such as `diesel` or `sqlx`.
3. They must not require standalone SQL files on the user's filesystem.
4. They must be testable using an in-memory database.

Options:

- `rusqlite_migration` inline (`const MIGRATIONS: &[M]`).
- `rusqlite_migration` per directory (`from-directory` feature).
- `refinery` with embedded SQL files via `include_str!`.
- A manual script executing PRAGMAs.

## Decision

Use `rusqlite_migration` in inline mode:

```rust
use rusqlite_migration::{Migrations, M};

const MIGRATIONS: &[M] = &[
    M::up(include_str!("../../migrations/0001_initial.sql")),
    // ...
];
```

Although the `.sql` files are stored in the `migrations/` directory as a reference for reviews, diffs, and snapshots, they are embedded into the binary using `include_str!`. The runtime source of truth is the constant slice.

The schema's state is managed using SQLite's `PRAGMA user_version`, which eliminates the need for an auxiliary table.

Migrations are automatically executed with every `backlog` command during connection bootstrap using `Migrations::from_slice(...).to_latest(&mut conn)`.

An `insta` snapshot of the results from `SELECT type, name, sql FROM sqlite_master ORDER BY name` is used to validate the final schema after all migrations have been applied.

## Consequences

**Positive:**
- The binary is self-contained, so users never see the SQL files.
- No external CLI is required, as binary upgrades automatically and transparently apply the new schema.
- Testing migrations is straightforward: use `Connection::open_in_memory()` along with the `to_latest` function.
- Using an `insta` snapshot converts any changes to a migration into a reviewable schema diff.

**Negative:**
- Once a migration has been published, it cannot be modified; any adjustments must be handled by a new migration. Rule: **Migrations are immutable after release.** Schema changes must always be additive or compensatory.
- The use of `include_str!` converts `.sql` files into `&'static str` values, which are small and have a negligible impact on runtime performance.

## Alternatives Considered

- **The `from-directory` option** (Rejected): This would require shipping files alongside the binary, which violates the single-binary requirement.
- **The `refinery` tool** (Rejected): It offers similar capabilities, but `rusqlite_migration` was chosen because it is lighter and uses the native `user_version`, avoiding the need for an auxiliary table.
- **Ad-hoc migrations using Rust code** (Rejected): This sacrifices the declarative SQL contract, which is easier to review in pull requests.

## Related

- [ADR-0002 — Satellites](0002-tasks-atomica-com-satelites.md) — EAV → column promotion generates a new immutable migration.
