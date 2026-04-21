# ADR-0003 — Inline Migrations via `rusqlite_migration`

- **Status:** Accepted
- **Date:** 2026-04-20

## Context

The CLI is a single binary distributed via `cargo install`. Schema migrations must:

1. Run automatically on first use and during upgrades.
2. Not depend on an external CLI (`diesel`, `sqlx`).
3. Not require loose SQL files on the user's filesystem.
4. Be testable against an in-memory database.

Options:

- `rusqlite_migration` inline (`const MIGRATIONS: &[M]`).
- `rusqlite_migration` per directory (`from-directory` feature).
- `refinery` with embedded SQL files via `include_str!`.
- Manual script executing PRAGMAs.

## Decision

Use `rusqlite_migration` in inline mode:

```rust
use rusqlite_migration::{Migrations, M};

const MIGRATIONS: &[M] = &[
    M::up(include_str!("../../migrations/0001_initial.sql")),
    // ...
];
```

The `.sql` files live in `migrations/` in the repo as a **human reference** (review, diff, snapshot) but are embedded into the binary via `include_str!`. The runtime source of truth is the constant slice.

The schema state is controlled by SQLite's `PRAGMA user_version` — no auxiliary table.

Migrations run automatically on every `backlog <any command>` via `Migrations::from_slice(...).to_latest(&mut conn)` during connection bootstrap.

An `insta` snapshot of the result of `SELECT type, name, sql FROM sqlite_master ORDER BY name` validates the final schema after applying all migrations.

## Consequences

**Positive:**
- Self-contained binary — the user never sees SQL files.
- Zero external CLI; binary upgrades apply the new schema transparently.
- Migration testing is trivial: `Connection::open_in_memory()` + `to_latest`.
- `insta` snapshot turns "I changed a migration" into a reviewable schema diff.

**Negative:**
- Already published migrations cannot be modified — a new adjusting migration is required. Rule: **a migration is immutable after release.** Schema changes must always be additive or compensatory.
- `include_str!` makes the .sql files become `&'static str` — very small at runtime, with negligible cost.

## Alternatives Considered

- **`from-directory`** — would require shipping files alongside the binary (breaks "single binary"); discarded.
- **`refinery`** — similar in capability, but `rusqlite_migration` is lighter and uses the native `user_version`, avoiding an auxiliary table.
- **Ad-hoc migrations via Rust code** — rejected: loses the declarative SQL contract that is easy to review in PRs.

## Related

- [ADR-0002 — Satellites](0002-tasks-atomica-com-satelites.md) — EAV → column promotion generates a new immutable migration.
