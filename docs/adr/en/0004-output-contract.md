# ADR-0004 — stdout/stderr Contract and Universal `--format`

- **Status:** Accepted
- **Date:** 2026-04-20

## Context

A CLI used in pipelines must have strict I/O discipline. Two types of bugs are extremely difficult to revert once users (or scripts) start depending on the behavior:

1. **Mixed Channels** — printing logs/progress to `stdout` breaks `backlog list | grep X`.
2. **Single Format** — human output (colored tables) is unparseable by other tools; scripts start regexing on ANSI codes, and any aesthetic change breaks consumers.

Explicit project requirement: consume output via AI agents without re-architecting later. This requires stable JSON from day one.

## Decision

### Channel Contract

- **`stdout`**: exclusively **data** from the command (table, JSON, tsv). Nothing else.
- **`stderr`**: everything that is not data — logs (`tracing`), progress, interactive prompts (`inquire`), and error messages (`miette`).
- Implemented via the `src/output.rs` module exposing `stdout_data()` / `stderr_msg()` helpers. No direct `println!` in subcommands — linting/review checks block it.
- `tracing-subscriber` configured to write to `stderr`.
- `is-terminal` detects if `stdout` is a TTY; if not, it automatically disables ANSI colors.

### Universal `--format`

Every read command (`list`, `show`, `export`, `projects list`) accepts `--format`:

- `table` — default interactive, colored, for humans.
- `json` — stable, documented, for scripts and AI agents.
- Possible future additions: `tsv`, `yaml`, `markdown`. Each is additive, without breaking `table`/`json`.

The JSON schema follows `snake_case` conventions and includes `schema_version` in the envelope for controlled evolution:

```json
{
  "schema_version": 1,
  "data": [ ... ]
}
```

## Consequences

**Positive:**
- `backlog list | jq` and `backlog export --format=json` work on day one.
- AI agents consume stable JSON; visual changes to `table` do not impact them.
- Verbose logs (`-vv`) never break pipes.
- Errors via `miette` remain visible in the terminal even when `stdout` is being captured.

**Negative:**
- Maintaining two formats is more work per command. Mitigation: centralized renderers in `src/format.rs`; subcommands produce `Vec<Struct>` and deliver it to the renderer.
- Evolving the JSON schema requires discipline (bump of `schema_version`, new ADR when breaking).
- Developers accustomed to `println!` need to learn the rule. Mitigation: document in the repo's `CLAUDE.md` and block in code reviews.

## Alternatives Considered

- **Only `--json` as a boolean flag** — rejected: prevents adding `tsv`/`markdown` without a new flag; `--format=X` is extensible.
- **JSON as default, table as a flag** — rejected: interactive UX suffers; human users are the primary consumers in daily use.
- **No stderr/stdout separation (casual use)** — rejected: the cost of fixing it later is extremely high.

## Related

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — output never leaks between tenants.
