//! Subcomandos do binário `backlog`.

use std::path::Path;

use clap::Subcommand;

use crate::bootstrap::App;
use crate::error::BacklogError;
use crate::format::Format;
use crate::resolver::{self, ResolvedTenant};

pub mod add;
pub mod archive;
pub mod attr;
pub mod doctor;
pub mod done;
pub mod edit;
pub mod events;
pub mod export;
pub mod init;
pub mod link;
pub mod list;
pub mod projects;
pub mod show;
pub mod tag;

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
    /// Edita campos de uma task.
    Edit(edit::EditArgs),
    /// Gerencia tags de uma task.
    Tag(tag::TagArgs),
    /// Cria/remove relações entre tasks.
    Link(link::LinkArgs),
    /// Atributos EAV (key/value) de tasks.
    Attr(attr::AttrArgs),
    /// Timeline de eventos da task.
    Events(events::EventsArgs),
    /// Dump do tenant em markdown ou JSON (contexto para IA).
    Export(export::ExportArgs),
    /// Diagnóstico e recuperação leve (`--fix` para limpar registry).
    Doctor(doctor::DoctorArgs),
}

#[tracing::instrument(level = "debug", skip(app, cmd), fields(subcommand))]
pub fn dispatch(cmd: Command, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let name = match &cmd {
        Command::Init(_) => "init",
        Command::Add(_) => "add",
        Command::List(_) => "list",
        Command::Show(_) => "show",
        Command::Done(_) => "done",
        Command::Archive(_) => "archive",
        Command::Projects(_) => "projects",
        Command::Edit(_) => "edit",
        Command::Tag(_) => "tag",
        Command::Link(_) => "link",
        Command::Attr(_) => "attr",
        Command::Events(_) => "events",
        Command::Export(_) => "export",
        Command::Doctor(_) => "doctor",
    };
    tracing::Span::current().record("subcommand", name);
    let started = std::time::Instant::now();
    let result = match cmd {
        Command::Init(args) => init::run(args, app, cwd),
        Command::Add(args) => add::run(args, app, cwd),
        Command::List(args) => list::run(args, app, cwd),
        Command::Show(args) => show::run(args, app, cwd),
        Command::Done(args) => done::run(args, app, cwd),
        Command::Archive(args) => archive::run(args, app, cwd),
        Command::Projects(args) => projects::run(args, app, cwd),
        Command::Edit(args) => edit::run(args, app, cwd),
        Command::Tag(args) => tag::run(args, app, cwd),
        Command::Link(args) => link::run(args, app, cwd),
        Command::Attr(args) => attr::run(args, app, cwd),
        Command::Events(args) => events::run(args, app, cwd),
        Command::Export(args) => export::run(args, app, cwd),
        Command::Doctor(args) => doctor::run(args, app, cwd),
    };
    tracing::debug!(
        elapsed_ms = started.elapsed().as_millis() as u64,
        subcommand = name,
        "subcommand finished"
    );
    result
}

/// Resolve o tenant da CWD; usado por todos subcomandos de dados.
pub(crate) fn resolve_tenant(app: &App, cwd: &Path) -> Result<ResolvedTenant, BacklogError> {
    resolver::resolve(cwd, &app.conn, &app.registry)
}

/// Converte string crua de `--format` em `Format`; produz `InvalidEnum` com
/// whitelist canônica `table, json`.
pub(crate) fn parse_format_arg(value: &str) -> Result<Format, BacklogError> {
    Format::parse(value).ok_or_else(|| BacklogError::InvalidEnum {
        field: "format",
        value: value.to_string(),
        allowed: "table, json".to_string(),
    })
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
