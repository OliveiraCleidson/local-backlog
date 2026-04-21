# ADR-0000 — Rust as the Implementation Language with a Strategic Learning Focus

- **Status:** Accepted
- **Date:** 2026-04-20
- **Author:** Cleidson Oliveira

## Context

`local-backlog` was created with an explicit dual purpose:

1. **A real tool:** A CLI the author will use across every personal repository to manage the backlog, port between machines via `cargo install`, and consume through AI agents.
2. **A learning vehicle:** Rust is a long-term bet in the author's professional trajectory (Staff/Distinguished Engineer anchor, 2–3 years), alongside Go, AWS, and AI. A small, well-bounded scope in a known domain provides the ideal environment to learn the language without external deadline pressure.

The alternatives evaluated were Go (single binary, already familiar, fast iteration) and Python (rapid prototyping, but weaker distribution). While Go would allow for faster shipping and Python would offer the least friction, neither would help bridge the identified gap in Rust expertise.

## Decision

Rust is the implementation language for `local-backlog`. This decision has three operational implications:

1. **The project's pace prioritizes understanding over delivery.** There is no external deadline. If a feature requires a deep dive into lifetimes, generic traits, or asynchronous programming, the time spent is considered a valuable part of the project's return rather than a detour.
2. **Crate choices may favor learning value** when two options offer equivalent capabilities. For example: preferring `figment` over `config-rs` to explore provider concepts, or choosing the hybrid `thiserror` + `miette` over `anyhow` to learn how to design typed errors.
3. **AI-generated or AI-assisted code must undergo an explanatory human review.** The goal is not merely to have functional code; the author must understand every non-trivial block. Code reviews (whether human or AI-assisted via Codex) must explain any opaque idioms rather than simply providing approval.

This decision is foundational; every subsequent ADR (0001–NNNN) assumes Rust as the implementation language.

## Consequences

**Positive:**
- This small project serves as a controlled training ground, covering core ecosystem idioms such as `rusqlite`, migrations, triggers, Serde, typed error handling, and CLI parsing.
- The return on effort is multifaceted: it results in a useful personal tool, deeper Rust knowledge, and a public portfolio piece aligned with the author's declared career anchor.
- A closed scope avoids the "learning Rust in critical production" anti-pattern.

**Negative:**
- Initial development velocity may be lower than it would be with Go or Python. Mitigation: There is no strict deadline, and this slower pace is an intentional trade-off.
- The learning curve may lead to architectural decisions that a senior Rust engineer might later reconsider. Mitigation: ADRs 0001–NNNN define the architectural boundaries (tenancy, schema, output contract); within those boundaries, iteration and rewriting are encouraged.
- **Pedagogical over-engineering is acceptable** within these boundaries—for instance, implementing a custom trait for `Output` when an `enum` would suffice, provided it facilitates learning trait design. However, it is not acceptable to compromise tenancy or the output contract simply for the sake of syntactic experimentation.

## Alternatives Considered

- **Go** (Rejected): It would reuse existing knowledge without addressing the identified strategic gap. While it would allow for faster tool delivery, the opportunity cost would be the loss of potential learning.
- **Python** (Rejected): Although it allows for rapid prototyping, its distribution model (PyPI/pipx) is inferior to `cargo install` for the intended use case, and it offers no strategic gain from learning a new language.
- **TypeScript/Node** (Rejected): For the same reasons as Python, with the added drawback of requiring an external runtime.

## Related

- [ADR-0001 — Strict Tenancy](0001-tenancy-estrita-por-projeto.md) — first architectural decision that takes Rust as given.
- [ADR-0003 — Inline Migrations](0003-migrations-inline.md) — depends on Rust ecosystem features (`include_str!`, `rusqlite_migration`).
