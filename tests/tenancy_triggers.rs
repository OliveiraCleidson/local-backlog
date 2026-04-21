//! Testes dos 6 triggers de tenancy (ADR-0001) garantindo que
//! relacionamentos cross-project são bloqueados com `RAISE(ABORT, ...)`.

use local_backlog::db;
use rusqlite::Connection;

fn setup() -> Connection {
    let conn = db::open_in_memory().expect("migrations aplicadas");
    conn.execute(
        "INSERT INTO projects (name, root_path) VALUES ('p1', '/tmp/p1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO projects (name, root_path) VALUES ('p2', '/tmp/p2')",
        [],
    )
    .unwrap();
    conn
}

fn insert_task(conn: &Connection, project_id: i64, title: &str) -> i64 {
    conn.execute(
        "INSERT INTO tasks (project_id, title, status) VALUES (?1, ?2, 'todo')",
        rusqlite::params![project_id, title],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn insert_tag(conn: &Connection, project_id: i64, name: &str) -> i64 {
    conn.execute(
        "INSERT INTO tags (project_id, name) VALUES (?1, ?2)",
        rusqlite::params![project_id, name],
    )
    .unwrap();
    conn.last_insert_rowid()
}

#[test]
fn tasks_parent_cross_project_insert_is_blocked() {
    let conn = setup();
    let parent = insert_task(&conn, 1, "parent");
    let err = conn
        .execute(
            "INSERT INTO tasks (project_id, title, status, parent_id) \
             VALUES (2, 'child', 'todo', ?1)",
            [parent],
        )
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("parent e child de projetos diferentes"));
}

#[test]
fn tasks_parent_cross_project_update_is_blocked() {
    let conn = setup();
    let parent = insert_task(&conn, 1, "parent");
    let child = insert_task(&conn, 2, "child");
    let err = conn
        .execute(
            "UPDATE tasks SET parent_id = ?1 WHERE id = ?2",
            [parent, child],
        )
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("parent e child de projetos diferentes"));
}

#[test]
fn task_tags_cross_project_insert_is_blocked() {
    let conn = setup();
    let task = insert_task(&conn, 1, "t");
    let tag = insert_tag(&conn, 2, "auth");
    let err = conn
        .execute(
            "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
            [task, tag],
        )
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("tag e task de projetos diferentes"));
}

#[test]
fn task_tags_cross_project_update_is_blocked() {
    let conn = setup();
    let t1 = insert_task(&conn, 1, "t1");
    let t2 = insert_task(&conn, 2, "t2");
    let tag1 = insert_tag(&conn, 1, "auth");
    let tag2 = insert_tag(&conn, 2, "auth");
    conn.execute(
        "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
        [t1, tag1],
    )
    .unwrap();
    // Mover o tag_id para um tag de outro projeto é cross-project.
    let err = conn
        .execute(
            "UPDATE task_tags SET tag_id = ?1 WHERE task_id = ?2",
            [tag2, t1],
        )
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("tag e task de projetos diferentes"));
    // Silencia unused warnings em caso de curto-circuito.
    let _ = t2;
}

#[test]
fn task_links_cross_project_insert_is_blocked() {
    let conn = setup();
    let a = insert_task(&conn, 1, "a");
    let b = insert_task(&conn, 2, "b");
    let err = conn
        .execute(
            "INSERT INTO task_links (from_id, to_id, kind) VALUES (?1, ?2, 'blocks')",
            [a, b],
        )
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("links entre projetos não são permitidos"));
}

#[test]
fn task_links_cross_project_update_is_blocked() {
    let conn = setup();
    let a1 = insert_task(&conn, 1, "a1");
    let a2 = insert_task(&conn, 1, "a2");
    let b = insert_task(&conn, 2, "b");
    conn.execute(
        "INSERT INTO task_links (from_id, to_id, kind) VALUES (?1, ?2, 'relates')",
        [a1, a2],
    )
    .unwrap();
    let err = conn
        .execute(
            "UPDATE task_links SET to_id = ?1 WHERE from_id = ?2 AND kind = 'relates'",
            [b, a1],
        )
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("links entre projetos não são permitidos"));
}
