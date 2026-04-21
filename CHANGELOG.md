# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Scaffolded the Rust CLI crate with `clap`, `clap-verbosity-flag`, `tracing`, `miette`, `thiserror`, `figment`, `rusqlite`, and `rusqlite_migration`.
- Added optimized release profile settings for a small single-binary CLI.
- Added `justfile` tasks for `check`, `test`, `build`, `install`, `bacon`, `review`, and `pre-commit`.
- Added strict output helpers so data goes to stdout and logs, prompts, progress, and diagnostics go to stderr.
- Added table and JSON renderers with schema-versioned JSON envelopes.
- Added global bootstrap for `~/.local-backlog/`, including `config.toml`, `registry.toml`, and `backlog.db`.
- Added configuration loading from defaults, global config, per-repository `.local-backlog.toml`, and environment variables.
- Added SQLite migration `0001_initial.sql` with `projects`, `tasks`, `tags`, `task_tags`, `task_attributes`, `task_links`, and `task_events`.
- Added inline migrations embedded in the binary via `include_str!`.
- Added schema snapshot tests with `insta`.
- Added tenancy triggers for parent tasks, task tags, and task links.
- Added immutable `project_id` triggers for `tasks` and `tags`.
- Added project registry resolution from current working directory with canonical paths and most-specific ancestor matching.
- Added `backlog init` for registering the current project.
- Added `backlog add` for creating tasks with title, body, type, priority, tags, parent, and status.
- Added `backlog list` with status, tag, type, priority, parent, archived, limit, order, table, and JSON options.
- Added `backlog show` with task details, tags, attributes, links, and recent events.
- Added `backlog done` and `backlog archive` task transitions.
- Added `backlog projects list`, `show`, `relink`, `archive`, and `archive --restore` as the only cross-tenant administration surface.
- Added task priority as `INTEGER NOT NULL DEFAULT 100` with configurable `priority.order` (`asc` or `desc`).
- Added event emission for task creation, status changes, archiving, field changes, tag changes, link changes, and attribute changes.
- Added `backlog edit` for updating title, body, status, priority, type, and parent fields.
- Added `backlog tag add`, `remove`, and `list`, including tenant-wide tag usage counts.
- Added `backlog link` for adding and removing task relationships with configured link kinds.
- Added `backlog attr set`, `unset`, and `list` for EAV-style task attributes.
- Added `backlog events` for reading task event timelines with kind and limit filters.
- Added `backlog export --format=markdown` for LLM-friendly project context grouped by status.
- Added `backlog export --format=json` for structured project context with project metadata, tasks, tags, attributes, links, and optional events.
- Added export filters for status, tag, type, archive inclusion, body inclusion, events inclusion, and updated-at date ranges.
- Added `backlog doctor` for registry, database, migration, tenancy, and orphan diagnostics.
- Added `backlog doctor --fix --yes` for non-interactive cleanup of fixable registry entries.
- Added doctor diagnostics for corrupt registry TOML, missing paths, root path divergence, user version drift, cross-tenant rows, and orphan rows.
- Added active task counts to project list and project show output.
- Added list table truncation for long task titles.
- Added deterministic ordering for list and export output.
- Added top-level export JSON envelope with `schema_version`, `project`, and `tasks`.
- Added default export filtering that hides `done` and `cancelled` tasks unless requested explicitly.
- Added full-day handling for date-only `export --since` and `--until` filters.
- Added consistent `--include-body` behavior for both markdown and JSON export output.
- Added stderr help output for no-subcommand usage while exiting successfully.
- Added tolerant registry loading so `doctor` can run with a corrupt `registry.toml`.
- Added idempotent duplicate tag attach and detach behavior without duplicate event emission.
- Added consistent link removal arguments and link event payloads.
- Added GitHub Actions CI for formatting, clippy, and tests on Ubuntu and macOS.
- Added optional pre-commit hook documentation.
- Added README documentation for project status, strict tenancy, AI export integration, JSON shape, markdown shape, event schema, and ADR links.
- Added `backlog completions <shell>` for emitting shell completion scripts (bash, zsh, fish, powershell, elvish) to stdout.
- Added README sections for install, quickstart, commands, and shell completion installation snippets.
- Added integration tests for all implemented command families.
- Added real SQLite in-memory database tests using the same migrations as production.

### Security

- Enforced strict project tenancy across task, tag, attribute, link, and event data commands.
- Enforced tenant scoping for task repository reads and mutations.
- Enforced tenant scoping for tag attach and detach operations.
- Ensured cross-tenant task references surface as `TaskNotFound` instead of leaking another project.
- Ensured `attr list` treats cross-tenant task IDs the same as missing task IDs.
- Kept cross-tenant access limited to `backlog projects ...` metadata commands.
- Added database triggers as defense in depth against cross-project relationships.
- Added diagnostic checks for cross-tenant data corruption in `backlog doctor`.
