//! `backlog link` — cria/remove relações entre tasks.

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::bootstrap::App;
use crate::cli::resolve_tenant;
use crate::db::events;
use crate::db::repo::link_repo;
use crate::error::BacklogError;
use crate::output::stderr_msg;

#[derive(Args, Debug)]
pub struct LinkArgs {
    /// ID de origem.
    pub from: i64,
    /// Remove em vez de criar; toma o ID de destino como argumento.
    #[arg(long, value_name = "TO")]
    pub remove: Option<i64>,
    /// ID de destino (obrigatório quando não se usa `--remove`).
    pub to: Option<i64>,
    /// Tipo da relação (whitelist em `config.toml::link.kinds`).
    #[arg(long)]
    pub kind: String,
}

pub fn run(args: LinkArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    if !app.config.link.kinds.iter().any(|k| k == &args.kind) {
        return Err(BacklogError::InvalidEnum {
            field: "kind",
            value: args.kind.clone(),
            allowed: app.config.link.kinds.join(", "),
        });
    }

    let (to, mode) = match (args.remove, args.to) {
        (Some(_), Some(_)) => {
            return Err(BacklogError::InvalidInput(
                "use `--remove <TO>` ou `<TO>` posicional, não ambos".to_string(),
            ));
        }
        (Some(to), None) => (to, Mode::Remove),
        (None, Some(to)) => (to, Mode::Add),
        (None, None) => {
            return Err(BacklogError::InvalidInput(
                "TO obrigatório (posicional ou via `--remove <TO>`)".to_string(),
            ));
        }
    };

    match mode {
        Mode::Remove => {
            let removed =
                link_repo::remove(&app.conn, tenant.project_id, args.from, to, &args.kind)?;
            if removed {
                events::emit(
                    &app.conn,
                    args.from,
                    "link_removed",
                    &json!({ "from": args.from, "to": to, "kind": args.kind }),
                )?;
                stderr_msg(format!(
                    "link {} -{}-> {} removido",
                    args.from, args.kind, to
                ));
            } else {
                stderr_msg(format!(
                    "link {} -{}-> {} não existia",
                    args.from, args.kind, to
                ));
            }
        }
        Mode::Add => {
            let created = link_repo::add(&app.conn, tenant.project_id, args.from, to, &args.kind)?;
            if created {
                events::emit(
                    &app.conn,
                    args.from,
                    "link_added",
                    &json!({ "from": args.from, "to": to, "kind": args.kind }),
                )?;
                stderr_msg(format!("link {} -{}-> {} criado", args.from, args.kind, to));
            } else {
                stderr_msg(format!(
                    "link {} -{}-> {} já existia",
                    args.from, args.kind, to
                ));
            }
        }
    }
    Ok(())
}

enum Mode {
    Add,
    Remove,
}
