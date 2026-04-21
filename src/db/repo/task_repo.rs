//! CRUD de tasks dentro de um `project_id` (primeiro parâmetro explícito).

use rusqlite::{params_from_iter, types::Value, Connection, OptionalExtension, Row};

use crate::config::PriorityOrder;
use crate::domain::Task;
use crate::error::BacklogError;

const COLS: &str = "id, project_id, title, body, status, priority, type as task_type, \
                    parent_id, archived_at, completed_at, created_at, updated_at";

fn row_to_task(row: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        body: row.get("body")?,
        status: row.get("status")?,
        priority: row.get("priority")?,
        task_type: row.get("task_type")?,
        parent_id: row.get("parent_id")?,
        archived_at: row.get("archived_at")?,
        completed_at: row.get("completed_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

#[derive(Debug, Clone)]
pub struct NewTask {
    pub title: String,
    pub body: Option<String>,
    pub status: String,
    pub priority: i64,
    pub task_type: Option<String>,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    pub status: Option<String>,
    pub tag: Option<String>,
    pub task_type: Option<String>,
    pub priority: Option<i64>,
    pub parent_id: Option<i64>,
    pub include_archived: bool,
    pub limit: Option<u32>,
    pub priority_order: Option<PriorityOrder>,
}

/// Patch parcial. `Some(None)` zera o campo; `None` mantém o valor.
#[derive(Debug, Clone, Default)]
pub struct TaskPatch {
    pub title: Option<String>,
    pub body: Option<Option<String>>,
    pub status: Option<String>,
    pub priority: Option<i64>,
    pub task_type: Option<Option<String>>,
    pub parent_id: Option<Option<i64>>,
}

pub fn insert(conn: &Connection, project_id: i64, new: &NewTask) -> Result<Task, BacklogError> {
    conn.execute(
        "INSERT INTO tasks (project_id, title, body, status, priority, type, parent_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            project_id,
            new.title,
            new.body,
            new.status,
            new.priority,
            new.task_type,
            new.parent_id,
        ],
    )?;
    let id = conn.last_insert_rowid();
    get(conn, project_id, id)?.ok_or(BacklogError::TaskNotFound { id })
}

/// Retorna a task apenas se ela pertence ao `project_id`. Caso contrário
/// (não existe ou é de outro tenant), devolve `None` — tenant-leak policy.
pub fn get(conn: &Connection, project_id: i64, id: i64) -> Result<Option<Task>, BacklogError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLS} FROM tasks WHERE id = ?1 AND project_id = ?2"
    ))?;
    let task = stmt
        .query_row(rusqlite::params![id, project_id], row_to_task)
        .optional()?;
    Ok(task)
}

pub fn exists(conn: &Connection, project_id: i64, id: i64) -> Result<bool, BacklogError> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM tasks WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![id, project_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

pub fn list(
    conn: &Connection,
    project_id: i64,
    filter: &ListFilter,
) -> Result<Vec<Task>, BacklogError> {
    let mut sql = format!("SELECT {COLS} FROM tasks t WHERE t.project_id = ?1");
    let mut args: Vec<Value> = vec![Value::Integer(project_id)];

    if !filter.include_archived {
        sql.push_str(" AND t.archived_at IS NULL");
    }
    if let Some(status) = &filter.status {
        args.push(Value::Text(status.clone()));
        sql.push_str(&format!(" AND t.status = ?{}", args.len()));
    }
    if let Some(task_type) = &filter.task_type {
        args.push(Value::Text(task_type.clone()));
        sql.push_str(&format!(" AND t.type = ?{}", args.len()));
    }
    if let Some(priority) = filter.priority {
        args.push(Value::Integer(priority));
        sql.push_str(&format!(" AND t.priority = ?{}", args.len()));
    }
    if let Some(parent) = filter.parent_id {
        args.push(Value::Integer(parent));
        sql.push_str(&format!(" AND t.parent_id = ?{}", args.len()));
    }
    if let Some(tag) = &filter.tag {
        args.push(Value::Text(tag.clone()));
        sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM task_tags tt JOIN tags g ON g.id = tt.tag_id \
                         WHERE tt.task_id = t.id AND g.name = ?{})",
            args.len()
        ));
    }

    let order = filter.priority_order.unwrap_or(PriorityOrder::Asc).sql();
    sql.push_str(&format!(
        " ORDER BY t.priority {order}, t.updated_at DESC, t.id ASC"
    ));
    if let Some(limit) = filter.limit {
        sql.push_str(&format!(" LIMIT {limit}"));
    }

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(args.iter()), row_to_task)?;
    let out = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(out)
}

