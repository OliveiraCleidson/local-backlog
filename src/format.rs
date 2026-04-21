//! Renderers e envelope JSON (ADR-0004). `stdout_data` aplica newline final.

use std::io::IsTerminal;

use owo_colors::OwoColorize;
use serde::Serialize;

use crate::domain::{Tag, Task};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Table,
    Json,
}

impl Format {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "table" => Some(Self::Table),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JsonEnvelope<T: Serialize> {
    pub schema_version: u32,
    pub data: T,
}

impl<T: Serialize> JsonEnvelope<T> {
    pub fn new(data: T) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            data,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TaskRow<'a> {
    pub id: i64,
    pub title: &'a str,
    pub status: &'a str,
    pub priority: i64,
    #[serde(rename = "type")]
    pub task_type: Option<&'a str>,
    pub parent_id: Option<i64>,
    pub archived_at: Option<&'a str>,
    pub completed_at: Option<&'a str>,
    pub tags: Vec<&'a str>,
}

impl<'a> TaskRow<'a> {
    pub fn from(task: &'a Task, tags: &'a [Tag]) -> Self {
        Self {
            id: task.id,
            title: &task.title,
            status: &task.status,
            priority: task.priority,
            task_type: task.task_type.as_deref(),
            parent_id: task.parent_id,
            archived_at: task.archived_at.as_deref(),
            completed_at: task.completed_at.as_deref(),
            tags: tags.iter().map(|t| t.name.as_str()).collect(),
        }
    }
}

pub fn render_tasks_json(tasks: &[(Task, Vec<Tag>)]) -> String {
    let rows: Vec<TaskRow> = tasks
        .iter()
        .map(|(t, tags)| TaskRow::from(t, tags))
        .collect();
    let envelope = JsonEnvelope::new(rows);
    serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| "{}".to_string())
}

/// Tabela determinística. Cores desligadas quando `stdout` não é TTY.
pub fn render_tasks_table(tasks: &[(Task, Vec<Tag>)]) -> String {
    if tasks.is_empty() {
        return "nenhuma task encontrada\n".to_string();
    }

    let color = std::io::stdout().is_terminal();

    // Larguras mínimas por cabeçalho.
    let mut w_id = "ID".len();
    let mut w_pri = "PRI".len();
    let mut w_stat = "STATUS".len();
    let mut w_type = "TYPE".len();
    let mut w_title = "TITLE".len();

    let prepped: Vec<(String, String, String, String, String, String)> = tasks
        .iter()
        .map(|(t, tags)| {
            let id = t.id.to_string();
            let pri = t.priority.to_string();
            let stat = t.status.clone();
            let typ = t.task_type.clone().unwrap_or_else(|| "-".into());
            let title = t.title.clone();
            let tagstr = if tags.is_empty() {
                String::new()
            } else {
                tags.iter()
                    .map(|g| format!("#{}", g.name))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            w_id = w_id.max(id.len());
            w_pri = w_pri.max(pri.len());
            w_stat = w_stat.max(stat.len());
            w_type = w_type.max(typ.len());
            w_title = w_title.max(title.len());
            (id, pri, stat, typ, title, tagstr)
        })
        .collect();

    let mut out = String::new();
    out.push_str(&format!(
        "{:>w_id$}  {:>w_pri$}  {:<w_stat$}  {:<w_type$}  {:<w_title$}  {}\n",
        "ID",
        "PRI",
        "STATUS",
        "TYPE",
        "TITLE",
        "TAGS",
        w_id = w_id,
        w_pri = w_pri,
        w_stat = w_stat,
        w_type = w_type,
        w_title = w_title,
    ));

    for (id, pri, stat, typ, title, tagstr) in &prepped {
        let stat_rendered = if color {
            color_status(stat)
        } else {
            stat.clone()
        };
        // Ajuste de padding: owo-colors adiciona escape codes que quebram
        // width do `format!`. Aplicamos padding manualmente sobre o texto.
        let stat_padded = pad_right(&stat_rendered, stat.chars().count(), w_stat);
        out.push_str(&format!(
            "{:>w_id$}  {:>w_pri$}  {}  {:<w_type$}  {:<w_title$}  {}\n",
            id,
            pri,
            stat_padded,
            typ,
            title,
            tagstr,
            w_id = w_id,
            w_pri = w_pri,
            w_type = w_type,
            w_title = w_title,
        ));
    }
    out
}

fn color_status(status: &str) -> String {
    match status {
        "todo" => status.blue().to_string(),
        "doing" => status.yellow().to_string(),
        "blocked" => status.red().to_string(),
        "done" => status.green().to_string(),
        "cancelled" => status.bright_black().to_string(),
        _ => status.to_string(),
    }
}

fn pad_right(rendered: &str, visible_len: usize, target: usize) -> String {
    if visible_len >= target {
        rendered.to_string()
    } else {
        let mut s = rendered.to_string();
        s.push_str(&" ".repeat(target - visible_len));
        s
    }
}
