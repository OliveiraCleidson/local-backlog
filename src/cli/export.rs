//! `backlog export` — dump do tenant atual em markdown ou JSON.
//!
//! Markdown serve como contexto para LLMs: agrupa por status na ordem do
//! `config.toml`, IDs prefixados com `T-`, tags como `#hashtag`, priority
//! entre colchetes. JSON é um envelope completo para consumo programático.

use std::fmt::Write as _;
use std::path::Path;

use clap::Args;
use rusqlite::params;
use serde::Serialize;

use crate::bootstrap::App;
use crate::cli::{resolve_tenant, validate_enum};
use crate::db::events;
use crate::db::repo::{attr_repo, link_repo, project_repo, tag_repo, task_repo};
use crate::domain::Task;
use crate::error::BacklogError;
use crate::format::{Format, SCHEMA_VERSION};
use crate::output::stdout_data;

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Formato: `markdown` (default) ou `json`.
    #[arg(long, default_value = "markdown")]
    pub format: String,

    /// Filtra por status (CSV: `todo,doing`).
    #[arg(long, value_delimiter = ',')]
    pub status: Vec<String>,

    /// Filtra por tag (nome exato, CSV aceito).
    #[arg(long, value_delimiter = ',')]
    pub tag: Vec<String>,

    /// Filtra por tipo (CSV aceito).
    #[arg(long = "type", value_name = "TYPE", value_delimiter = ',')]
    pub task_type: Vec<String>,

    /// Inclui tasks arquivadas (default: oculta).
    #[arg(long, default_value_t = false)]
    pub include_archived: bool,

    /// Inclui `body` das tasks no dump.
    #[arg(long, default_value_t = false)]
    pub include_body: bool,

    /// Inclui timeline de eventos por task.
    #[arg(long, default_value_t = false)]
    pub include_events: bool,

    /// Inclui apenas tasks com `updated_at >= SINCE` (ex.: `2026-04-01`).
    #[arg(long)]
    pub since: Option<String>,

    /// Inclui apenas tasks com `updated_at <= UNTIL` (ex.: `2026-04-30`).
    #[arg(long)]
    pub until: Option<String>,
}

pub fn run(args: ExportArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    let fmt = match args.format.as_str() {
        "markdown" | "md" => Format::Table, // reutilizamos o enum por conveniência
        "json" => Format::Json,
        other => {
            return Err(BacklogError::InvalidEnum {
                field: "format",
                value: other.to_string(),
                allowed: "markdown, json".to_string(),
            });
        }
    };

    for s in &args.status {
        validate_enum("status", s, &app.config.status.values)?;
    }
    for t in &args.task_type {
        validate_enum("type", t, &app.config.task_type.values)?;
    }

    let rows = collect_rows(app, tenant.project_id, &args)?;

    let project = project_repo::get_by_id(&app.conn, tenant.project_id)?.ok_or_else(|| {
        BacklogError::ProjectNotFound {
            name: tenant.name.clone(),
        }
    })?;

    let out = match fmt {
        Format::Table => render_markdown(
            &project.name,
            &app.config.status.values,
            &rows,
            args.include_body,
        ),
        Format::Json => render_json(&project, &rows, args.include_body)?,
    };
    let trimmed = out.strip_suffix('\n').unwrap_or(&out);
    stdout_data(trimmed);
    Ok(())
}

/// Status terminais ocultados por default no export — só aparecem quando o
/// usuário passa `--status` explicitamente ou `--include-archived`.
const TERMINAL_STATUSES: &[&str] = &["done", "cancelled"];

fn is_terminal_status(s: &str) -> bool {
    TERMINAL_STATUSES.contains(&s)
}

/// Se `s` é `YYYY-MM-DD`, devolve `YYYY-MM-DD 00:00:00`; caso contrário devolve `s`.
fn expand_date_lower(s: &str) -> String {
    if looks_like_date(s) {
        format!("{s} 00:00:00")
    } else {
        s.to_string()
    }
}

/// Se `s` é `YYYY-MM-DD`, devolve `YYYY-MM-DD 23:59:59`; caso contrário devolve `s`.
fn expand_date_upper(s: &str) -> String {
    if looks_like_date(s) {
        format!("{s} 23:59:59")
    } else {
        s.to_string()
    }
}

fn looks_like_date(s: &str) -> bool {
    s.len() == 10
        && s.as_bytes()[4] == b'-'
        && s.as_bytes()[7] == b'-'
        && s.bytes()
            .enumerate()
            .all(|(i, b)| matches!(i, 4 | 7) || b.is_ascii_digit())
}

/// Bundle completo de uma task já com satélites carregados.
struct ExportRow {
    task: Task,
    tags: Vec<String>,
    attrs: Vec<(String, String)>,
    links_out: Vec<link_repo::Link>,
    links_in: Vec<link_repo::Link>,
    events: Vec<events::TaskEvent>,
}

