# Architecture Decision Records

Architectural decision records for `local-backlog`. Short format, focused on **context**, **decision**, and **consequences**.

## Index

- [ADR-0000 — Rust as the Implementation Language with a Strategic Learning Focus](0000-rust-como-linguagem-de-aprendizado.md)
- [ADR-0001 — Strict Project-Based Tenancy](0001-tenancy-estrita-por-projeto.md)
- [ADR-0002 — Atomic `tasks` Table with Extensible Satellites](0002-tasks-atomica-com-satelites.md)
- [ADR-0003 — Inline Migrations via `rusqlite_migration`](0003-migrations-inline.md)
- [ADR-0004 — stdout/stderr Contract and Universal `--format`](0004-output-contract.md)
- [ADR-0005 — Project Identification via Global Registry](0005-registry-global.md)

Use [`TEMPLATE.md`](TEMPLATE.md) as the starting point for new ADRs.

## Conventions

- Filename: `NNNN-slug-kebab-case.md`.
- Status: `Proposed` → `Accepted` → `Superseded by ADR-NNNN` / `Deprecated`.
- One decision per ADR. Related decisions reference each other, they do not merge.
- An ADR is immutable after being `Accepted`. Changes generate a new ADR that supersedes the previous one.
