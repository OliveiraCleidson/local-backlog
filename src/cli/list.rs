//! `backlog list` — lista tasks do tenant atual.

use std::path::Path;

use clap::Args;

use crate::bootstrap::App;
use crate::cli::{resolve_tenant, validate_enum};
use crate::config::PriorityOrder;
use crate::db::repo::{tag_repo, task_repo};
use crate::error::BacklogError;
use crate::format::{render_tasks_json, render_tasks_table, Format};
use crate::output::stdout_data;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filtra por status (ex.: `todo`, `doing`).
    #[arg(long)]
    pub status: Option<String>,

    /// Filtra por tag (nome exato).
    #[arg(long)]
    pub tag: Option<String>,

    /// Filtra por tipo (validado contra `task_type.values`).
    #[arg(long = "type", value_name = "TYPE")]
    pub task_type: Option<String>,

    /// Filtra por prioridade exata.
    #[arg(long)]
    pub priority: Option<i64>,

    /// Filtra por id do parent.
    #[arg(long)]
    pub parent: Option<i64>,

    /// Inclui tasks arquivadas.
    #[arg(long, default_value_t = false)]
    pub include_archived: bool,

    /// Máximo de linhas.
    #[arg(long)]
    pub limit: Option<u32>,

    /// Formato de saída: `table` (default) ou `json`.
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Ordem de prioridade (`asc` | `desc`). Default vem do config.
    #[arg(long = "order")]
    pub order: Option<String>,
}

pub fn run(args: ListArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    let fmt = Format::parse(&args.format).ok_or_else(|| BacklogError::InvalidEnum {
        field: "format",
        value: args.format.clone(),
        allowed: "table, json".to_string(),
    })?;

    if let Some(status) = &args.status {
        validate_enum("status", status, &app.config.status.values)?;
    }
    if let Some(tt) = &args.task_type {
        validate_enum("type", tt, &app.config.task_type.values)?;
    }

    let order = match args.order.as_deref() {
        Some("asc") => Some(PriorityOrder::Asc),
        Some("desc") => Some(PriorityOrder::Desc),
        Some(other) => {
            return Err(BacklogError::InvalidEnum {
                field: "order",
                value: other.to_string(),
                allowed: "asc, desc".to_string(),
            });
        }
        None => Some(app.config.priority.order),
    };

    let filter = task_repo::ListFilter {
        status: args.status,
        tag: args.tag,
        task_type: args.task_type,
        priority: args.priority,
        parent_id: args.parent,
        include_archived: args.include_archived,
        limit: args.limit,
        priority_order: order,
    };

    let tasks = task_repo::list(&app.conn, tenant.project_id, &filter)?;
    let mut rows = Vec::with_capacity(tasks.len());
    for t in tasks {
        let tags = tag_repo::list_for_task(&app.conn, tenant.project_id, t.id)?;
        rows.push((t, tags));
    }

    let out = match fmt {
        Format::Table => render_tasks_table(&rows),
        Format::Json => render_tasks_json(&rows),
    };
    // `stdout_data` adiciona newline; remove se o renderer já termina com `\n`.
    let trimmed = out.strip_suffix('\n').unwrap_or(&out);
    stdout_data(trimmed);
    Ok(())
}
