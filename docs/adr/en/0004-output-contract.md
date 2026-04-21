# ADR-0004 — stdout/stderr Contract and Universal `--format`

- **Status:** Accepted
- **Date:** 2026-04-20

## Context

A CLI used in pipelines requires strict I/O discipline. Once users or scripts begin to rely on specific behaviors, two types of bugs become particularly difficult to resolve:

1. **Mixed Channels:** Printing logs or progress updates to `stdout` can interfere with commands like `backlog list | grep X`.
2. **Single Format:** Output intended for humans, such as colored tables, is unparseable by other tools. This can lead to scripts using regular expressions on ANSI codes, which may then be broken by any aesthetic changes.

A key project requirement is to ensure that output can be consumed by AI agents without needing to re-architect the system later. This necessitates stable JSON output from the very beginning.

## Decision

### Channel Contract

- **`stdout`**: Reserved exclusively for command data, such as tables, JSON, or TSV; no other information should be sent here.
- **`stderr`**: Used for all non-data information, including logs (`tracing`), progress updates, interactive prompts (`inquire`), and error messages (`miette`).
- This is implemented through the `src/output.rs` module, which provides `stdout_data()` and `stderr_msg()` helper functions. Direct `println!` calls are prohibited in subcommands and are caught during linting and code reviews.
- The `tracing-subscriber` is configured to output to `stderr`.
- The `is-terminal` library is used to detect if `stdout` is a TTY; if it is not, ANSI colors are automatically disabled.

### Universal `--format`

Every command that reads data, such as `list`, `show`, `export`, and `projects list`, supports the `--format` option:

- `table`: The default format, which is interactive, colored, and designed for human readers.
- `json`: A stable and documented format intended for scripts and AI agents.
- Future additions may include TSV, YAML, and Markdown formats. Each will be added without disrupting the existing table or JSON formats.

The JSON schema adheres to `snake_case` conventions and includes a `schema_version` in the envelope to allow for controlled evolution:

```json
{
  "schema_version": 1,
  "data": [ ... ]
}
```

## Consequences

**Positive:**
- Commands such as `backlog list | jq` and `backlog export --format=json` are functional from the start.
- AI agents can consume stable JSON, ensuring that visual updates to the table format do not affect their operation.
- Verbose logs, enabled with `-vv`, will not disrupt data pipes.
- Error messages generated via `miette` remain visible in the terminal, even if `stdout` is being redirected or captured.

**Negative:**
- Supporting multiple formats increases the workload for each command. Mitigation: Use centralized renderers in `src/format.rs`; subcommands will produce a `Vec<Struct>` and pass it to the renderer.
- Modifying the JSON schema requires careful management, including incrementing the `schema_version` and creating a new ADR for any breaking changes.
- Developers who are used to using `println!` must learn this new approach. Mitigation: This rule is documented in the project's `CLAUDE.md` and enforced through code reviews.

## Alternatives Considered

- **Using only a `--json` boolean flag** (Rejected): This would necessitate new flags for formats like TSV or Markdown, whereas `--format=X` is easily extensible.
- **Setting JSON as the default and using a flag for tables** (Rejected): This would degrade the interactive user experience for human users, who are the primary audience.
- **Omitting the separation of `stderr` and `stdout`** (Rejected): The cost of addressing this issue later would be prohibitively high.

## Related

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — output never leaks between tenants.
