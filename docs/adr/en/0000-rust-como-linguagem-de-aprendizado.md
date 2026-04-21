# ADR-0000 — Rust as the Implementation Language with a Strategic Learning Focus

- **Status:** Accepted
- **Date:** 2026-04-20
- **Author:** Cleidson Oliveira

## Context

`local-backlog` is born with an explicit dual purpose:

1. **A real tool:** a CLI the author will use across every personal repository to manage the backlog, port between machines via `cargo install`, and consume through AI agents.
2. **A learning vehicle:** Rust is a long-term bet in the author's public trajectory (Staff/Distinguished Engineer anchor, 2–3 years), alongside Go/AWS/AI. A small, well-bounded scope in a known domain is the ideal environment to learn the language without external deadline pressure.

The alternatives evaluated were Go (single binary, already familiar, iteration speed) and Python (fast prototype, weaker distribution). Go would ship faster; Python would offer the least friction. Neither contributes to the stated depth gap in Rust.

## Decision

Rust is the implementation language for `local-backlog`. The decision has three operational implications:

1. **Pace favors understanding over delivery.** There is no external deadline. If a feature demands understanding lifetimes, generic traits, or detailed `async`, the time spent is part of the project's return — not a detour.
2. **Crate choices may favor learning value** when two options are equivalent in capability. Examples: prefer `figment` over `config-rs` for exposing provider concepts; prefer the hybrid `thiserror`+`miette` over `anyhow` to teach the design of typed errors.
3. **AI-generated or AI-assisted code goes through explanatory human review.** The goal is not just to work — the author must understand every non-trivial block. Code review (human or adversarial via Codex) must explain idioms when opaque, not merely approve.

This decision is foundational — every other ADR (0001–NNNN) assumes Rust as a given.

## Consequences

**Positive:**
- A small project becomes a controlled training ground: atomic `tasks` + satellites covers `rusqlite`, migrations, triggers, serde, typed error handling, CLI parsing — the core idioms of the ecosystem.
- Return on effort compounds: useful personal tool + Rust depth + public portfolio piece aligned with the stated anchor.
- Closed scope avoids the "learn Rust in critical production" anti-pattern.

**Negative:**
- Initial velocity lower than Go/Python. Mitigation: no deadline; the cost is accepted by design.
- The learning curve may produce decisions a senior Rust engineer would revisit later. Mitigation: ADRs 0001–NNNN fix the architectural boundaries (tenancy, schema, output contract); within those boundaries, iteration and rewriting are welcome.
- **Pedagogical over-engineering is acceptable** within the boundaries: implementing a custom trait for `Output` when an `enum` would suffice, if it teaches trait design. Not acceptable outside the boundaries: breaking tenancy or the output contract out of syntactic curiosity.

## Alternatives Considered

- **Go** — rejected: it would reuse existing knowledge without contributing to the stated strategic gap. It would ship the tool faster, but the opportunity cost is the learning that would not happen.
- **Python** — rejected: fast prototype, but distribution (PyPI/pipx) is inferior to `cargo install` for the intended usage model; and zero strategic gain from a new language.
- **TypeScript/Node** — rejected for the same reasons as Python, with the aggravating factor of a mandatory external runtime.

## Related

- [ADR-0001 — Strict Tenancy](0001-tenancy-estrita-por-projeto.md) — first architectural decision that takes Rust as given.
- [ADR-0003 — Inline Migrations](0003-migrations-inline.md) — depends on Rust ecosystem features (`include_str!`, `rusqlite_migration`).
