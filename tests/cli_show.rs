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

fn add_task(base: &std::path::Path, cwd: &std::path::Path, args: &[&str]) -> i64 {
    let out = backlog(
        base,
        cwd,
        &[&["add"], args].concat().into_iter().collect::<Vec<_>>(),
    )
    .assert()
    .success();
    let s = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    s.trim().parse().unwrap()
}

#[test]
fn show_table_renders_fields_and_tags_and_event() {
    let (base, cwd) = setup_tenant();
    let id = add_task(
        base.path(),
        &cwd,
        &["minha task", "--priority", "42", "--tag", "infra,urgent"],
    );

    let out = backlog(base.path(), &cwd, &["show", &id.to_string()])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains(&format!("id:       {id}")));
    assert!(stdout.contains("minha task"));
    assert!(stdout.contains("priority: 42"));
    assert!(stdout.contains("#infra"));
    assert!(stdout.contains("#urgent"));
    assert!(stdout.contains("created"));
}

#[test]
fn show_json_envelope_has_expected_shape() {
    let (base, cwd) = setup_tenant();
    let id = add_task(
        base.path(),
        &cwd,
        &["outra", "--priority", "7", "--tag", "x"],
    );

    let out = backlog(
        base.path(),
        &cwd,
        &["show", &id.to_string(), "--format", "json"],
    )
    .assert()
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["data"]["task"]["id"], id);
    assert_eq!(v["data"]["task"]["priority"], 7);
    assert_eq!(v["data"]["tags"][0]["name"], "x");
    assert!(v["data"]["attributes"].as_array().unwrap().is_empty());
    assert!(v["data"]["links"].as_array().unwrap().is_empty());
    // evento `created` foi emitido por `add`
    let events = v["data"]["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["kind"], "created");
}

#[test]
fn show_unknown_id_is_not_found() {
    let (base, cwd) = setup_tenant();
    let assert = backlog(base.path(), &cwd, &["show", "9999"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("task 9999 não encontrada"));
}

#[test]
fn show_cross_tenant_is_indistinguishable_from_not_found() {
    let base = tempfile::tempdir().unwrap();
    let cwd_a = tempfile::tempdir().unwrap().keep();
    let cwd_b = tempfile::tempdir().unwrap().keep();
    let a = std::fs::canonicalize(&cwd_a).unwrap();
    let b = std::fs::canonicalize(&cwd_b).unwrap();

    bin()
        .current_dir(&a)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["init", "--name", "a", "--yes"])
        .assert()
        .success();
    bin()
        .current_dir(&b)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["init", "--name", "b", "--yes"])
        .assert()
        .success();

    let out = bin()
        .current_dir(&a)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["add", "secreta"])
        .assert()
        .success();
    let id_a: i64 = String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    let assert = bin()
        .current_dir(&b)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["show", &id_a.to_string()])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains(&format!("task {id_a} não encontrada")));
    assert!(
        !stderr.contains("project") && !stderr.contains("tenant"),
        "não deve vazar info de tenant"
    );
}
