//! Subcomandos do binário `backlog`.

use std::path::Path;

use clap::Subcommand;

use crate::bootstrap::App;
use crate::error::BacklogError;
use crate::resolver::{self, ResolvedTenant};

pub mod add;
pub mod archive;
pub mod done;
pub mod init;
pub mod list;
pub mod projects;
pub mod show;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Registra o projeto atual no registry global.
    Init(init::InitArgs),
    /// Adiciona uma task no tenant atual.
    Add(add::AddArgs),
    /// Lista tasks do tenant atual.
    List(list::ListArgs),
    /// Mostra uma task agregada (tags, atributos, links, eventos).
    Show(show::ShowArgs),
    /// Marca a task como concluída.
    Done(done::DoneArgs),
    /// Arquiva uma task (soft delete).
    Archive(archive::ArchiveArgs),
    /// Administra projetos (superfície cross-tenant).
    Projects(projects::ProjectsArgs),
}

pub fn dispatch(cmd: Command, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    match cmd {
        Command::Init(args) => init::run(args, app, cwd),
        Command::Add(args) => add::run(args, app, cwd),
        Command::List(args) => list::run(args, app, cwd),
        Command::Show(args) => show::run(args, app, cwd),
        Command::Done(args) => done::run(args, app, cwd),
        Command::Archive(args) => archive::run(args, app, cwd),
        Command::Projects(args) => projects::run(args, app, cwd),
    }
}

/// Resolve o tenant da CWD; usado por todos subcomandos de dados.
pub(crate) fn resolve_tenant(app: &App, cwd: &Path) -> Result<ResolvedTenant, BacklogError> {
    resolver::resolve(cwd, &app.conn, &app.registry)
}

/// Valida que `value` pertence à lista `allowed`. Erro produz `InvalidEnum`
/// com campo, valor recebido e whitelist legível.
pub(crate) fn validate_enum(
    field: &'static str,
    value: &str,
    allowed: &[String],
) -> Result<(), BacklogError> {
    if allowed.iter().any(|a| a == value) {
        Ok(())
    } else {
        Err(BacklogError::InvalidEnum {
            field,
            value: value.to_string(),
            allowed: allowed.join(", "),
        })
    }
}
