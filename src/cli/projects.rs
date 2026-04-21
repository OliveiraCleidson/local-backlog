//! `backlog projects` — única superfície cross-tenant (meta).

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::bootstrap::App;
use crate::db::repo::project_repo;
use crate::domain::Project;
use crate::error::BacklogError;
use crate::format::{Format, JsonEnvelope};
use crate::output::{stderr_msg, stdout_data};
use crate::registry::RegistryEntry;

#[derive(Args, Debug)]
pub struct ProjectsArgs {
    #[command(subcommand)]
    pub command: ProjectsCmd,
}

#[derive(Subcommand, Debug)]
pub enum ProjectsCmd {
    /// Lista todos os projetos registrados.
    List(ListArgs),
    /// Detalha um projeto por id ou nome.
    Show(ShowArgs),
    /// Atualiza o `root_path` de um projeto (mantém `id`).
    Relink(RelinkArgs),
    /// Arquiva (ou restaura com `--restore`) um projeto.
    Archive(ArchiveArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long, default_value = "table")]
    pub format: String,
    #[arg(long, default_value_t = false)]
    pub include_archived: bool,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// ID numérico ou nome do projeto.
    pub target: String,
    #[arg(long, default_value = "table")]
    pub format: String,
}

#[derive(Args, Debug)]
pub struct RelinkArgs {
    pub target: String,
    /// Novo `root_path` (será canonizado).
    pub new_path: PathBuf,
}

#[derive(Args, Debug)]
pub struct ArchiveArgs {
    pub target: String,
    /// Restaura em vez de arquivar.
    #[arg(long, default_value_t = false)]
    pub restore: bool,
}

pub fn run(args: ProjectsArgs, app: &mut App, _cwd: &Path) -> Result<(), BacklogError> {
    match args.command {
        ProjectsCmd::List(a) => list(a, app),
        ProjectsCmd::Show(a) => show(a, app),
        ProjectsCmd::Relink(a) => relink(a, app),
        ProjectsCmd::Archive(a) => archive(a, app),
    }
}

fn parse_format(s: &str) -> Result<Format, BacklogError> {
    Format::parse(s).ok_or_else(|| BacklogError::InvalidEnum {
        field: "format",
        value: s.to_string(),
        allowed: "table, json".to_string(),
    })
}

fn resolve(app: &App, target: &str) -> Result<Project, BacklogError> {
    if let Ok(id) = target.parse::<i64>() {
        if let Some(p) = project_repo::get_by_id(&app.conn, id)? {
            return Ok(p);
        }
    }
    project_repo::get_by_name(&app.conn, target)?.ok_or_else(|| BacklogError::ProjectNotFound {
        name: target.to_string(),
    })
}

fn list(args: ListArgs, app: &App) -> Result<(), BacklogError> {
    let fmt = parse_format(&args.format)?;
    let mut all = project_repo::list_all(&app.conn)?;
    if !args.include_archived {
        all.retain(|p| p.archived_at.is_none());
    }

    let out = match fmt {
        Format::Json => serde_json::to_string_pretty(&JsonEnvelope::new(&all))
            .unwrap_or_else(|_| "{}".to_string()),
        Format::Table => render_projects_table(&all),
    };
    let trimmed = out.strip_suffix('\n').unwrap_or(&out);
    stdout_data(trimmed);
    Ok(())
}

fn show(args: ShowArgs, app: &App) -> Result<(), BacklogError> {
    let fmt = parse_format(&args.format)?;
    let project = resolve(app, &args.target)?;

    let out = match fmt {
        Format::Json => serde_json::to_string_pretty(&JsonEnvelope::new(&project))
            .unwrap_or_else(|_| "{}".to_string()),
        Format::Table => render_project_detail(&project),
    };
    let trimmed = out.strip_suffix('\n').unwrap_or(&out);
    stdout_data(trimmed);
    Ok(())
}

