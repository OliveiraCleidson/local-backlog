use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("backlog").unwrap()
}

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
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

fn add(base: &std::path::Path, cwd: &std::path::Path, args: &[&str]) -> i64 {
    let mut v = vec!["add"];
    v.extend_from_slice(args);
    let out = bin()
        .current_dir(cwd)
        .env("LOCAL_BACKLOG_HOME", base)
        .args(v)
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

fn edit_ok(base: &std::path::Path, cwd: &std::path::Path, args: &[&str]) {
    let mut v = vec!["edit"];
    v.extend_from_slice(args);
    bin()
        .current_dir(cwd)
        .env("LOCAL_BACKLOG_HOME", base)
        .args(v)
        .assert()
        .success();
}

#[test]
fn edit_updates_title_and_emits_field_changed() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, &["velho"]);

    edit_ok(base.path(), &cwd, &[&id.to_string(), "--title", "novo"]);

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let title: String = conn
        .query_row("SELECT title FROM tasks WHERE id = ?1", [id], |r| r.get(0))
        .unwrap();
    assert_eq!(title, "novo");

    let (kind, payload): (String, String) = conn
        .query_row(
            "SELECT kind, payload FROM task_events WHERE task_id = ?1 AND kind = 'field_changed' ORDER BY id DESC LIMIT 1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(kind, "field_changed");
    let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(v["field"], "title");
    assert_eq!(v["from"], "velho");
    assert_eq!(v["to"], "novo");
}

#[test]
fn edit_empty_title_is_rejected() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, &["x"]);

    let assert = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["edit", &id.to_string(), "--title", ""])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("title"));
}

#[test]
fn edit_body_empty_zeroes_field() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, &["x", "--body", "algum texto"]);

    edit_ok(base.path(), &cwd, &[&id.to_string(), "--body", ""]);

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let body: Option<String> = conn
        .query_row("SELECT body FROM tasks WHERE id = ?1", [id], |r| r.get(0))
        .unwrap();
    assert!(body.is_none());
}

#[test]
fn edit_parent_none_clears_parent() {
    let (base, cwd) = setup();
    let parent = add(base.path(), &cwd, &["p"]);
    let child = add(base.path(), &cwd, &["c", "--parent", &parent.to_string()]);

    edit_ok(base.path(), &cwd, &[&child.to_string(), "--parent", "none"]);

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let p: Option<i64> = conn
        .query_row("SELECT parent_id FROM tasks WHERE id = ?1", [child], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(p.is_none());
}

#[test]
fn edit_priority_emits_field_changed() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, &["x", "--priority", "50"]);

    edit_ok(base.path(), &cwd, &[&id.to_string(), "--priority", "10"]);

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let payload: String = conn
        .query_row(
            "SELECT payload FROM task_events WHERE task_id = ?1 AND kind = 'field_changed' ORDER BY id DESC LIMIT 1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(v["field"], "priority");
    assert_eq!(v["from"], 50);
    assert_eq!(v["to"], 10);
}

#[test]
fn edit_rejects_invalid_status() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, &["x"]);
    let assert = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["edit", &id.to_string(), "--status", "bogus"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("status"));
}
