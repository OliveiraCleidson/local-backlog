//! Resolve tenant (project) a partir da CWD via `Registry` + tabela `projects`.
//!
//! Regras:
//! - CWD é canonizada (symlinks resolvidos) antes do match.
//! - Match de maior profundidade vence quando múltiplos ancestrais existirem.
//! - Projeto arquivado → `ProjectArchived`.
//! - Nenhum match → `ProjectNotRegistered`.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db::repo::project_repo;
use crate::error::BacklogError;
use crate::registry::Registry;

#[derive(Debug, Clone)]
pub struct ResolvedTenant {
    pub project_id: i64,
    pub name: String,
    pub root_path: PathBuf,
}

/// Resolve o tenant correspondente a `cwd`. `conn` é a conexão ao banco
/// (para checar `archived_at` — registro TOML é apenas cache).
pub fn resolve(
    cwd: &Path,
    conn: &Connection,
    registry: &Registry,
) -> Result<ResolvedTenant, BacklogError> {
    let canonical = std::fs::canonicalize(cwd).map_err(|source| BacklogError::Io {
        path: cwd.to_path_buf(),
        source,
    })?;

    let entry =
        registry
            .find_ancestor(&canonical)
            .ok_or_else(|| BacklogError::ProjectNotRegistered {
                cwd: canonical.clone(),
            })?;

    let project =
        project_repo::get_by_id(conn, entry.id)?.ok_or_else(|| BacklogError::RegistryCorrupt {
            path: PathBuf::from(crate::registry::REGISTRY_FILE),
            reason: format!("registry aponta para project_id={} inexistente", entry.id),
        })?;

    if project.archived_at.is_some() {
        return Err(BacklogError::ProjectArchived {
            id: project.id,
            name: project.name,
        });
    }

    Ok(ResolvedTenant {
        project_id: project.id,
        name: project.name,
        root_path: entry.root_path.clone(),
    })
}
