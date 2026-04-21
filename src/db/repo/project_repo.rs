//! Superfície cross-tenant (meta): administra a tabela `projects`.
//! Os demais repos só aceitam dados dentro de um `project_id` explícito.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::domain::Project;
use crate::error::BacklogError;

fn row_to_project(row: &Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get("id")?,
        name: row.get("name")?,
        root_path: row.get("root_path")?,
        description: row.get("description")?,
        archived_at: row.get("archived_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

const COLS: &str = "id, name, root_path, description, archived_at, created_at, updated_at";

pub fn insert(
    conn: &Connection,
    name: &str,
    root_path: &str,
    description: Option<&str>,
) -> Result<Project, BacklogError> {
    conn.execute(
        "INSERT INTO projects (name, root_path, description) VALUES (?1, ?2, ?3)",
        params![name, root_path, description],
    )?;
    let id = conn.last_insert_rowid();
    get_by_id(conn, id)?.ok_or_else(|| BacklogError::ProjectNotFound {
        name: name.to_string(),
    })
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Project>, BacklogError> {
    let mut stmt = conn.prepare(&format!("SELECT {COLS} FROM projects WHERE id = ?1"))?;
    let project = stmt.query_row(params![id], row_to_project).optional()?;
    Ok(project)
}

pub fn get_by_name(conn: &Connection, name: &str) -> Result<Option<Project>, BacklogError> {
    let mut stmt = conn.prepare(&format!("SELECT {COLS} FROM projects WHERE name = ?1"))?;
    let project = stmt.query_row(params![name], row_to_project).optional()?;
    Ok(project)
}

pub fn get_by_root_path(
    conn: &Connection,
    root_path: &str,
) -> Result<Option<Project>, BacklogError> {
    let mut stmt = conn.prepare(&format!("SELECT {COLS} FROM projects WHERE root_path = ?1"))?;
    let project = stmt
        .query_row(params![root_path], row_to_project)
        .optional()?;
    Ok(project)
}

pub fn list_all(conn: &Connection) -> Result<Vec<Project>, BacklogError> {
    let mut stmt = conn.prepare(&format!("SELECT {COLS} FROM projects ORDER BY id ASC"))?;
    let rows = stmt.query_map([], row_to_project)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Conta tasks ativas (não arquivadas) de um projeto.
pub fn count_active_tasks(conn: &Connection, project_id: i64) -> Result<i64, BacklogError> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND archived_at IS NULL",
        params![project_id],
        |r| r.get(0),
    )?;
    Ok(n)
}

pub fn update_root_path(conn: &Connection, id: i64, new_path: &str) -> Result<(), BacklogError> {
    conn.execute(
        "UPDATE projects SET root_path = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![new_path, id],
    )?;
    Ok(())
}

pub fn archive(conn: &Connection, id: i64) -> Result<(), BacklogError> {
    conn.execute(
        "UPDATE projects
            SET archived_at = datetime('now'),
                updated_at  = datetime('now')
          WHERE id = ?1 AND archived_at IS NULL",
        params![id],
    )?;
    Ok(())
}

pub fn restore(conn: &Connection, id: i64) -> Result<(), BacklogError> {
    conn.execute(
        "UPDATE projects
            SET archived_at = NULL,
                updated_at  = datetime('now')
          WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
