//! `backlog attr` — atributos EAV de task.

use std::path::Path;

use clap::{Args, Subcommand};
use serde_json::json;

use crate::bootstrap::App;
use crate::cli::resolve_tenant;
use crate::db::events;
use crate::db::repo::{attr_repo, task_repo};
use crate::error::BacklogError;
use crate::format::{Format, JsonEnvelope};
use crate::output::{stderr_msg, stdout_data};

#[derive(Args, Debug)]
pub struct AttrArgs {
    #[command(subcommand)]
    pub command: AttrCmd,
}

#[derive(Subcommand, Debug)]
pub enum AttrCmd {
    /// Define atributo (upsert).
    Set(SetArgs),
    /// Remove atributo.
    Unset(UnsetArgs),
    /// Lista atributos da task.
    List(ListArgs),
}

#[derive(Args, Debug)]
pub struct SetArgs {
    pub id: i64,
    pub key: String,
    pub value: String,
}

#[derive(Args, Debug)]
pub struct UnsetArgs {
    pub id: i64,
    pub key: String,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    pub id: i64,
    #[arg(long, default_value = "table")]
    pub format: String,
}

pub fn run(args: AttrArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    match args.command {
        AttrCmd::Set(a) => set(a, app, cwd),
        AttrCmd::Unset(a) => unset(a, app, cwd),
        AttrCmd::List(a) => list(a, app, cwd),
    }
}

fn set(args: SetArgs, app: &App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;
    let prev = attr_repo::get(&app.conn, tenant.project_id, args.id, &args.key)?;
    attr_repo::set(
        &app.conn,
        tenant.project_id,
        args.id,
        &args.key,
        &args.value,
    )?;
    events::emit(
        &app.conn,
        args.id,
        "attr_set",
        &json!({ "key": args.key, "from": prev, "to": args.value }),
    )?;
    stderr_msg(format!("attr {} definido em task {}", args.key, args.id));
    Ok(())
}

fn unset(args: UnsetArgs, app: &App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;
    let removed = attr_repo::unset(&app.conn, tenant.project_id, args.id, &args.key)?;
    if removed {
        events::emit(
            &app.conn,
            args.id,
            "attr_unset",
            &json!({ "key": args.key }),
        )?;
        stderr_msg(format!("attr {} removido de task {}", args.key, args.id));
    } else {
        stderr_msg(format!(
            "attr {} não estava presente em task {}",
            args.key, args.id
        ));
    }
    Ok(())
}

fn list(args: ListArgs, app: &App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;
    let fmt = Format::parse(&args.format).ok_or_else(|| BacklogError::InvalidEnum {
        field: "format",
        value: args.format.clone(),
        allowed: "table, json".to_string(),
    })?;
    if !task_repo::exists(&app.conn, tenant.project_id, args.id)? {
        return Err(BacklogError::TaskNotFound { id: args.id });
    }
    let attrs = attr_repo::list_for_task(&app.conn, tenant.project_id, args.id)?;

    let out = match fmt {
        Format::Json => {
            let data: Vec<serde_json::Value> = attrs
                .iter()
                .map(|(k, v)| json!({ "key": k, "value": v }))
                .collect();
            serde_json::to_string_pretty(&JsonEnvelope::new(&data))
                .unwrap_or_else(|_| "{}".to_string())
        }
        Format::Table => {
            if attrs.is_empty() {
                "sem atributos".to_string()
            } else {
                attrs
                    .iter()
                    .map(|(k, v)| format!("{k} = {v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    };
    stdout_data(out);
    Ok(())
}
