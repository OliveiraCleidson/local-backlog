//! Atributos EAV de tasks. Todos os métodos checam tenant via join em `tasks`.

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::BacklogError;

/// Chave EAV: começa com letra minúscula; aceita `[a-z0-9_.-]*`.
pub fn is_valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-'))
}

pub fn set(
    conn: &Connection,
    project_id: i64,
    task_id: i64,
    key: &str,
    value: &str,
) -> Result<(), BacklogError> {
    if !is_valid_key(key) {
        return Err(BacklogError::InvalidInput(format!(
            "chave de atributo inválida: '{key}' (use [a-z][a-z0-9_.-]*)"
        )));
    }
    // Garante que task pertence ao tenant.
    let owner: Option<i64> = conn
        .query_row(
            "SELECT project_id FROM tasks WHERE id = ?1",
            params![task_id],
            |r| r.get(0),
        )
        .optional()?;
    if owner != Some(project_id) {
        return Err(BacklogError::TaskNotFound { id: task_id });
    }

    conn.execute(
        "INSERT INTO task_attributes (task_id, key, value) VALUES (?1, ?2, ?3) \
         ON CONFLICT(task_id, key) DO UPDATE SET value = excluded.value",
        params![task_id, key, value],
    )?;
    Ok(())
}

pub fn unset(
    conn: &Connection,
    project_id: i64,
    task_id: i64,
    key: &str,
) -> Result<bool, BacklogError> {
    let owner: Option<i64> = conn
        .query_row(
            "SELECT project_id FROM tasks WHERE id = ?1",
            params![task_id],
            |r| r.get(0),
        )
        .optional()?;
    if owner != Some(project_id) {
        return Err(BacklogError::TaskNotFound { id: task_id });
    }
    let n = conn.execute(
        "DELETE FROM task_attributes WHERE task_id = ?1 AND key = ?2",
        params![task_id, key],
    )?;
    Ok(n > 0)
}

pub fn get(
    conn: &Connection,
    project_id: i64,
    task_id: i64,
    key: &str,
) -> Result<Option<String>, BacklogError> {
    let val: Option<String> = conn
        .query_row(
            "SELECT a.value FROM task_attributes a JOIN tasks t ON t.id = a.task_id \
             WHERE a.task_id = ?1 AND a.key = ?2 AND t.project_id = ?3",
            params![task_id, key, project_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(val)
}

pub fn list_for_task(
    conn: &Connection,
    project_id: i64,
    task_id: i64,
) -> Result<Vec<(String, String)>, BacklogError> {
    let mut stmt = conn.prepare(
        "SELECT a.key, a.value FROM task_attributes a JOIN tasks t ON t.id = a.task_id \
         WHERE a.task_id = ?1 AND t.project_id = ?2 ORDER BY a.key ASC",
    )?;
    let rows = stmt.query_map(params![task_id, project_id], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