fn collect_rows(
    app: &App,
    project_id: i64,
    args: &ExportArgs,
) -> Result<Vec<ExportRow>, BacklogError> {
    // Deriva filtro base; tag/status podem ser múltiplos, então filtramos após query.
    let filter = task_repo::ListFilter {
        status: None,
        tag: None,
        task_type: None,
        priority: None,
        parent_id: None,
        include_archived: args.include_archived,
        limit: None,
        priority_order: Some(app.config.priority.order),
    };
    let tasks = task_repo::list(&app.conn, project_id, &filter)?;

    // Normaliza filtros de data: data-only vira intervalo inclusivo do dia.
    // `since="2026-04-01"` → compara contra `updated_at >= "2026-04-01 00:00:00"`;
    // `until="2026-04-30"` → compara contra `updated_at <= "2026-04-30 23:59:59"`.
    let since_norm = args.since.as_deref().map(expand_date_lower);
    let until_norm = args.until.as_deref().map(expand_date_upper);

    let hide_terminal = args.status.is_empty() && !args.include_archived;

    let mut out = Vec::new();
    for task in tasks {
        if !args.status.is_empty() && !args.status.iter().any(|s| s == &task.status) {
            continue;
        }
        if hide_terminal && is_terminal_status(&task.status) {
            continue;
        }
        if !args.task_type.is_empty() {
            let Some(tt) = task.task_type.as_deref() else {
                continue;
            };
            if !args.task_type.iter().any(|t| t == tt) {
                continue;
            }
        }

        let tag_names: Vec<String> = tag_repo::list_for_task(&app.conn, project_id, task.id)?
            .into_iter()
            .map(|t| t.name)
            .collect();
        if !args.tag.is_empty() && !args.tag.iter().any(|t| tag_names.iter().any(|n| n == t)) {
            continue;
        }
        if let Some(since) = since_norm.as_deref() {
            if task.updated_at.as_str() < since {
                continue;
            }
        }
        if let Some(until) = until_norm.as_deref() {
            if task.updated_at.as_str() > until {
                continue;
            }
        }

        let attrs = attr_repo::list_for_task(&app.conn, project_id, task.id)?;
        let links_out = links_out_for(&app.conn, project_id, task.id)?;
        let links_in = links_in_for(&app.conn, project_id, task.id)?;
        let ev = if args.include_events {
            events::list_for_task(&app.conn, project_id, task.id, u32::MAX)?
        } else {
            Vec::new()
        };

        out.push(ExportRow {
            task,
            tags: tag_names,
            attrs,
            links_out,
            links_in,
            events: ev,
        });
    }
    Ok(out)
}

