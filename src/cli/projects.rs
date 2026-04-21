//! `backlog projects` — única superfície cross-tenant (meta).

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::bootstrap::App;
use crate::cli::parse_format_arg;
use crate::db::repo::project_repo;
use crate::domain::Project;
use crate::error::BacklogError;
use crate::format::{render_json, Format};
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
    let fmt = parse_format_arg(&args.format)?;
    let mut all = project_repo::list_all(&app.conn)?;
    if !args.include_archived {
        all.retain(|p| p.archived_at.is_none());
    }
    let mut rows: Vec<(Project, i64)> = Vec::with_capacity(all.len());
    for p in all {
        let count = project_repo::count_active_tasks(&app.conn, p.id)?;
        rows.push((p, count));
    }

    let out = match fmt {
        Format::Json => {
            let data: Vec<serde_json::Value> = rows
                .iter()
                .map(|(p, c)| {
                    serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "root_path": p.root_path,
                        "description": p.description,
                        "archived_at": p.archived_at,
                        "active_task_count": c,
                    })
                })
                .collect();
            render_json(&data)
        }
        Format::Table => render_projects_table(&rows),
    };
    stdout_data(out);
    Ok(())
}

fn show(args: ShowArgs, app: &App) -> Result<(), BacklogError> {
    let fmt = parse_format_arg(&args.format)?;
    let project = resolve(app, &args.target)?;
    let count = project_repo::count_active_tasks(&app.conn, project.id)?;

    let out = match fmt {
        Format::Json => {
            let payload = serde_json::json!({
                "id": project.id,
                "name": project.name,
                "root_path": project.root_path,
                "description": project.description,
                "archived_at": project.archived_at,
                "created_at": project.created_at,
                "updated_at": project.updated_at,
                "active_task_count": count,
            });
            render_json(&payload)
        }
        Format::Table => render_project_detail(&project, count),
    };
    stdout_data(out);
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

fn render_projects_table(rows: &[(Project, i64)]) -> String {
    if rows.is_empty() {
        return "nenhum projeto registrado".to_string();
    }
    let mut w_id = "ID".len();
    let mut w_name = "NAME".len();
    let mut w_status = "STATUS".len();
    let mut w_tasks = "TASKS".len();
    for (p, c) in rows {
        w_id = w_id.max(p.id.to_string().len());
        w_name = w_name.max(p.name.len());
        let status = if p.archived_at.is_some() {
            "archived"
        } else {
            "active"
        };
        w_status = w_status.max(status.len());
        w_tasks = w_tasks.max(c.to_string().len());
    }

    let mut out = String::new();
    out.push_str(&format!(
        "{:>w_id$}  {:<w_name$}  {:<w_status$}  {:>w_tasks$}  {}\n",
        "ID",
        "NAME",
        "STATUS",
        "TASKS",
        "PATH",
        w_id = w_id,
        w_name = w_name,
        w_status = w_status,
        w_tasks = w_tasks,
    ));
    for (p, c) in rows {
        let status = if p.archived_at.is_some() {
            "archived"
        } else {
            "active"
        };
        out.push_str(&format!(
            "{:>w_id$}  {:<w_name$}  {:<w_status$}  {:>w_tasks$}  {}\n",
            p.id,
            p.name,
            status,
            c,
            p.root_path,
            w_id = w_id,
            w_name = w_name,
            w_status = w_status,
            w_tasks = w_tasks,
        ));
    }
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

fn render_project_detail(p: &Project, active_task_count: i64) -> String {
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
    let _ = writeln!(s, "active_tasks: {active_task_count}");
    if s.ends_with('\n') {
        s.pop();
    }
    s
}
