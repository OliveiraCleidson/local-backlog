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
    /// ID de destino.
    pub to: i64,
    /// Tipo da relação (whitelist em `config.toml::link.kinds`).
    #[arg(long)]
    pub kind: String,
    /// Remove em vez de criar.
    #[arg(long, default_value_t = false)]
    pub remove: bool,
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

    if args.remove {
        let removed =
            link_repo::remove(&app.conn, tenant.project_id, args.from, args.to, &args.kind)?;
        if removed {
            events::emit(
                &app.conn,
                args.from,
                "link_removed",
                &json!({ "to": args.to, "kind": args.kind }),
            )?;
            stderr_msg(format!(
                "link {} -{}-> {} removido",
                args.from, args.kind, args.to
            ));
        } else {
            stderr_msg(format!(
                "link {} -{}-> {} não existia",
                args.from, args.kind, args.to
            ));
        }
    } else {
        let created = link_repo::add(&app.conn, tenant.project_id, args.from, args.to, &args.kind)?;
        if created {
            events::emit(
                &app.conn,
                args.from,
                "link_added",
                &json!({ "to": args.to, "kind": args.kind }),
            )?;
            stderr_msg(format!(
                "link {} -{}-> {} criado",
                args.from, args.kind, args.to
            ));
        } else {
            stderr_msg(format!(
                "link {} -{}-> {} já existia",
                args.from, args.kind, args.to
            ));
        }
    }
    Ok(())
}
