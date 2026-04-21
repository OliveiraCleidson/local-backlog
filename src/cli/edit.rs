//! `backlog edit <ID>` — atualiza campos de uma task.
//!
//! Convenção (MANIFESTO §2.10):
//! - Omissão da flag → mantém o valor.
//! - `--body ""` zera o campo; `--title ""` é rejeitado (título é obrigatório).
//! - `--type ""` zera; `--parent none` zera; `--priority` só aceita inteiro.
//!
//! Cada mudança emite evento `field_changed { field, from, to }`.

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::bootstrap::App;
use crate::cli::{resolve_tenant, validate_enum};
use crate::db::events;
use crate::db::repo::task_repo;
use crate::error::BacklogError;
use crate::output::stderr_msg;

#[derive(Args, Debug)]
pub struct EditArgs {
    pub id: i64,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long)]
    pub body: Option<String>,

    #[arg(long)]
    pub status: Option<String>,

    #[arg(long)]
    pub priority: Option<i64>,

    #[arg(long = "type")]
    pub task_type: Option<String>,

    /// `--parent <id>` define parent; `--parent none` zera.
    #[arg(long)]
    pub parent: Option<String>,
}

pub fn run(args: EditArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    let current = task_repo::get(&app.conn, tenant.project_id, args.id)?
        .ok_or(BacklogError::TaskNotFound { id: args.id })?;

    if let Some(t) = &args.title {
        if t.trim().is_empty() {
            return Err(BacklogError::InvalidInput(
                "title não pode ser vazio".to_string(),
            ));
        }
    }
    if let Some(s) = &args.status {
        validate_enum("status", s, &app.config.status.values)?;
    }
    if let Some(tt) = &args.task_type {
        if !tt.is_empty() {
            validate_enum("type", tt, &app.config.task_type.values)?;
        }
    }

    let parent_patch: Option<Option<i64>> = match args.parent.as_deref() {
        None => None,
        Some("none") => Some(None),
        Some(s) => {
            let pid: i64 = s.parse().map_err(|_| {
                BacklogError::InvalidInput(format!("parent inválido: '{s}' (use id ou `none`)"))
            })?;
            if !task_repo::exists(&app.conn, tenant.project_id, pid)? {
                return Err(BacklogError::TaskNotFound { id: pid });
            }
            Some(Some(pid))
        }
    };

    let task_type_patch: Option<Option<String>> =
        args.task_type
            .as_ref()
            .map(|v| if v.is_empty() { None } else { Some(v.clone()) });
    let body_patch: Option<Option<String>> =
        args.body
            .as_ref()
            .map(|v| if v.is_empty() { None } else { Some(v.clone()) });

    let patch = task_repo::TaskPatch {
        title: args.title.clone(),
        body: body_patch.clone(),
        status: args.status.clone(),
        priority: args.priority,
        task_type: task_type_patch.clone(),
        parent_id: parent_patch,
    };

    let updated = task_repo::update_fields(&app.conn, tenant.project_id, args.id, &patch)?;

    // Emite `field_changed` para cada campo que mudou de fato.
    if let Some(new_title) = &args.title {
        if current.title != *new_title {
            emit_change(&app.conn, args.id, "title", &current.title, new_title)?;
        }
    }
    if let Some(new_status) = &args.status {
        if current.status != *new_status {
            emit_change(&app.conn, args.id, "status", &current.status, new_status)?;
        }
    }
    if let Some(new_priority) = args.priority {
        if current.priority != new_priority {
            events::emit(
                &app.conn,
                args.id,
                "field_changed",
                &json!({
                    "field": "priority",
                    "from": current.priority,
                    "to": new_priority,
                }),
            )?;
        }
    }
    if let Some(body) = body_patch {
        if current.body != body {
            events::emit(
                &app.conn,
                args.id,
                "field_changed",
                &json!({ "field": "body", "from": current.body, "to": body }),
            )?;
        }
    }
    if let Some(tt) = task_type_patch {
        if current.task_type != tt {
            events::emit(
                &app.conn,
                args.id,
                "field_changed",
                &json!({ "field": "type", "from": current.task_type, "to": tt }),
            )?;
        }
    }
    if let Some(parent) = patch.parent_id {
        if current.parent_id != parent {
            events::emit(
                &app.conn,
                args.id,
                "field_changed",
                &json!({ "field": "parent_id", "from": current.parent_id, "to": parent }),
            )?;
        }
    }

    stderr_msg(format!("task {} atualizada", updated.id));
    Ok(())
}

fn emit_change(
    conn: &rusqlite::Connection,
    task_id: i64,
    field: &str,
    from: &str,
    to: &str,
) -> Result<(), BacklogError> {
    events::emit(
        conn,
        task_id,
        "field_changed",
        &json!({ "field": field, "from": from, "to": to }),
    )
}
