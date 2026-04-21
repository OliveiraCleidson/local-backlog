# local-backlog

<p align="center">
  <img src="docs/assets/logo.webp" alt="local-backlog logo" width="280">
</p>

A personal, project-scoped backlog CLI backed by a single local SQLite database.

`local-backlog` is designed to live on every machine where its author codes. You install it once with `cargo install`, and from then on every repository you enter becomes its own isolated backlog: tasks, tags, attributes, links, and events — all tenant-isolated by project. No cloud, no sync, no server. Just a binary and a `.db` file under `~/.local-backlog/`.

## Why

When developing solo or with AI agents across many repositories, notes and plans tend to scatter across JSON, Markdown, and chat transcripts. Re-hydrating that context into a prompt wastes tokens. A SQLite database gives structure (priority, type, tags, technical debt) and lets agents consume focused, filterable context by project.

## Portability as a first-class goal

The tool is meant to be **copied between machines and used across every repository the author owns**. Two consequences:

- A single binary, static where possible, installable via `cargo install` with no external runtime.
- A single global location (`~/.local-backlog/`) holding the database, the project registry, and configuration — easy to back up or sync via dotfiles, easy to reason about.

When this project is stable, `cargo install local-backlog` on a new machine plus `backlog init` inside each repository is all that is required.

## Strict tenancy

Every repository is its own tenant. Commands like `backlog list`, `backlog add`, `backlog tag` only ever see data belonging to the project inferred from the current working directory. There is no `--all-projects` escape hatch in data commands; the only cross-tenant surface is `backlog projects ...`, which operates on metadata, never on task content.

This removes an entire class of mistakes: tasks leaking between repos, tag collisions, AI context accidentally mixing projects.

## Status

Pre-execution. Architectural decisions are captured under [`docs/adr/`](docs/adr/); implementation has not started yet.

The foundational choice of Rust as the implementation language is documented in [ADR-0000](docs/adr/pt-BR/0000-rust-como-linguagem-de-aprendizado.md) (canonical in `pt-BR`; translations available in `en` and `es-AR`).

## AI integration

`backlog export` is the single seam for feeding project context to an LLM. It dumps the current tenant — and only the current tenant — in one of two shapes:

```sh
# Human- and LLM-friendly outline grouped by status
backlog export --format markdown

# Full structured dump for programmatic consumers
backlog export --format json
```

Both formats support the same filters:

- `--status todo,doing` — restrict to one or more statuses
- `--tag infra,urgent` — restrict to tasks carrying any of the tags
- `--type feature,bug` — restrict to task types
- `--include-archived` — opt in to archived tasks (hidden by default)
- `--include-body` — append each task's body text
- `--include-events` — append each task's event timeline

### Markdown shape

Tasks are grouped under `## <status>` headers in the order declared in `config.toml::status.values`. Empty statuses are omitted. Each task renders as:

```
- T-42 [50] (feature) refactor auth middleware #security #debt
  > Optional body, one line per original line.
  - attrs: `jira=ABC-123` `estimate.h=4`
  - links: blocks T-17, relates T-8
  - backlinks: T-99 relates
  - events:
    - 2026-04-20 12:34:56 `created` {"title":"...","type":"feature","priority":50}
```

The `T-` prefix, `[priority]` bracket, and `#hashtag` convention are stable: write your LLM prompts against them.

### JSON shape

The JSON export is wrapped in the standard envelope (`schema_version` + `data`) described in [ADR-0004](docs/adr/pt-BR/0004-output-contract.md):

```json
{
  "schema_version": 1,
  "data": {
    "project": { "id": 1, "name": "proj", "root_path": "...", "description": null, "archived_at": null },
    "tasks": [
      {
        "id": 42,
        "title": "refactor auth middleware",
        "status": "doing",
        "priority": 50,
        "type": "feature",
        "tags": ["security", "debt"],
        "attributes": [{ "key": "jira", "value": "ABC-123" }],
        "links_out": [{ "from_id": 42, "to_id": 17, "kind": "blocks" }],
        "links_in":  [{ "from_id": 99, "to_id": 42, "kind": "relates" }],
        "events": []
      }
    ]
  }
}
```

Ordering is deterministic (priority, then `updated_at`, then `id`), so two runs against an unchanged database produce byte-identical output — safe to diff or check into a snapshot.

### Event schema

Payload schemas per `kind` are documented in the appendix of [ADR-0002](docs/adr/pt-BR/0002-tasks-atomica-com-satelites.md). Consumers must tolerate unknown fields; new `kind`s are additive.

## Documentation

- `docs/adr/` — Architecture Decision Records, canonical in `pt-BR` with `en` and `es-AR` translations. Start new ADRs from `TEMPLATE.md`.

## License

[MIT](LICENSE) © 2026 Cleidson Oliveira.
