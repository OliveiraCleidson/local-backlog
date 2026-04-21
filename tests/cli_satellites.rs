//! Integração para `tag`, `link`, `attr`, `events`.

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

fn run(base: &std::path::Path, cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    bin()
        .current_dir(cwd)
        .env("LOCAL_BACKLOG_HOME", base)
        .args(args)
        .assert()
        .success()
        .get_output()
        .clone()
}

#[test]
fn tag_add_remove_list_full_cycle() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, "t");

    run(base.path(), &cwd, &["tag", "add", &id.to_string(), "a,b"]);
    let out = run(
        base.path(),
        &cwd,
        &["tag", "list", &id.to_string(), "--format", "json"],
    );
    let s = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let names: Vec<String> = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, vec!["a".to_string(), "b".to_string()]);

    run(base.path(), &cwd, &["tag", "remove", &id.to_string(), "a"]);
    let out = run(
        base.path(),
        &cwd,
        &["tag", "list", &id.to_string(), "--format", "json"],
    );
    let s = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["data"].as_array().unwrap().len(), 1);

    // eventos tag_added (2x) + tag_removed (1x)
    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let added: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_events WHERE task_id = ?1 AND kind = 'tag_added'",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    let removed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_events WHERE task_id = ?1 AND kind = 'tag_removed'",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(added, 2);
    assert_eq!(removed, 1);
}

#[test]
fn link_add_and_remove_with_whitelist() {
    let (base, cwd) = setup();
    let a = add(base.path(), &cwd, "a");
    let b = add(base.path(), &cwd, "b");

    run(
        base.path(),
        &cwd,
        &["link", &a.to_string(), &b.to_string(), "--kind", "blocks"],
    );

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let kind: String = conn
        .query_row(
            "SELECT kind FROM task_links WHERE from_id = ?1 AND to_id = ?2",
            [a, b],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(kind, "blocks");

    // kind fora da whitelist rejeita
    let assert = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["link", &a.to_string(), &b.to_string(), "--kind", "bogus"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("kind"));

    // remove (nova sintaxe: `link FROM --remove TO --kind K`)
    run(
        base.path(),
        &cwd,
        &[
            "link",
            &a.to_string(),
            "--remove",
            &b.to_string(),
            "--kind",
            "blocks",
        ],
    );
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_links WHERE from_id = ?1 AND to_id = ?2",
            [a, b],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn attr_set_unset_list_and_key_validation() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, "t");

    run(
        base.path(),
        &cwd,
        &["attr", "set", &id.to_string(), "jira", "ABC-1"],
    );
    run(
        base.path(),
        &cwd,
        &["attr", "set", &id.to_string(), "estimate.h", "4"],
    );

    let out = run(
        base.path(),
        &cwd,
        &["attr", "list", &id.to_string(), "--format", "json"],
    );
    let s = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["data"].as_array().unwrap().len(), 2);

    // chave inválida
    let assert = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["attr", "set", &id.to_string(), "Uppercase", "x"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("inválida") || stderr.contains("invalid"));

    run(
        base.path(),
        &cwd,
        &["attr", "unset", &id.to_string(), "jira"],
    );
    let out = run(
        base.path(),
        &cwd,
        &["attr", "list", &id.to_string(), "--format", "json"],
    );
    let s = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["data"].as_array().unwrap().len(), 1);
}

#[test]
fn events_lists_timeline_and_filters_by_kind() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, "t");

    run(base.path(), &cwd, &["tag", "add", &id.to_string(), "x"]);
    run(base.path(), &cwd, &["done", &id.to_string()]);

    let out = run(
        base.path(),
        &cwd,
        &["events", &id.to_string(), "--format", "json"],
    );
    let s = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    // created + tag_added + status_changed
    assert!(v["data"].as_array().unwrap().len() >= 3);

    let out = run(
        base.path(),
        &cwd,
        &[
            "events",
            &id.to_string(),
            "--kind",
            "status_changed",
            "--format",
            "json",
        ],
    );
    let s = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let list = v["data"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["kind"], "status_changed");
}
