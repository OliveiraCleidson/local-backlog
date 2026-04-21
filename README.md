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

## Documentation

- `docs/adr/` — Architecture Decision Records, canonical in `pt-BR` with `en` and `es-AR` translations. Start new ADRs from `TEMPLATE.md`.

## License

TBD.