pub fn update_fields(
    conn: &Connection,
    project_id: i64,
    id: i64,
    patch: &TaskPatch,
) -> Result<Task, BacklogError> {
    // Confirma que task existe no tenant antes de update.
    if !exists(conn, project_id, id)? {
        return Err(BacklogError::TaskNotFound { id });
    }

    let mut sets: Vec<String> = Vec::new();
    let mut args: Vec<Value> = Vec::new();

    macro_rules! push {
        ($col:expr, $val:expr) => {{
            args.push($val);
            sets.push(format!("{} = ?{}", $col, args.len()));
        }};
    }

    if let Some(title) = &patch.title {
        push!("title", Value::Text(title.clone()));
    }
    if let Some(body) = &patch.body {
        push!(
            "body",
            match body {
                Some(v) => Value::Text(v.clone()),
                None => Value::Null,
            }
        );
    }
    if let Some(status) = &patch.status {
        push!("status", Value::Text(status.clone()));
    }
    if let Some(priority) = patch.priority {
        push!("priority", Value::Integer(priority));
    }
    if let Some(tt) = &patch.task_type {
        push!(
            "type",
            match tt {
                Some(v) => Value::Text(v.clone()),
                None => Value::Null,
            }
        );
    }
    if let Some(parent) = &patch.parent_id {
        push!(
            "parent_id",
            match parent {
                Some(v) => Value::Integer(*v),
                None => Value::Null,
            }
        );
    }

    if sets.is_empty() {
        return get(conn, project_id, id)?.ok_or(BacklogError::TaskNotFound { id });
    }

    sets.push("updated_at = datetime('now')".to_string());
    args.push(Value::Integer(id));
    let id_idx = args.len();
    args.push(Value::Integer(project_id));
    let project_idx = args.len();

    let sql = format!(
        "UPDATE tasks SET {} WHERE id = ?{} AND project_id = ?{}",
        sets.join(", "),
        id_idx,
        project_idx
    );
    conn.execute(&sql, params_from_iter(args.iter()))?;
    get(conn, project_id, id)?.ok_or(BacklogError::TaskNotFound { id })
}

/// Retorna `(task_atualizada, status_mudou)`.
pub fn set_status(
    conn: &Connection,
    project_id: i64,
    id: i64,
    new_status: &str,
    mark_completed: bool,
) -> Result<(Task, bool), BacklogError> {
    let current = get(conn, project_id, id)?.ok_or(BacklogError::TaskNotFound { id })?;
    if current.status == new_status {
        return Ok((current, false));
    }
    let completed = if mark_completed {
        "datetime('now')"
    } else {
        "completed_at"
    };
    conn.execute(
        &format!(
            "UPDATE tasks SET status = ?1, completed_at = {completed}, \
                             updated_at = datetime('now') \
             WHERE id = ?2 AND project_id = ?3"
        ),
        rusqlite::params![new_status, id, project_id],
    )?;
    let updated = get(conn, project_id, id)?.ok_or(BacklogError::TaskNotFound { id })?;
    Ok((updated, true))
}

/// Retorna `true` se o valor de `archived_at` foi alterado.
pub fn set_archived(conn: &Connection, project_id: i64, id: i64) -> Result<bool, BacklogError> {
    let current = get(conn, project_id, id)?.ok_or(BacklogError::TaskNotFound { id })?;
    if current.archived_at.is_some() {
        return Ok(false);
    }
    conn.execute(
        "UPDATE tasks SET archived_at = datetime('now'), updated_at = datetime('now') \
         WHERE id = ?1 AND project_id = ?2",
        rusqlite::params![id, project_id],
    )?;
    Ok(true)
}
