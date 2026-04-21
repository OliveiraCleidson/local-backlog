//! `backlog show <ID>` — visão agregada de uma task.

use std::path::Path;

use clap::Args;
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::bootstrap::App;
use crate::cli::resolve_tenant;
use crate::db::events::{self, TaskEvent};
use crate::db::repo::{tag_repo, task_repo};
use crate::domain::{Tag, Task};
use crate::error::BacklogError;
use crate::format::{Format, JsonEnvelope};
use crate::output::stdout_data;

const EVENT_LIMIT: u32 = 10;

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// ID da task.
    pub id: i64,

    /// Formato: `table` (default) ou `json`.
    #[arg(long, default_value = "table")]
    pub format: String,
}

#[derive(Debug, Serialize)]
struct Attribute {
    key: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct Link {
    direction: &'static str, // "out" | "in"
    kind: String,
    other_id: i64,
}

#[derive(Debug, Serialize)]
struct ShowData {
    task: Task,
    tags: Vec<Tag>,
    attributes: Vec<Attribute>,
    links: Vec<Link>,
    events: Vec<TaskEvent>,
}

pub fn run(args: ShowArgs, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    let tenant = resolve_tenant(app, cwd)?;

    let fmt = Format::parse(&args.format).ok_or_else(|| BacklogError::InvalidEnum {
        field: "format",
        value: args.format.clone(),
        allowed: "table, json".to_string(),
    })?;

    // Tenant-leak policy: cross-tenant id devolve mesma mensagem de "não existe".
    let task = task_repo::get(&app.conn, tenant.project_id, args.id)?
        .ok_or(BacklogError::TaskNotFound { id: args.id })?;

    let tags = tag_repo::list_for_task(&app.conn, tenant.project_id, task.id)?;
    let attributes = list_attributes(&app.conn, tenant.project_id, task.id)?;
    let links = list_links(&app.conn, tenant.project_id, task.id)?;
    let events = events::list_for_task(&app.conn, tenant.project_id, task.id, EVENT_LIMIT)?;

    let data = ShowData {
        task,
        tags,
        attributes,
        links,
        events,
    };

    let out = match fmt {
        Format::Json => serde_json::to_string_pretty(&JsonEnvelope::new(&data))
            .unwrap_or_else(|_| "{}".to_string()),
        Format::Table => render_table(&data),
    };
    let trimmed = out.strip_suffix('\n').unwrap_or(&out);
    stdout_data(trimmed);
    Ok(())
}

fn list_attributes(
    conn: &Connection,
    project_id: i64,
    task_id: i64,
) -> Result<Vec<Attribute>, BacklogError> {
    let mut stmt = conn.prepare(
        "SELECT a.key, a.value FROM task_attributes a \
         JOIN tasks t ON t.id = a.task_id \
         WHERE a.task_id = ?1 AND t.project_id = ?2 \
         ORDER BY a.key ASC",
    )?;
    let rows = stmt.query_map(params![task_id, project_id], |r| {
        Ok(Attribute {
            key: r.get(0)?,
            value: r.get(1)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn list_links(conn: &Connection, project_id: i64, task_id: i64) -> Result<Vec<Link>, BacklogError> {
    let mut out = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT l.kind, l.to_id FROM task_links l \
         JOIN tasks t ON t.id = l.from_id \
         WHERE l.from_id = ?1 AND t.project_id = ?2 \
         ORDER BY l.kind, l.to_id",
    )?;
    let rows = stmt.query_map(params![task_id, project_id], |r| {
        Ok(Link {
            direction: "out",
            kind: r.get(0)?,
            other_id: r.get(1)?,
        })
    })?;
    for r in rows {
        out.push(r?);
    }

    let mut stmt = conn.prepare(
        "SELECT l.kind, l.from_id FROM task_links l \
         JOIN tasks t ON t.id = l.to_id \
         WHERE l.to_id = ?1 AND t.project_id = ?2 \
         ORDER BY l.kind, l.from_id",
    )?;
    let rows = stmt.query_map(params![task_id, project_id], |r| {
        Ok(Link {
            direction: "in",
            kind: r.get(0)?,
            other_id: r.get(1)?,
        })
    })?;
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn render_table(d: &ShowData) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let t = &d.task;
    let _ = writeln!(s, "id:       {}", t.id);
    let _ = writeln!(s, "title:    {}", t.title);
    let _ = writeln!(s, "status:   {}", t.status);
    let _ = writeln!(s, "priority: {}", t.priority);
    if let Some(tt) = &t.task_type {
        let _ = writeln!(s, "type:     {tt}");
    }
    if let Some(p) = t.parent_id {
        let _ = writeln!(s, "parent:   {p}");
    }
    if let Some(a) = &t.archived_at {
        let _ = writeln!(s, "archived: {a}");
    }
    if let Some(c) = &t.completed_at {
        let _ = writeln!(s, "done:     {c}");
    }
    let _ = writeln!(s, "created:  {}", t.created_at);
    let _ = writeln!(s, "updated:  {}", t.updated_at);

    if let Some(body) = &t.body {
        s.push_str("\nbody:\n");
        s.push_str(body);
        s.push('\n');
    }

    if !d.tags.is_empty() {
        s.push_str("\ntags: ");
        let names: Vec<String> = d.tags.iter().map(|g| format!("#{}", g.name)).collect();
        s.push_str(&names.join(" "));
        s.push('\n');
    }

    if !d.attributes.is_empty() {
        s.push_str("\nattributes:\n");
        for a in &d.attributes {
            let _ = writeln!(s, "  {} = {}", a.key, a.value);
        }
    }

    if !d.links.is_empty() {
        s.push_str("\nlinks:\n");
        for l in &d.links {
            let arrow = if l.direction == "out" { "→" } else { "←" };
            let _ = writeln!(s, "  {} {} {}", arrow, l.kind, l.other_id);
        }
    }

    if !d.events.is_empty() {
        s.push_str("\nevents:\n");
        for e in &d.events {
            let _ = writeln!(s, "  {}  {}  {}", e.created_at, e.kind, e.payload);
        }
    }

    s
}
