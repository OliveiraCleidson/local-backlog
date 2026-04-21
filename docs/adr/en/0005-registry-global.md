# ADR-0005 — Project Identification via Global Registry

- **Status:** Accepted
- **Date:** 2026-04-20

## Context

Strict tenancy ([ADR-0001](0001-tenancy-estrita-por-projeto.md)) necessitates identifying the current project with every CLI invocation. Three strategies were evaluated:

1. **A `.local-backlog` file in the repository** containing a `project_id`: This can be version-controlled in Git, but it clutters the repository; if not committed, it will be lost.
2. **A hash of the Git repository path** (`git rev-parse --show-toplevel`): While this requires zero configuration, a repository cloned to a different machine would silently be treated as a new project, and it would not work outside of Git.
3. **A global registry** in `~/.local-backlog/registry.toml` that maps an `absolute_path` to a `project_id`: This approach avoids repository clutter, although it does require a `backlog relink` when folders are moved; it also makes listing projects straightforward.

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

The registry serves as a canonical mirror of the SQLite `projects` table; the database remains the source of truth, while the TOML file provides a fast lookup cache and a format for human inspection.

### Synchronization

Changes to project metadata are first applied to SQLite and then persisted to `registry.toml` by atomically rewriting the file:

- `backlog init` adds a project to the `projects` table and creates the corresponding registry entry.
- `backlog projects relink <id|name> --path <new>` updates the `projects.root_path` and regenerates the registry.
- `backlog projects archive <id|name>` sets `projects.archived_at` and updates the registry.
- The `backlog doctor` command compares the SQLite database with `registry.toml`, reporting any missing entries, stale or duplicate paths, and mismatched IDs.

### Resolution

For every command:

1. Canonicalize the Current Working Directory (CWD) by resolving symlinks and normalizing the path.
2. Search up the directory tree for a match within `root_path`.
3. If a match is found, use the corresponding `project_id`.
4. If no match is found, return an error via `miette` suggesting the use of `backlog init` or `backlog projects relink`.

### Meta Commands (The Only Cross-Tenant Namespace)

- `backlog projects list`: Displays all registered projects, including their IDs, names, paths, and task counts.
- `backlog projects show <id|name>`: Provides metadata for a specific project.
- `backlog projects relink <id|name> --path <new>`: Updates the `root_path` when a repository folder is moved.
- `backlog projects archive <id|name>`: Marks a project as archived; it will no longer appear in the `list`, but its data will be preserved.

## Consequences

**Positive:**
- No `local-backlog` files are stored within the repositories, ensuring that `git status` remains uncluttered.
- The system works outside of Git for projects that do not use version control.
- The `backlog projects list` command provides an immediate global inventory of all projects.
- When folders are moved, a simple `relink` command addresses the issue without any data loss.

**Negative:**
- Deleting or losing `~/.local-backlog/` will break the project links, as the database and registry are stored together. Mitigation: The entire directory is easy to back up, and users can choose to version-control it within their dotfiles.
- Two different checkouts of the same repository in different paths will be treated as distinct projects, which may be intentional or an error. Mitigation: `backlog init` identifies Git repositories and asks whether it's a new project or a duplicate; the `doctor` command flags paths with the same `origin` that are mapped to different IDs.
- Portability between machines requires synchronizing `~/.local-backlog/`, or otherwise accepting independent states on each machine. This is acceptable, as the tool is designed for use on a single machine.

## Alternatives Considered

- **Placing a `.local-backlog` file in the repository** (Rejected): This clutters repositories and requires `.gitignore` management or a commit containing a coupled ID.
- **Using a hash of the `git remote get-url origin`** (Rejected): This fails for repositories without an origin, in forks, and in non-Git repositories.
- **Using the directory name as a key** (Rejected): This would cause collisions if two repositories were both named `api/`.

## Related

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — the registry is the mechanism that delivers the tenant-id from the CWD.
