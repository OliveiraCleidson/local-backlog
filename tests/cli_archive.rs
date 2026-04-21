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
fn archive_sets_archived_at_and_emits_event() {
    let (base, cwd) = setup_tenant();
    let id = add(base.path(), &cwd, "t");

    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["archive", &id.to_string()])
        .assert()
        .success();

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let archived: Option<String> = conn
        .query_row("SELECT archived_at FROM tasks WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(archived.is_some());

    let kind: String = conn
        .query_row(
            "SELECT kind FROM task_events WHERE task_id = ?1 ORDER BY id DESC LIMIT 1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(kind, "archived");
}

#[test]
fn archive_is_idempotent() {
    let (base, cwd) = setup_tenant();
    let id = add(base.path(), &cwd, "t");

    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["archive", &id.to_string()])
        .assert()
        .success();
    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["archive", &id.to_string()])
        .assert()
        .success();

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_events WHERE task_id = ?1 AND kind = 'archived'",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn archived_tasks_excluded_from_list_by_default() {
    let (base, cwd) = setup_tenant();
    let id = add(base.path(), &cwd, "gone");
    let _keep = add(base.path(), &cwd, "keep");

    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["archive", &id.to_string()])
        .assert()
        .success();

    let out = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["list", "--format", "json"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let titles: Vec<String> = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(titles, vec!["keep".to_string()]);

    let out = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["list", "--format", "json", "--include-archived"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["data"].as_array().unwrap().len(), 2);
}
