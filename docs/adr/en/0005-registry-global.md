# ADR-0005 — Project Identification via Global Registry

- **Status:** Accepted
- **Date:** 2026-04-20

## Context

Strict tenancy ([ADR-0001](0001-tenancy-estrita-por-projeto.md)) requires resolving "which project is this?" at every CLI invocation. Three strategies were considered:

1. **`.local-backlog` file in the repo** with a `project_id`. Versionable in git; clutters repositories; if not committed, it disappears.
2. **Hash of the git repo path** (`git rev-parse --show-toplevel`). Zero configuration; a clone on another machine silently becomes a "different project"; does not work outside of git.
3. **Global Registry** in `~/.local-backlog/registry.toml` mapping `absolute_path → project_id`. Zero clutter in the repo; requires `backlog relink` when moving folders; easy to list projects.

## Decision

Use option 3: a single global registry.

### Structure

```toml
# ~/.local-backlog/registry.toml
[[projects]]
id = 1
name = "local-backlog"
root_path = "/Users/cleidson/github/personal/local-backlog"

[[projects]]
id = 2
name = "hub"
root_path = "/Users/cleidson/github/personal/hub"
```

The registry is a canonical mirror of the SQLite `projects` table — the source of truth is the database; the TOML file exists for human inspection and as a fast lookup cache.

### Synchronization

Project metadata changes are applied to SQLite first and then persisted to `registry.toml` by rewriting the file atomically:

- `backlog init` inserts the project in `projects` and writes the corresponding registry entry.
- `backlog projects relink <id|name> --path <new>` updates `projects.root_path` and rewrites the registry.
- `backlog projects archive <id|name>` updates `projects.archived_at` and rewrites the registry.
- `backlog doctor` compares SQLite and `registry.toml`, reporting missing entries, stale paths, duplicate paths, and mismatched IDs.

### Resolution

For every command:

1. Canonicalize the CWD (resolve symlinks, normalize).
2. Traverse up the directory tree looking for a match in `root_path`.
3. If found → use the corresponding `project_id`.
4. If not found → error via `miette` suggesting `backlog init` or `backlog projects relink`.

### Meta Commands (The Only Cross-Tenant Namespace)

- `backlog projects list` — shows all registered projects (id, name, path, task count).
- `backlog projects show <id|name>` — metadata of a project.
- `backlog projects relink <id|name> --path <new>` — updates `root_path` when the repo moves folders.
- `backlog projects archive <id|name>` — marks a project as archived (does not appear in `list`); data is preserved.

## Consequences

**Positive:**
- No `local-backlog` files inside the repos — does not clutter `git status`.
- Works outside of git (projects without version control).
- `backlog projects list` provides an instant global inventory.
- Folder moves → a simple `relink` fixes it; no data loss.

**Negative:**
- Losing `~/.local-backlog/` breaks the links (database + registry live together). Mitigation: the entire folder is trivial to back up; the user can version it in their dotfiles if they wish.
- Two different checkouts of the same repo in different paths become distinct projects (this may be intended or an error). Mitigation: `backlog init` detects Git repos and asks if it's a new project or a duplicate; `doctor` flags paths with the same `origin` mapped to different IDs.
- Portability between machines requires synchronizing `~/.local-backlog/` (or accepting independent states per machine). Accepted — the tool is single-machine by design.

## Alternatives Considered

- **`.local-backlog` in the repo** — rejected: clutters the user's repositories, requires `.gitignore` discipline or a commit with a coupled ID.
- **Hash of `git remote get-url origin`** — rejected: fails in repos without an origin, in forks, and in non-git repos.
- **Directory name as key** — rejected: two repos named `api/` would collide.

## Related

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — the registry is the mechanism that delivers the tenant-id from the CWD.
