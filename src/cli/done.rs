//! `backlog done <ID>` — transita task para o status terminal `done`.

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::bootstrap::App;
use crate::cli::resolve_tenant;
use crate::db::events;
use crate::db::repo::task_repo;
use crate::error::BacklogError;
use crate::output::stderr_msg;

const DONE_STATUS: &str = "done";

#[derive(Args, Debug)]
pub struct DoneArgs {
    /// ID da task.
    pub id: i64,
}

pub fn run(args: DoneArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    // Valida que status `done` está na whitelist do config.
    if !app.config.status.values.iter().any(|s| s == DONE_STATUS) {
        return Err(BacklogError::InvalidEnum {
            field: "status",
            value: DONE_STATUS.to_string(),
            allowed: app.config.status.values.join(", "),
        });
    }

    // Capta status atual para payload do evento.
    let current = task_repo::get(&app.conn, tenant.project_id, args.id)?
        .ok_or(BacklogError::TaskNotFound { id: args.id })?;

    let (_task, changed) =
        task_repo::set_status(&app.conn, tenant.project_id, args.id, DONE_STATUS, true)?;

    if changed {
        events::emit(
            &app.conn,
            args.id,
            "status_changed",
            &json!({ "from": current.status, "to": DONE_STATUS }),
        )?;
        stderr_msg(format!("task {} marcada como done", args.id));
    } else {
        stderr_msg(format!("task {} já estava em done", args.id));
    }
    Ok(())
}
