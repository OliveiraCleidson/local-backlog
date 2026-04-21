//! Emissão de `task_events`. Payload sempre serializado como JSON.
//!
//! Schema inicial (Fase 2):
//!
//! | kind              | payload                                  |
//! |-------------------|------------------------------------------|
//! | `created`         | `{"title": "...", "type": "...", "priority": N}` |
//! | `status_changed`  | `{"from": "...", "to": "..."}`          |
//! | `archived`        | `{}`                                    |
//!
//! Fase 3 amplia com `field_changed`, `tag_added/removed`,
//! `link_added/removed`, `attr_set/unset`. Ver ADR-0002 (anexo).

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::BacklogError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub id: i64,
    pub task_id: i64,
    pub kind: String,
    pub payload: serde_json::Value,
    pub created_at: String,
}

pub fn emit(
    conn: &Connection,
    task_id: i64,
    kind: &str,
    payload: &impl Serialize,
) -> Result<(), BacklogError> {
    let json = serde_json::to_string(payload).map_err(|e| {
        BacklogError::InvalidInput(format!("payload de evento não serializável: {e}"))
    })?;
    conn.execute(
        "INSERT INTO task_events (task_id, kind, payload) VALUES (?1, ?2, ?3)",
        params![task_id, kind, json],
    )?;
    Ok(())
}

/// Emite com payload vazio `{}`.
pub fn emit_bare(conn: &Connection, task_id: i64, kind: &str) -> Result<(), BacklogError> {
    emit(conn, task_id, kind, &serde_json::json!({}))
}

/// Últimos `limit` eventos da task, do mais recente ao mais antigo.
/// Tenant-scoped via `project_id` (join com `tasks`).
pub fn list_for_task(
    conn: &Connection,
    project_id: i64,
    task_id: i64,
    limit: u32,
) -> Result<Vec<TaskEvent>, BacklogError> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.task_id, e.kind, e.payload, e.ts \
         FROM task_events e JOIN tasks t ON t.id = e.task_id \
         WHERE e.task_id = ?1 AND t.project_id = ?2 \
         ORDER BY e.id DESC LIMIT ?3",
    )?;
    let rows = stmt.query_map(params![task_id, project_id, limit], |r| {
        let payload_str: Option<String> = r.get("payload")?;
        let payload: serde_json::Value = payload_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::Value::Null);
        Ok(TaskEvent {
            id: r.get("id")?,
            task_id: r.get("task_id")?,
            kind: r.get("kind")?,
            payload,
            created_at: r.get("ts")?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
