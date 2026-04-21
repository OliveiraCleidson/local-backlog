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
        .args(["init", "--name", "proj-a", "--yes"])
        .assert()
        .success();

    let _persist = cwd.keep();
    (base, canon)
}

fn backlog<'a>(base: &'a std::path::Path, cwd: &'a std::path::Path, args: &[&str]) -> Command {
    let mut cmd = bin();
    cmd.current_dir(cwd).env("LOCAL_BACKLOG_HOME", base);
    for a in args {
        cmd.arg(a);
    }
    cmd
}

#[test]
fn list_empty_prints_placeholder() {
    let (base, cwd) = setup_tenant();
    let out = backlog(base.path(), &cwd, &["list"]).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("nenhuma task"));
}

#[test]
fn list_table_contains_task_fields() {
    let (base, cwd) = setup_tenant();
    backlog(
        base.path(),
        &cwd,
        &["add", "primeira", "--priority", "50", "--tag", "infra"],
    )
    .assert()
    .success();

    let out = backlog(base.path(), &cwd, &["list"]).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("ID"));
    assert!(stdout.contains("PRI"));
    assert!(stdout.contains("STATUS"));
    assert!(stdout.contains("primeira"));
    assert!(stdout.contains("#infra"));
}

#[test]
fn list_json_envelope_is_schema_versioned() {
    let (base, cwd) = setup_tenant();
    backlog(base.path(), &cwd, &["add", "a", "--priority", "10"])
        .assert()
        .success();
    backlog(base.path(), &cwd, &["add", "b", "--priority", "20"])
        .assert()
        .success();

    let out = backlog(base.path(), &cwd, &["list", "--format", "json"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["schema_version"], 1);
    let data = v["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    // ASC por default → priority 10 antes de 20
    assert_eq!(data[0]["priority"], 10);
    assert_eq!(data[1]["priority"], 20);
}

#[test]
fn list_order_desc_flips_priority_order() {
    let (base, cwd) = setup_tenant();
    backlog(base.path(), &cwd, &["add", "a", "--priority", "10"])
        .assert()
        .success();
    backlog(base.path(), &cwd, &["add", "b", "--priority", "20"])
        .assert()
        .success();

    let out = backlog(
        base.path(),
        &cwd,
        &["list", "--format", "json", "--order", "desc"],
    )
    .assert()
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let data = v["data"].as_array().unwrap();
    assert_eq!(data[0]["priority"], 20);
    assert_eq!(data[1]["priority"], 10);
}

#[test]
fn list_filters_by_status_and_tag() {
    let (base, cwd) = setup_tenant();
    backlog(base.path(), &cwd, &["add", "a", "--tag", "infra"])
        .assert()
        .success();
    backlog(base.path(), &cwd, &["add", "b", "--tag", "urgent"])
        .assert()
        .success();
    backlog(
        base.path(),
        &cwd,
        &["add", "c", "--status", "doing", "--tag", "infra"],
    )
    .assert()
    .success();

    let out = backlog(
        base.path(),
        &cwd,
        &["list", "--format", "json", "--tag", "infra"],
    )
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
    assert_eq!(titles.len(), 2);
    assert!(titles.contains(&"a".to_string()));
    assert!(titles.contains(&"c".to_string()));

    let out = backlog(
        base.path(),
        &cwd,
        &["list", "--format", "json", "--status", "doing"],
    )
    .assert()
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let data = v["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["title"], "c");
}

#[test]
fn list_rejects_invalid_status_and_format() {
    let (base, cwd) = setup_tenant();
    let assert = backlog(base.path(), &cwd, &["list", "--status", "bogus"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("status"));

    let assert = backlog(base.path(), &cwd, &["list", "--format", "xml"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("format"));
}
