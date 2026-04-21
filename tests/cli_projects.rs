use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("backlog").unwrap()
}

fn init_at(base: &std::path::Path, name: &str) -> std::path::PathBuf {
    let cwd = tempfile::tempdir().unwrap().keep();
    let canon = std::fs::canonicalize(&cwd).unwrap();
    bin()
        .current_dir(&canon)
        .env("LOCAL_BACKLOG_HOME", base)
        .args(["init", "--name", name, "--yes"])
        .assert()
        .success();
    canon
}

#[test]
fn projects_list_shows_all_registered() {
    let base = tempfile::tempdir().unwrap();
    init_at(base.path(), "a");
    init_at(base.path(), "b");

    let out = bin()
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["projects", "list", "--format", "json"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let names: Vec<String> = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"a".to_string()));
    assert!(names.contains(&"b".to_string()));
}

#[test]
fn projects_show_by_id_and_name() {
    let base = tempfile::tempdir().unwrap();
    init_at(base.path(), "only");

    let by_name = bin()
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["projects", "show", "only", "--format", "json"])
        .assert()
        .success();
    let s = String::from_utf8(by_name.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let id = v["data"]["id"].as_i64().unwrap();

    let by_id = bin()
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["projects", "show", &id.to_string(), "--format", "json"])
        .assert()
        .success();
    let s = String::from_utf8(by_id.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["data"]["name"], "only");
}

#[test]
fn projects_show_unknown_fails() {
    let base = tempfile::tempdir().unwrap();
    init_at(base.path(), "a");
    let assert = bin()
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["projects", "show", "inexistente"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("projeto") || stderr.contains("inexistente"));
}

#[test]
fn projects_archive_makes_cwd_commands_fail_with_project_archived() {
    let base = tempfile::tempdir().unwrap();
    let cwd = init_at(base.path(), "a");

    bin()
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["projects", "archive", "a"])
        .assert()
        .success();

    let assert = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["list"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("arquivado") || stderr.contains("archived"),
        "esperava erro ProjectArchived, veio: {stderr}"
    );
}

#[test]
fn projects_archive_restore_reenables_cwd_commands() {
    let base = tempfile::tempdir().unwrap();
    let cwd = init_at(base.path(), "a");

    bin()
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["projects", "archive", "a"])
        .assert()
        .success();
    bin()
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["projects", "archive", "a", "--restore"])
        .assert()
        .success();

    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["list"])
        .assert()
        .success();
}

#[test]
fn projects_relink_updates_root_path() {
    let base = tempfile::tempdir().unwrap();
    init_at(base.path(), "a");

    let new_cwd = tempfile::tempdir().unwrap().keep();
    let canon = std::fs::canonicalize(&new_cwd).unwrap();

    bin()
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["projects", "relink", "a", canon.to_str().unwrap()])
        .assert()
        .success();

    // agora `backlog list` no novo cwd resolve o projeto `a`
    bin()
        .current_dir(&canon)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["list"])
        .assert()
        .success();
}
