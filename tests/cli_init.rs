use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("backlog").unwrap()
}

#[test]
fn init_yes_registers_project_and_prints_id() {
    let base = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let cwd_canon = std::fs::canonicalize(cwd.path()).unwrap();

    let out = bin()
        .current_dir(&cwd_canon)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["init", "--name", "proj-a", "--yes"])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let id: i64 = stdout.trim().parse().expect("stdout deve ser ID numérico");
    assert!(id > 0);

    let registry_text = std::fs::read_to_string(base.path().join("registry.toml")).unwrap();
    assert!(registry_text.contains("proj-a"));
    assert!(registry_text.contains(cwd_canon.to_str().unwrap()));
}

#[test]
fn init_is_idempotent() {
    let base = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let cwd_canon = std::fs::canonicalize(cwd.path()).unwrap();

    let first = bin()
        .current_dir(&cwd_canon)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["init", "--name", "proj-a", "--yes"])
        .assert()
        .success();
    let id1: i64 = String::from_utf8(first.get_output().stdout.clone())
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    let second = bin()
        .current_dir(&cwd_canon)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["init", "--name", "outro-nome", "--yes"])
        .assert()
        .success();
    let id2: i64 = String::from_utf8(second.get_output().stdout.clone())
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    assert_eq!(id1, id2, "init em CWD já registrada deve retornar mesmo ID");

    let stderr = String::from_utf8(second.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("já registrado"));
}

#[test]
fn init_default_name_is_cwd_basename() {
    let base = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let cwd = tmp.path().join("meu-projeto");
    std::fs::create_dir_all(&cwd).unwrap();
    let cwd_canon = std::fs::canonicalize(&cwd).unwrap();

    bin()
        .current_dir(&cwd_canon)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["init", "--yes"])
        .assert()
        .success();

    let registry_text = std::fs::read_to_string(base.path().join("registry.toml")).unwrap();
    assert!(registry_text.contains("meu-projeto"));
}

#[test]
fn no_subcommand_prints_help_and_exits_zero() {
    bin().assert().success();
}