fn links_out_for(
    conn: &rusqlite::Connection,
    project_id: i64,
    from_id: i64,
) -> Result<Vec<link_repo::Link>, BacklogError> {
    let mut stmt = conn.prepare(
        "SELECT l.from_id, l.to_id, l.kind FROM task_links l \
         JOIN tasks t ON t.id = l.from_id \
         WHERE l.from_id = ?1 AND t.project_id = ?2 \
         ORDER BY l.kind ASC, l.to_id ASC",
    )?;
    let rows = stmt.query_map(params![from_id, project_id], |r| {
        Ok(link_repo::Link {
            from_id: r.get(0)?,
            to_id: r.get(1)?,
            kind: r.get(2)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn links_in_for(
    conn: &rusqlite::Connection,
    project_id: i64,
    to_id: i64,
) -> Result<Vec<link_repo::Link>, BacklogError> {
    let mut stmt = conn.prepare(
        "SELECT l.from_id, l.to_id, l.kind FROM task_links l \
         JOIN tasks t ON t.id = l.to_id \
         WHERE l.to_id = ?1 AND t.project_id = ?2 \
         ORDER BY l.kind ASC, l.from_id ASC",
    )?;
    let rows = stmt.query_map(params![to_id, project_id], |r| {
        Ok(link_repo::Link {
            from_id: r.get(0)?,
            to_id: r.get(1)?,
            kind: r.get(2)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn render_markdown(
    project_name: &str,
    status_order: &[String],
    rows: &[ExportRow],
    include_body: bool,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {project_name}");
    let _ = writeln!(out);

    if rows.is_empty() {
        let _ = writeln!(out, "_sem tasks_");
        return out;
    }

    for status in status_order {
        let bucket: Vec<&ExportRow> = rows.iter().filter(|r| r.task.status == *status).collect();
        if bucket.is_empty() {
            continue;
        }
        let _ = writeln!(out, "## {status}");
        let _ = writeln!(out);
        for row in bucket {
            render_row_markdown(&mut out, row, include_body);
        }
        let _ = writeln!(out);
    }

    // tasks com status fora da whitelist (defensivo) vão ao final.
    let fallback: Vec<&ExportRow> = rows
        .iter()
        .filter(|r| !status_order.iter().any(|s| s == &r.task.status))
        .collect();
    if !fallback.is_empty() {
        let _ = writeln!(out, "## other");
        let _ = writeln!(out);
        for row in fallback {
            render_row_markdown(&mut out, row, include_body);
        }
    }
    out
}

fn render_row_markdown(out: &mut String, row: &ExportRow, include_body: bool) {
    let tags = if row.tags.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            row.tags
                .iter()
                .map(|t| format!("#{t}"))
                .collect::<Vec<_>>()
                .join(" ")
        )
    };
    let typ = row
        .task
        .task_type
        .as_deref()
        .map(|t| format!(" ({t})"))
        .unwrap_or_default();
    let archived = if row.task.archived_at.is_some() {
        " _(archived)_"
    } else {
        ""
    };
    let _ = writeln!(
        out,
        "- T-{} [{}]{} {}{}{}",
        row.task.id, row.task.priority, typ, row.task.title, tags, archived
    );
    if include_body {
        if let Some(body) = row.task.body.as_deref() {
            if !body.trim().is_empty() {
                for line in body.lines() {
                    let _ = writeln!(out, "  > {line}");
                }
            }
        }
    }
    if !row.attrs.is_empty() {
        let joined = row
            .attrs
            .iter()
            .map(|(k, v)| format!("`{k}={v}`"))
            .collect::<Vec<_>>()
            .join(" ");
        let _ = writeln!(out, "  - attrs: {joined}");
    }
    if !row.links_out.is_empty() {
        let joined = row
            .links_out
            .iter()
            .map(|l| format!("{} T-{}", l.kind, l.to_id))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(out, "  - links: {joined}");
    }
    if !row.links_in.is_empty() {
        let joined = row
            .links_in
            .iter()
            .map(|l| format!("T-{} {}", l.from_id, l.kind))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(out, "  - backlinks: {joined}");
    }
    if !row.events.is_empty() {
        let _ = writeln!(out, "  - events:");
        for ev in &row.events {
            let _ = writeln!(out, "    - {} `{}` {}", ev.created_at, ev.kind, ev.payload);
        }
    }
}

#[derive(Serialize)]
struct ExportJsonProject<'a> {
    id: i64,
    name: &'a str,
    root_path: &'a str,
    description: Option<&'a str>,
    archived_at: Option<&'a str>,
}

#[derive(Serialize)]
struct ExportJsonTask<'a> {
    id: i64,
    title: &'a str,
    body: Option<&'a str>,
    status: &'a str,
    priority: i64,
    #[serde(rename = "type")]
    task_type: Option<&'a str>,
    parent_id: Option<i64>,
    archived_at: Option<&'a str>,
    completed_at: Option<&'a str>,
    created_at: &'a str,
    updated_at: &'a str,
    tags: &'a [String],
    attributes: Vec<ExportJsonAttr<'a>>,
    links_out: &'a [link_repo::Link],
    links_in: &'a [link_repo::Link],
    events: Vec<ExportJsonEvent<'a>>,
}

#[derive(Serialize)]
struct ExportJsonAttr<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Serialize)]
struct ExportJsonEvent<'a> {
    id: i64,
    kind: &'a str,
    payload: &'a serde_json::Value,
    created_at: &'a str,
}

#[derive(Serialize)]
struct ExportJsonPayload<'a> {
    schema_version: u32,
    project: ExportJsonProject<'a>,
    tasks: Vec<ExportJsonTask<'a>>,
}

fn render_json(
    project: &crate::domain::Project,
    rows: &[ExportRow],
    include_body: bool,
) -> Result<String, BacklogError> {
    let tasks: Vec<ExportJsonTask> = rows
        .iter()
        .map(|r| ExportJsonTask {
            id: r.task.id,
            title: &r.task.title,
            body: if include_body {
                r.task.body.as_deref()
            } else {
                None
            },
            status: &r.task.status,
            priority: r.task.priority,
            task_type: r.task.task_type.as_deref(),
            parent_id: r.task.parent_id,
            archived_at: r.task.archived_at.as_deref(),
            completed_at: r.task.completed_at.as_deref(),
            created_at: &r.task.created_at,
            updated_at: &r.task.updated_at,
            tags: &r.tags,
            attributes: r
                .attrs
                .iter()
                .map(|(k, v)| ExportJsonAttr { key: k, value: v })
                .collect(),
            links_out: &r.links_out,
            links_in: &r.links_in,
            events: r
                .events
                .iter()
                .map(|e| ExportJsonEvent {
                    id: e.id,
                    kind: &e.kind,
                    payload: &e.payload,
                    created_at: &e.created_at,
                })
                .collect(),
        })
        .collect();

    let payload = ExportJsonPayload {
        schema_version: SCHEMA_VERSION,
        project: ExportJsonProject {
            id: project.id,
            name: &project.name,
            root_path: &project.root_path,
            description: project.description.as_deref(),
            archived_at: project.archived_at.as_deref(),
        },
        tasks,
    };
    serde_json::to_string_pretty(&payload)
        .map_err(|e| BacklogError::InvalidInput(format!("falha ao serializar export: {e}")))
}