fn relink(args: RelinkArgs, app: &mut App) -> Result<(), BacklogError> {
    let project = resolve(app, &args.target)?;

    let canon = std::fs::canonicalize(&args.new_path).map_err(|source| BacklogError::Io {
        path: args.new_path.clone(),
        source,
    })?;
    let canon_str = canon.to_string_lossy().into_owned();

    // Rejeita se outro projeto já ocupa o novo path.
    if let Some(other) = project_repo::get_by_root_path(&app.conn, &canon_str)? {
        if other.id != project.id {
            return Err(BacklogError::InvalidInput(format!(
                "path já registrado pelo projeto '{}' (id={})",
                other.name, other.id
            )));
        }
    }

    project_repo::update_root_path(&app.conn, project.id, &canon_str)?;
    app.registry.upsert(RegistryEntry {
        id: project.id,
        name: project.name.clone(),
        root_path: canon.clone(),
    });
    app.save_registry()?;

    stderr_msg(format!(
        "projeto '{}' (id={}) relinkado para {}",
        project.name,
        project.id,
        canon.display()
    ));
    stdout_data(project.id);
    Ok(())
}

fn archive(args: ArchiveArgs, app: &mut App) -> Result<(), BacklogError> {
    let project = resolve(app, &args.target)?;

    if args.restore {
        project_repo::restore(&app.conn, project.id)?;
        // Registry: re-inserir entry (pode ter sido removida ao arquivar).
        app.registry.upsert(RegistryEntry {
            id: project.id,
            name: project.name.clone(),
            root_path: PathBuf::from(&project.root_path),
        });
        app.save_registry()?;
        stderr_msg(format!(
            "projeto '{}' (id={}) restaurado",
            project.name, project.id
        ));
    } else {
        project_repo::archive(&app.conn, project.id)?;
        // Mantém entry no registry: resolver precisa dela para devolver
        // `ProjectArchived` em vez de `ProjectNotRegistered` na CWD.
        stderr_msg(format!(
            "projeto '{}' (id={}) arquivado",
            project.name, project.id
        ));
    }
    stdout_data(project.id);
    Ok(())
}

fn render_projects_table(all: &[Project]) -> String {
    if all.is_empty() {
        return "nenhum projeto registrado\n".to_string();
    }
    let mut w_id = "ID".len();
    let mut w_name = "NAME".len();
    let mut w_status = "STATUS".len();
    for p in all {
        w_id = w_id.max(p.id.to_string().len());
        w_name = w_name.max(p.name.len());
        let status = if p.archived_at.is_some() {
            "archived"
        } else {
            "active"
        };
        w_status = w_status.max(status.len());
    }

    let mut out = String::new();
    out.push_str(&format!(
        "{:>w_id$}  {:<w_name$}  {:<w_status$}  {}\n",
        "ID",
        "NAME",
        "STATUS",
        "PATH",
        w_id = w_id,
        w_name = w_name,
        w_status = w_status,
    ));
    for p in all {
        let status = if p.archived_at.is_some() {
            "archived"
        } else {
            "active"
        };
        out.push_str(&format!(
            "{:>w_id$}  {:<w_name$}  {:<w_status$}  {}\n",
            p.id,
            p.name,
            status,
            p.root_path,
            w_id = w_id,
            w_name = w_name,
            w_status = w_status,
        ));
    }
    out
}

fn render_project_detail(p: &Project) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "id:          {}", p.id);
    let _ = writeln!(s, "name:        {}", p.name);
    let _ = writeln!(s, "root_path:   {}", p.root_path);
    if let Some(d) = &p.description {
        let _ = writeln!(s, "description: {d}");
    }
    if let Some(a) = &p.archived_at {
        let _ = writeln!(s, "archived_at: {a}");
    }
    let _ = writeln!(s, "created_at:  {}", p.created_at);
    let _ = writeln!(s, "updated_at:  {}", p.updated_at);
    s
}
