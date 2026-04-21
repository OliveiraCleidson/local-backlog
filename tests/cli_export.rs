//! Integração para `backlog export`.

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
        .args(["init", "--name", "proj", "--yes"])
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
        .args(&v)
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

fn run(base: &std::path::Path, cwd: &std::path::Path, args: &[&str]) -> String {
    let out = bin()
        .current_dir(cwd)
        .env("LOCAL_BACKLOG_HOME", base)
        .args(args)
        .assert()
        .success()
        .get_output()
        .clone();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn markdown_groups_by_status_in_config_order() {
    let (base, cwd) = setup();
    let a = add(base.path(), &cwd, &["A", "--priority", "10", "--tag", "x"]);
    let _b = add(base.path(), &cwd, &["B", "--priority", "20"]);
    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["done", &a.to_string()])
        .assert()
        .success();

    let md = run(base.path(), &cwd, &["export", "--format", "markdown"]);
    // project header
    assert!(md.starts_with("# proj\n"), "header faltando: {md}");
    // status 'todo' vem antes de 'done' (ordem config)
    let idx_todo = md.find("## todo").expect("seção todo");
    let idx_done = md.find("## done").expect("seção done");
    assert!(idx_todo < idx_done);
    // IDs com prefixo T-
    assert!(md.contains("T-"));
    assert!(md.contains("[10]"));
    assert!(md.contains("#x"));
}

#[test]
fn markdown_filters_by_status_and_tag() {
    let (base, cwd) = setup();
    let _a = add(base.path(), &cwd, &["foo", "--tag", "alpha"]);
    let _b = add(base.path(), &cwd, &["bar", "--tag", "beta"]);

    let md = run(
        base.path(),
        &cwd,
        &["export", "--format", "markdown", "--tag", "alpha"],
    );
    assert!(md.contains("foo"));
    assert!(!md.contains("bar"));
}

#[test]
fn markdown_empty_when_no_tasks() {
    let (base, cwd) = setup();
    let md = run(base.path(), &cwd, &["export", "--format", "markdown"]);
    assert!(md.contains("_sem tasks_"));
}

#[test]
fn json_export_has_envelope_and_project() {
    let (base, cwd) = setup();
    let _a = add(base.path(), &cwd, &["task-a", "--tag", "k"]);

    let json = run(base.path(), &cwd, &["export", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["data"]["project"]["name"], "proj");
    let tasks = v["data"]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "task-a");
    assert_eq!(tasks[0]["tags"][0], "k");
}

#[test]
fn json_export_is_byte_stable_across_runs() {
    let (base, cwd) = setup();
    let _a = add(base.path(), &cwd, &["a", "--priority", "5"]);
    let _b = add(base.path(), &cwd, &["b", "--priority", "1"]);
    let first = run(base.path(), &cwd, &["export", "--format", "json"]);
    let second = run(base.path(), &cwd, &["export", "--format", "json"]);
    assert_eq!(first, second);
}

#[test]
fn include_body_and_events_flags() {
    let (base, cwd) = setup();
    let id = add(base.path(), &cwd, &["t", "--body", "corpo"]);

    // sem --include-body, body é omitido do markdown e do json.
    let md_default = run(base.path(), &cwd, &["export", "--format", "markdown"]);
    assert!(!md_default.contains("> corpo"));
    let json_default = run(base.path(), &cwd, &["export", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&json_default).unwrap();
    assert!(v["data"]["tasks"][0]["body"].is_null());

    let md = run(
        base.path(),
        &cwd,
        &[
            "export",
            "--format",
            "markdown",
            "--include-body",
            "--include-events",
        ],
    );
    assert!(md.contains("> corpo"));
    assert!(md.contains(&format!("T-{id}")));
    assert!(md.contains("`created`"));

    let json = run(
        base.path(),
        &cwd,
        &["export", "--format", "json", "--include-body"],
    );
    let v2: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v2["data"]["tasks"][0]["body"], "corpo");
}
