//! `backlog events <id>` — timeline da task.

use std::path::Path;

use clap::Args;

use crate::bootstrap::App;
use crate::cli::resolve_tenant;
use crate::db::events;
use crate::db::repo::task_repo;
use crate::error::BacklogError;
use crate::format::{Format, JsonEnvelope};
use crate::output::stdout_data;

const DEFAULT_LIMIT: u32 = 50;

#[derive(Args, Debug)]
pub struct EventsArgs {
    pub id: i64,

    /// Filtra por `kind` (ex.: `status_changed`, `tag_added`).
    #[arg(long)]
    pub kind: Option<String>,

    #[arg(long, default_value_t = DEFAULT_LIMIT)]
    pub limit: u32,

    #[arg(long, default_value = "table")]
    pub format: String,
}

pub fn run(args: EventsArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    let fmt = Format::parse(&args.format).ok_or_else(|| BacklogError::InvalidEnum {
        field: "format",
        value: args.format.clone(),
        allowed: "table, json".to_string(),
    })?;

    if !task_repo::exists(&app.conn, tenant.project_id, args.id)? {
        return Err(BacklogError::TaskNotFound { id: args.id });
    }

    let mut list = events::list_for_task(&app.conn, tenant.project_id, args.id, args.limit)?;
    if let Some(k) = &args.kind {
        list.retain(|e| e.kind == *k);
    }

    let out = match fmt {
        Format::Json => serde_json::to_string_pretty(&JsonEnvelope::new(&list))
            .unwrap_or_else(|_| "{}".to_string()),
        Format::Table => {
            if list.is_empty() {
                "sem eventos".to_string()
            } else {
                list.iter()
                    .map(|e| format!("{}  {}  {}", e.created_at, e.kind, e.payload))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    };
    stdout_data(out);
    Ok(())
}
