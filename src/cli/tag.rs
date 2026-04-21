//! `backlog tag` — gerencia tags de uma task.

use std::path::Path;

use clap::{Args, Subcommand};
use serde_json::json;

use crate::bootstrap::App;
use crate::cli::{parse_format_arg, resolve_tenant};
use crate::db::events;
use crate::db::repo::{tag_repo, task_repo};
use crate::error::BacklogError;
use crate::format::{render_json, Format};
use crate::output::{stderr_msg, stdout_data};

#[derive(Args, Debug)]
pub struct TagArgs {
    #[command(subcommand)]
    pub command: TagCmd,
}

#[derive(Subcommand, Debug)]
pub enum TagCmd {
    /// Anexa tags (CSV ou repetição) a uma task.
    Add(MutateArgs),
    /// Remove tags de uma task.
    Remove(MutateArgs),
    /// Lista tags de uma task.
    List(ListArgs),
}

#[derive(Args, Debug)]
pub struct MutateArgs {
    pub id: i64,
    /// Uma ou mais tags (aceita CSV e repetição).
    #[arg(value_delimiter = ',', required = true)]
    pub tags: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Se informado, lista só as tags dessa task. Sem ID, lista todas as tags
    /// do tenant com contagem de uso.
    pub id: Option<i64>,
    #[arg(long, default_value = "table")]
    pub format: String,
}

pub fn run(args: TagArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    match args.command {
        TagCmd::Add(a) => add(a, app, cwd),
        TagCmd::Remove(a) => remove(a, app, cwd),
        TagCmd::List(a) => list(a, app, cwd),
    }
}

fn add(args: MutateArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;
    if !task_repo::exists(&app.conn, tenant.project_id, args.id)? {
        return Err(BacklogError::TaskNotFound { id: args.id });
    }
    for raw in &args.tags {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        let tag = tag_repo::ensure(&app.conn, tenant.project_id, name)?;
        let attached = tag_repo::attach(&app.conn, tenant.project_id, args.id, tag.id)?;
        if attached {
            events::emit(&app.conn, args.id, "tag_added", &json!({ "tag": name }))?;
        }
    }
    stderr_msg(format!("tags anexadas em task {}", args.id));
    Ok(())
}

fn remove(args: MutateArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;
    if !task_repo::exists(&app.conn, tenant.project_id, args.id)? {
        return Err(BacklogError::TaskNotFound { id: args.id });
    }
    for raw in &args.tags {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        if let Some(tag) = tag_repo::get_by_name(&app.conn, tenant.project_id, name)? {
            let detached = tag_repo::detach(&app.conn, tenant.project_id, args.id, tag.id)?;
            if detached {
                events::emit(&app.conn, args.id, "tag_removed", &json!({ "tag": name }))?;
            }
        }
    }
    stderr_msg(format!("tags removidas de task {}", args.id));
    Ok(())
}

fn list(args: ListArgs, app: &App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;
    let fmt = parse_format_arg(&args.format)?;

    if let Some(id) = args.id {
        if !task_repo::exists(&app.conn, tenant.project_id, id)? {
            return Err(BacklogError::TaskNotFound { id });
        }
        let tags = tag_repo::list_for_task(&app.conn, tenant.project_id, id)?;
        let out = match fmt {
            Format::Json => render_json(&tags),
            Format::Table => {
                if tags.is_empty() {
                    "sem tags".to_string()
                } else {
                    tags.iter()
                        .map(|t| format!("#{}", t.name))
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            }
        };
        stdout_data(out);
        return Ok(());
    }

    let rows = tag_repo::list_all_with_counts(&app.conn, tenant.project_id)?;
    let out = match fmt {
        Format::Json => {
            let data: Vec<serde_json::Value> = rows
                .iter()
                .map(|(t, c)| json!({ "name": t.name, "count": c }))
                .collect();
            render_json(&data)
        }
        Format::Table => {
            if rows.is_empty() {
                "sem tags".to_string()
            } else {
                let mut w_name = "TAG".len();
                for (t, _) in &rows {
                    w_name = w_name.max(t.name.len() + 1);
                }
                let mut s = String::new();
                s.push_str(&format!("{:<w_name$}  USES\n", "TAG", w_name = w_name));
                for (t, c) in &rows {
                    s.push_str(&format!(
                        "{:<w_name$}  {}\n",
                        format!("#{}", t.name),
                        c,
                        w_name = w_name,
                    ));
                }
                s.trim_end_matches('\n').to_string()
            }
        }
    };
    stdout_data(out);
    Ok(())
}
