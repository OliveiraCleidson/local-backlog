# Architecture Decision Records

Architectural decision records for `local-backlog`. Short format, focused on **context**, **decision**, and **consequences**.

## Languages

- [English (en)](en/README.md)
- [Português (pt-BR)](pt-BR/README.md)
- [Español (es-AR)](es-AR/README.md)

## Template

New ADRs start from the template: [en](en/TEMPLATE.md) · [pt-BR](pt-BR/TEMPLATE.md) · [es-AR](es-AR/TEMPLATE.md).

## Index

| # | Title | en | pt-BR | es-AR |
|---|---|---|---|---|
| 0000 | Rust as the Implementation Language (Strategic Learning Focus) | [en](en/0000-rust-como-linguagem-de-aprendizado.md) | [pt-BR](pt-BR/0000-rust-como-linguagem-de-aprendizado.md) | [es-AR](es-AR/0000-rust-como-linguagem-de-aprendizado.md) |
| 0001 | Strict Project-Based Tenancy | [en](en/0001-tenancy-estrita-por-projeto.md) | [pt-BR](pt-BR/0001-tenancy-estrita-por-projeto.md) | [es-AR](es-AR/0001-tenancy-estrita-por-projeto.md) |
| 0002 | Atomic `tasks` Table with Extensible Satellites | [en](en/0002-tasks-atomica-com-satelites.md) | [pt-BR](pt-BR/0002-tasks-atomica-com-satelites.md) | [es-AR](es-AR/0002-tasks-atomica-com-satelites.md) |
| 0003 | Inline Migrations via `rusqlite_migration` | [en](en/0003-migrations-inline.md) | [pt-BR](pt-BR/0003-migrations-inline.md) | [es-AR](es-AR/0003-migrations-inline.md) |
| 0004 | stdout/stderr Contract and Universal `--format` | [en](en/0004-output-contract.md) | [pt-BR](pt-BR/0004-output-contract.md) | [es-AR](es-AR/0004-output-contract.md) |
| 0005 | Project Identification via Global Registry | [en](en/0005-registry-global.md) | [pt-BR](pt-BR/0005-registry-global.md) | [es-AR](es-AR/0005-registry-global.md) |

## Conventions

- **Filename:** `NNNN-slug-kebab-case.md`. The slug is stable across locales (Portuguese base) so that cross-locale links never break.
- **Status:** `Proposed` → `Accepted` → `Superseded by ADR-NNNN` / `Deprecated`.
- **One decision per ADR.** Related decisions reference each other; they never merge.
- **Immutability:** an ADR is immutable after `Accepted`. Changes generate a new ADR that supersedes the previous one.
- **Dates:** ISO 8601 (`YYYY-MM-DD`) in every locale. No locale-specific date formats.

## Canonical Language

**`pt-BR` is the canonical language.** It is the source of truth for every ADR.

- Every new ADR **starts in `pt-BR`**. `en` and `es-AR` are translations derived from it.
- Any change to the content of an ADR **must be applied first in `pt-BR`**, then propagated to `en` and `es-AR` in the same commit.
- A pull request that updates an ADR in only one locale is incomplete and must not be merged.
- When locales diverge, `pt-BR` wins. A divergence found later is fixed by aligning `en` and `es-AR` back to `pt-BR`.

Rationale: the project's primary author and decision context are Portuguese. Keeping the canonical source in `pt-BR` prevents translation drift from silently altering meaning. The other locales exist for public reach and personal brand, not as parallel specifications.
