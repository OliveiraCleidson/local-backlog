//! `backlog doctor` — diagnóstico e recuperação leve.
//!
//! Checagens:
//! - registry carrega sem erro e aponta para paths existentes.
//! - toda entry do registry tem `project_id` correspondente em `projects`.
//! - todo projeto não-arquivado em `projects` tem entry no registry.
//! - `PRAGMA integrity_check` retorna `ok`.
//! - zero órfãos em `task_tags`, `task_attributes`, `task_links`, `task_events`.
//!
//! `--fix` remove entries do registry cujo path não existe mais ou cujo
//! `project_id` não está na tabela. Pede confirmação via `inquire` por padrão;
//! `--yes` pula a confirmação.
//!
//! Exit codes: `0` tudo ok, `1` warnings (ex.: paths inexistentes), `2` erros
//! graves (integrity_check falhou, órfãos, divergências não cobertas por `--fix`).

use std::path::Path;

use clap::Args;
use inquire::Confirm;
use rusqlite::OptionalExtension;

use crate::bootstrap::App;
use crate::db::repo::project_repo;
use crate::error::BacklogError;
use crate::output::{stderr_msg, stdout_data};

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Tenta corrigir inconsistências não-destrutivas (limpa entries do registry
    /// com path ausente ou `project_id` órfão).
    #[arg(long, default_value_t = false)]
    pub fix: bool,

    /// Aceita automaticamente as correções sem prompt.
    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

#[derive(Debug, Default)]
struct Report {
    warnings: Vec<String>,
    errors: Vec<String>,
    fixable: Vec<FixableIssue>,
}

#[derive(Debug)]
enum FixableIssue {
    RegistryMissingPath { id: i64, path: String },
    RegistryOrphanProject { id: i64 },
}

pub fn run(args: DoctorArgs, app: &mut App, _cwd: &Path) -> Result<(), BacklogError> {
    let mut report = Report::default();

    if let Some(reason) = app.registry_corrupt.as_deref() {
        report.errors.push(format!(
            "registry.toml inválido: {reason} (corrija manualmente ou apague o arquivo)"
        ));
    }
    check_integrity(&app.conn, &mut report)?;
    check_user_version(&app.conn, &mut report)?;
    check_registry(app, &mut report)?;
    check_projects_vs_registry(app, &mut report)?;
    check_orphans(&app.conn, &mut report)?;
    check_cross_tenant(&app.conn, &mut report)?;

    print_report(&report);

    if args.fix && !report.fixable.is_empty() {
        apply_fixes(app, &report, args.yes)?;
    }

    let code = if !report.errors.is_empty() {
        2
    } else if !report.warnings.is_empty() || !report.fixable.is_empty() {
        1
    } else {
        0
    };
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

fn check_integrity(conn: &rusqlite::Connection, report: &mut Report) -> Result<(), BacklogError> {
    let result: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if result != "ok" {
        report
            .errors
            .push(format!("integrity_check falhou: {result}"));
    }
    Ok(())
}

fn check_user_version(
    conn: &rusqlite::Connection,
    report: &mut Report,
) -> Result<(), BacklogError> {
    let actual: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let expected = crate::db::migrations::EXPECTED_USER_VERSION;
    if actual != expected {
        report.errors.push(format!(
            "user_version divergente: DB={actual}, binário espera={expected}"
        ));
    }
    Ok(())
}

fn check_registry(app: &App, report: &mut Report) -> Result<(), BacklogError> {
    for entry in &app.registry.entries {
        if !entry.root_path.exists() {
            report.warnings.push(format!(
                "registry: path de project {} ({}) não existe: {}",
                entry.id,
                entry.name,
                entry.root_path.display()
            ));
            report.fixable.push(FixableIssue::RegistryMissingPath {
                id: entry.id,
                path: entry.root_path.display().to_string(),
            });
            continue;
        }
        let project = match project_repo::get_by_id(&app.conn, entry.id)? {
            Some(p) => p,
            None => {
                report.errors.push(format!(
                    "registry: project_id={} não existe na tabela projects",
                    entry.id
                ));
                report
                    .fixable
                    .push(FixableIssue::RegistryOrphanProject { id: entry.id });
                continue;
            }
        };

        // Divergência DB↔registry: root_path canonizado no registry precisa
        // bater com `projects.root_path`. Se diverge, o registry resolveria o
        // tenant para um projeto errado — erro não-auto-fixable (usa `relink`).
        let registry_canon =
            std::fs::canonicalize(&entry.root_path).unwrap_or_else(|_| entry.root_path.clone());
        let db_canon = std::fs::canonicalize(&project.root_path)
            .unwrap_or_else(|_| std::path::PathBuf::from(&project.root_path));
        if registry_canon != db_canon {
            report.errors.push(format!(
                "registry: project {} aponta para '{}' mas projects.root_path é '{}' \
                 (rode `backlog projects relink` para conciliar)",
                entry.id,
                registry_canon.display(),
                db_canon.display(),
            ));
        }
    }
    Ok(())
}

fn check_projects_vs_registry(app: &App, report: &mut Report) -> Result<(), BacklogError> {
    for project in project_repo::list_all(&app.conn)? {
        if project.archived_at.is_some() {
            continue;
        }
        let present = app.registry.entries.iter().any(|e| e.id == project.id);
        if !present {
            report.errors.push(format!(
                "projects: project {} '{}' ativo mas ausente do registry",
                project.id, project.name
            ));
        }
    }
    Ok(())
}

fn check_cross_tenant(
    conn: &rusqlite::Connection,
    report: &mut Report,
) -> Result<(), BacklogError> {
    // Os triggers já bloqueiam essas linhas na inserção — este check é
    // defense-in-depth contra DBs editados manualmente ou triggers dropados.
    let checks: &[(&str, &str)] = &[
        (
            "tasks.parent_id cross-project",
            "SELECT COUNT(*) FROM tasks c JOIN tasks p ON p.id = c.parent_id \
             WHERE c.parent_id IS NOT NULL AND c.project_id <> p.project_id",
        ),
        (
            "task_tags cross-project",
            "SELECT COUNT(*) FROM task_tags tt \
             JOIN tasks t ON t.id = tt.task_id \
             JOIN tags g ON g.id = tt.tag_id \
             WHERE t.project_id <> g.project_id",
        ),
        (
            "task_links cross-project",
            "SELECT COUNT(*) FROM task_links l \
             JOIN tasks a ON a.id = l.from_id \
             JOIN tasks b ON b.id = l.to_id \
             WHERE a.project_id <> b.project_id",
        ),
    ];
    for (label, sql) in checks {
        let count: i64 = conn
            .query_row(sql, [], |r| r.get(0))
            .optional()?
            .unwrap_or(0);
        if count > 0 {
            report
                .errors
                .push(format!("{label}: {count} linha(s) violando tenancy"));
        }
    }
    Ok(())
}

fn check_orphans(conn: &rusqlite::Connection, report: &mut Report) -> Result<(), BacklogError> {
    let queries: &[(&str, &str)] = &[
        (
            "task_tags",
            "SELECT COUNT(*) FROM task_tags tt \
             WHERE NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = tt.task_id)",
        ),
        (
            "task_attributes",
            "SELECT COUNT(*) FROM task_attributes a \
             WHERE NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = a.task_id)",
        ),
        (
            "task_links",
            "SELECT COUNT(*) FROM task_links l \
             WHERE NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = l.from_id) \
                OR NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = l.to_id)",
        ),
        (
            "task_events",
            "SELECT COUNT(*) FROM task_events e \
             WHERE NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = e.task_id)",
        ),
    ];
    for (table, sql) in queries {
        let count: i64 = conn
            .query_row(sql, [], |r| r.get(0))
            .optional()?
            .unwrap_or(0);
        if count > 0 {
            report
                .errors
                .push(format!("{table}: {count} órfãos sem task pai"));
        }
    }
    Ok(())
}

fn print_report(report: &Report) {
    if report.warnings.is_empty() && report.errors.is_empty() {
        stdout_data("ok — zero problemas detectados");
        return;
    }
    for w in &report.warnings {
        stderr_msg(format!("warn: {w}"));
    }
    for e in &report.errors {
        stderr_msg(format!("error: {e}"));
    }
    stdout_data(format!(
        "{} warnings, {} errors",
        report.warnings.len(),
        report.errors.len()
    ));
}

fn apply_fixes(app: &mut App, report: &Report, yes: bool) -> Result<(), BacklogError> {
    if !yes {
        let confirmed = Confirm::new(&format!(
            "aplicar {} correção(ões) automática(s) no registry?",
            report.fixable.len()
        ))
        .with_default(false)
        .prompt()
        .unwrap_or(false);
        if !confirmed {
            stderr_msg("nenhuma correção aplicada");
            return Ok(());
        }
    }

    let mut removed = 0;
    for issue in &report.fixable {
        match issue {
            FixableIssue::RegistryMissingPath { id, path } => {
                app.registry.remove(*id);
                removed += 1;
                stderr_msg(format!(
                    "registry: removido entry {id} (path ausente: {path})"
                ));
            }
            FixableIssue::RegistryOrphanProject { id } => {
                app.registry.remove(*id);
                removed += 1;
                stderr_msg(format!("registry: removido entry {id} (projeto órfão)"));
            }
        }
    }
    if removed > 0 {
        app.save_registry()?;
        stderr_msg(format!("{removed} entry(ies) removida(s) do registry"));
    }
    Ok(())
}
