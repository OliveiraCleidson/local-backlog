use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("backlog").unwrap()
}

fn setup_tenant() -> (tempfile::TempDir, std::path::PathBuf) {
    let base = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let canon = std::fs::canonicalize(cwd.path()).unwrap();
    bin()
        .current_dir(&canon)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["init", "--name", "p", "--yes"])
        .assert()
        .success();
    let _persist = cwd.keep();
    (base, canon)
}

fn add(base: &std::path::Path, cwd: &std::path::Path, title: &str) -> i64 {
    let out = bin()
        .current_dir(cwd)
        .env("LOCAL_BACKLOG_HOME", base)
        .args(["add", title])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

#[test]
fn done_sets_status_and_completed_at_and_emits_event() {
    let (base, cwd) = setup_tenant();
    let id = add(base.path(), &cwd, "t");

    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["done", &id.to_string()])
        .assert()
        .success();

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let (status, completed): (String, Option<String>) = conn
        .query_row(
            "SELECT status, completed_at FROM tasks WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "done");
    assert!(completed.is_some());

    let (kind, payload): (String, String) = conn
        .query_row(
            "SELECT kind, payload FROM task_events WHERE task_id = ?1 ORDER BY id DESC LIMIT 1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(kind, "status_changed");
    let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(v["from"], "todo");
    assert_eq!(v["to"], "done");
}

#[test]
fn done_is_idempotent_and_does_not_emit_duplicate_event() {
    let (base, cwd) = setup_tenant();
    let id = add(base.path(), &cwd, "t");

    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["done", &id.to_string()])
        .assert()
        .success();
    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["done", &id.to_string()])
        .assert()
        .success();

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_events WHERE task_id = ?1 AND kind = 'status_changed'",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn done_unknown_id_is_not_found() {
    let (base, cwd) = setup_tenant();
    let assert = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["done", "9999"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("task 9999 não encontrada"));
}
