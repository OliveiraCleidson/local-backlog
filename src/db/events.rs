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
use serde::Serialize;

use crate::error::BacklogError;

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
