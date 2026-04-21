//! Links entre tasks dentro de um mesmo tenant. Triggers do schema já
//! bloqueiam cross-tenant; este módulo valida antes para mensagem uniforme.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::db::repo::task_repo;
use crate::error::BacklogError;

#[derive(Debug, Clone, Serialize)]
pub struct Link {
    pub from_id: i64,
    pub to_id: i64,
    pub kind: String,
}

pub fn add(
    conn: &Connection,
    project_id: i64,
    from_id: i64,
    to_id: i64,
    kind: &str,
) -> Result<bool, BacklogError> {
    // Tenant-leak policy: ambos devem existir no tenant.
    if !task_repo::exists(conn, project_id, from_id)? {
        return Err(BacklogError::TaskNotFound { id: from_id });
    }
    if !task_repo::exists(conn, project_id, to_id)? {
        return Err(BacklogError::TaskNotFound { id: to_id });
    }
    let n = conn.execute(
        "INSERT OR IGNORE INTO task_links (from_id, to_id, kind) VALUES (?1, ?2, ?3)",
        params![from_id, to_id, kind],
    )?;
    Ok(n > 0)
}

pub fn remove(
    conn: &Connection,
    project_id: i64,
    from_id: i64,
    to_id: i64,
    kind: &str,
) -> Result<bool, BacklogError> {
    // Só valida origem para mensagem uniforme — destino pode já ter sido removido.
    if !task_repo::exists(conn, project_id, from_id)? {
        return Err(BacklogError::TaskNotFound { id: from_id });
    }
    let n = conn.execute(
        "DELETE FROM task_links WHERE from_id = ?1 AND to_id = ?2 AND kind = ?3 \
         AND EXISTS (SELECT 1 FROM tasks WHERE id = ?1 AND project_id = ?4)",
        params![from_id, to_id, kind, project_id],
    )?;
    Ok(n > 0)
}

pub fn exists(
    conn: &Connection,
    project_id: i64,
    from_id: i64,
    to_id: i64,
    kind: &str,
) -> Result<bool, BacklogError> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM task_links l JOIN tasks t ON t.id = l.from_id \
             WHERE l.from_id = ?1 AND l.to_id = ?2 AND l.kind = ?3 AND t.project_id = ?4",
            params![from_id, to_id, kind, project_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}
