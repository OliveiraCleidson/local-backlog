//! CRUD de tags dentro de um `project_id`.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::domain::Tag;
use crate::error::BacklogError;

fn row_to_tag(row: &Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
    })
}

pub fn get_by_name(
    conn: &Connection,
    project_id: i64,
    name: &str,
) -> Result<Option<Tag>, BacklogError> {
    let mut stmt =
        conn.prepare("SELECT id, project_id, name FROM tags WHERE project_id = ?1 AND name = ?2")?;
    let tag = stmt
        .query_row(params![project_id, name], row_to_tag)
        .optional()?;
    Ok(tag)
}

/// Retorna a tag existente; cria no tenant caso não exista.
pub fn ensure(conn: &Connection, project_id: i64, name: &str) -> Result<Tag, BacklogError> {
    if let Some(tag) = get_by_name(conn, project_id, name)? {
        return Ok(tag);
    }
    conn.execute(
        "INSERT INTO tags (project_id, name) VALUES (?1, ?2)",
        params![project_id, name],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        project_id,
        name: name.to_string(),
    })
}

/// Retorna `true` se a tag foi anexada agora; `false` se já estava anexada.
pub fn attach(
    conn: &Connection,
    _project_id: i64,
    task_id: i64,
    tag_id: i64,
) -> Result<bool, BacklogError> {
    let n = conn.execute(
        "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
        params![task_id, tag_id],
    )?;
    Ok(n > 0)
}

/// Retorna `true` se a tag estava anexada e foi removida.
pub fn detach(
    conn: &Connection,
    _project_id: i64,
    task_id: i64,
    tag_id: i64,
) -> Result<bool, BacklogError> {
    let n = conn.execute(
        "DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2",
        params![task_id, tag_id],
    )?;
    Ok(n > 0)
}

pub fn list_for_task(
    conn: &Connection,
    project_id: i64,
    task_id: i64,
) -> Result<Vec<Tag>, BacklogError> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.project_id, g.name FROM tags g \
         JOIN task_tags tt ON tt.tag_id = g.id \
         WHERE tt.task_id = ?1 AND g.project_id = ?2 \
         ORDER BY g.name ASC",
    )?;
    let rows = stmt.query_map(params![task_id, project_id], row_to_tag)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn list_all_with_counts(
    conn: &Connection,
    project_id: i64,
) -> Result<Vec<(Tag, i64)>, BacklogError> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.project_id, g.name, COUNT(tt.task_id) AS cnt \
         FROM tags g LEFT JOIN task_tags tt ON tt.tag_id = g.id \
         WHERE g.project_id = ?1 GROUP BY g.id ORDER BY g.name ASC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok((row_to_tag(row)?, row.get::<_, i64>("cnt")?))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
