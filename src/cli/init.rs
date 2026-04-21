//! `backlog init` — registra a CWD como projeto no registry global.

use std::path::Path;

use clap::Args;

use crate::bootstrap::App;
use crate::db::repo::project_repo;
use crate::error::BacklogError;
use crate::output::{stderr_msg, stdout_data};
use crate::registry::RegistryEntry;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Nome do projeto (default: basename da CWD).
    #[arg(long)]
    pub name: Option<String>,

    /// Descrição opcional do projeto.
    #[arg(long)]
    pub description: Option<String>,

    /// Pula prompts interativos; usa defaults/flags.
    #[arg(long)]
    pub yes: bool,
}

pub fn run(args: InitArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let canonical = std::fs::canonicalize(cwd).map_err(|source| BacklogError::Io {
        path: cwd.to_path_buf(),
        source,
    })?;
    let root_str = canonical.to_string_lossy().into_owned();

    if let Some(existing) = project_repo::get_by_root_path(&app.conn, &root_str)? {
        stderr_msg(format!(
            "projeto '{}' (id={}) já registrado em {}",
            existing.name,
            existing.id,
            canonical.display()
        ));
        stdout_data(existing.id);
        return Ok(());
    }

    let default_name = canonical
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string();

    let (name, description) = collect_inputs(&args, &default_name)?;

    let project = project_repo::insert(&app.conn, &name, &root_str, description.as_deref())?;

    app.registry.upsert(RegistryEntry {
        id: project.id,
        name: project.name.clone(),
        root_path: canonical.clone(),
    });
    if let Err(e) = app.save_registry() {
        tracing::error!(error = %e, "falha ao escrever registry; rode `backlog doctor`");
        return Err(e);
    }

    stderr_msg(format!(
        "projeto '{}' registrado em {}",
        project.name,
        canonical.display()
    ));
    stdout_data(project.id);
    Ok(())
}

fn collect_inputs(
    args: &InitArgs,
    default_name: &str,
) -> Result<(String, Option<String>), BacklogError> {
    use std::io::IsTerminal;

    // `--yes` ou stdin não-TTY: nada de prompts.
    let non_interactive = args.yes || !std::io::stdin().is_terminal();
    if non_interactive {
        let name = args
            .name
            .clone()
            .unwrap_or_else(|| default_name.to_string());
        return Ok((name, args.description.clone()));
    }

    let name = match &args.name {
        Some(n) => n.clone(),
        None => inquire::Text::new("Nome do projeto:")
            .with_default(default_name)
            .prompt()
            .map_err(|e| BacklogError::InvalidInput(e.to_string()))?,
    };
    let description = match &args.description {
        Some(d) => Some(d.clone()),
        None => inquire::Text::new("Descrição (opcional):")
            .prompt_skippable()
            .map_err(|e| BacklogError::InvalidInput(e.to_string()))?
            .filter(|s| !s.trim().is_empty()),
    };
    Ok((name, description))
}
