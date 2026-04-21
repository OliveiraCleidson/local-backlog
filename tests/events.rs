use local_backlog::db;
use local_backlog::db::events;
use local_backlog::db::repo::project_repo;
use rusqlite::params;
use serde_json::json;

fn seed_task(conn: &rusqlite::Connection) -> i64 {
    let p = project_repo::insert(conn, "p", "/tmp/p", None).unwrap();
    conn.execute(
        "INSERT INTO tasks (project_id, title, status) VALUES (?1, 'first', 'todo')",
        params![p.id],
    )
    .unwrap();
    conn.last_insert_rowid()
}

#[test]
fn emit_inserts_row_with_json_payload() {
    let conn = db::open_in_memory().unwrap();
    let task_id = seed_task(&conn);

    events::emit(
        &conn,
        task_id,
        "created",
        &json!({"title": "first", "type": null, "priority": 100}),
    )
    .unwrap();

    let (kind, payload): (String, String) = conn
        .query_row(
            "SELECT kind, payload FROM task_events WHERE task_id = ?1",
            params![task_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(kind, "created");
    let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(parsed["title"], "first");
    assert_eq!(parsed["priority"], 100);
}

#[test]
fn emit_bare_uses_empty_object() {
    let conn = db::open_in_memory().unwrap();
    let task_id = seed_task(&conn);

    events::emit_bare(&conn, task_id, "archived").unwrap();

    let payload: String = conn
        .query_row(
            "SELECT payload FROM task_events WHERE task_id = ?1 AND kind = 'archived'",
            params![task_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(payload, "{}");
}
