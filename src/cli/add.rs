//! `backlog add` — cria task no tenant atual.

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::bootstrap::App;
use crate::cli::{resolve_tenant, validate_enum};
use crate::db::events;
use crate::db::repo::{tag_repo, task_repo};
use crate::error::BacklogError;
use crate::output::{stderr_msg, stdout_data};

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Título da task.
    pub title: String,

    /// Corpo/descrição (texto livre).
    #[arg(long)]
    pub body: Option<String>,

    /// Tipo da task (validado contra `task_type.values` do config).
    #[arg(long = "type", value_name = "TYPE")]
    pub task_type: Option<String>,

    /// Prioridade (inteiro). Default: `priority.default` do config.
    #[arg(long)]
    pub priority: Option<i64>,

    /// Tags aplicadas. Aceita repetição e CSV: `--tag a,b --tag c`.
    #[arg(long, value_delimiter = ',')]
    pub tag: Vec<String>,

    /// ID da task pai. Deve pertencer ao mesmo tenant.
    #[arg(long)]
    pub parent: Option<i64>,

    /// Status inicial (default: `status.default` do config).
    #[arg(long)]
    pub status: Option<String>,
}

pub fn run(args: AddArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    let title = args.title.trim();
    if title.is_empty() {
        return Err(BacklogError::InvalidInput(
            "title não pode ser vazio".to_string(),
        ));
    }

    let status = args
        .status
        .unwrap_or_else(|| app.config.status.default.clone());
    validate_enum("status", &status, &app.config.status.values)?;

    if let Some(tt) = &args.task_type {
        validate_enum("type", tt, &app.config.task_type.values)?;
    }

    if let Some(parent_id) = args.parent {
        // Mesma mensagem para "não existe" e "tenant diferente" (ADR-0001).
        if !task_repo::exists(&app.conn, tenant.project_id, parent_id)? {
            return Err(BacklogError::TaskNotFound { id: parent_id });
        }
    }

    let priority = args.priority.unwrap_or(app.config.priority.default);

    let new = task_repo::NewTask {
        title: title.to_string(),
        body: args.body,
        status,
        priority,
        task_type: args.task_type,
        parent_id: args.parent,
    };
    let task = task_repo::insert(&app.conn, tenant.project_id, &new)?;

    for raw in &args.tag {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        let tag = tag_repo::ensure(&app.conn, tenant.project_id, name)?;
        let _ = tag_repo::attach(&app.conn, tenant.project_id, task.id, tag.id)?;
    }

    events::emit(
        &app.conn,
        task.id,
        "created",
        &json!({
            "title": task.title,
            "type": task.task_type,
            "priority": task.priority,
        }),
    )?;

    stderr_msg(format!("task {} criada em '{}'", task.id, tenant.name));
    stdout_data(task.id);
    Ok(())
}
