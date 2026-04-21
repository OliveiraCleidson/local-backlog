//! `backlog archive <ID>` — arquiva task (soft delete) sem remover do DB.

use std::path::Path;

use clap::Args;

use crate::bootstrap::App;
use crate::cli::resolve_tenant;
use crate::db::events;
use crate::db::repo::task_repo;
use crate::error::BacklogError;
use crate::output::stderr_msg;

#[derive(Args, Debug)]
pub struct ArchiveArgs {
    /// ID da task.
    pub id: i64,
}

pub fn run(args: ArchiveArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    let changed = task_repo::set_archived(&app.conn, tenant.project_id, args.id)?;
    if changed {
        events::emit_bare(&app.conn, args.id, "archived")?;
        stderr_msg(format!("task {} arquivada", args.id));
    } else {
        stderr_msg(format!("task {} já estava arquivada", args.id));
    }
    Ok(())
}
