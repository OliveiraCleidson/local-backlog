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

    // Emite `field_changed` quando o novo valor difere do atual. A macro encapsula
    // comparação + json!+ emit para que cada campo fique em uma única linha.
    macro_rules! emit_if_changed {
        ($field:expr, $old:expr, $new:expr) => {
            if $old != $new {
                events::emit(
                    &app.conn,
                    args.id,
                    "field_changed",
                    &json!({ "field": $field, "from": $old, "to": $new }),
                )?;
            }
        };
    }

    if let Some(new_title) = &args.title {
        emit_if_changed!("title", &current.title, new_title);
    }
    if let Some(new_status) = &args.status {
        emit_if_changed!("status", &current.status, new_status);
    }
    if let Some(new_priority) = args.priority {
        emit_if_changed!("priority", current.priority, new_priority);
    }
    if let Some(body) = body_patch {
        emit_if_changed!("body", current.body, body);
    }
    if let Some(tt) = task_type_patch {
        emit_if_changed!("type", current.task_type, tt);
    }
    if let Some(parent) = patch.parent_id {
        emit_if_changed!("parent_id", current.parent_id, parent);
    }

    stderr_msg(format!("task {} atualizada", updated.id));
    Ok(())
}
