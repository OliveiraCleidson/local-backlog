use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("backlog").unwrap()
}

/// Cria um tenant `proj-a` em `cwd` e retorna `(base_dir, cwd_canon)`.
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

    // tempdir `cwd` é dropado aqui; isso deletaria o diretório, então
    // consolidamos criando um subdir persistente. Preservamos via leak.
    let _persist = cwd.keep();
    (base, canon)
}

fn add_task(
    base: &std::path::Path,
    cwd: &std::path::Path,
    args: &[&str],
) -> assert_cmd::assert::Assert {
    let mut cmd = bin();
    cmd.current_dir(cwd)
        .env("LOCAL_BACKLOG_HOME", base)
        .arg("add");
    for a in args {
        cmd.arg(a);
    }
    cmd.assert()
}

#[test]
fn add_prints_id_and_defaults_fields() {
    let (base, cwd) = setup_tenant();
    let out = add_task(base.path(), &cwd, &["primeira"]).success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let id: i64 = stdout.trim().parse().unwrap();
    assert!(id > 0);
}

#[test]
fn add_applies_priority_and_status_defaults() {
    let (base, cwd) = setup_tenant();
    add_task(base.path(), &cwd, &["a"]).success();

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let (status, priority): (String, i64) = conn
        .query_row(
            "SELECT status, priority FROM tasks WHERE title = 'a'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "todo");
    assert_eq!(priority, 100);
}

#[test]
fn add_rejects_invalid_status() {
    let (base, cwd) = setup_tenant();
    let assert = add_task(base.path(), &cwd, &["x", "--status", "not-a-status"]).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("status"));
    assert!(stderr.contains("not-a-status"));
}

#[test]
fn add_creates_tags_via_csv() {
    let (base, cwd) = setup_tenant();
    add_task(base.path(), &cwd, &["x", "--tag", "infra,urgent"]).success();

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let tags: Vec<String> = conn
        .prepare("SELECT name FROM tags ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(tags, vec!["infra".to_string(), "urgent".to_string()]);
}

#[test]
fn add_rejects_unknown_parent() {
    let (base, cwd) = setup_tenant();
    let assert = add_task(base.path(), &cwd, &["x", "--parent", "9999"]).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("task 9999 não encontrada"));
}

#[test]
fn add_parent_cross_tenant_indistinguishable_from_not_found() {
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

    // task 1 em `a`
    let out = bin()
        .current_dir(&a)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["add", "uma"])
        .assert()
        .success();
    let id_a: i64 = String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    // em `b`, usando o id da task de `a` como parent: deve falhar com
    // a mesma mensagem de "não existe".
    let assert = bin()
        .current_dir(&b)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["add", "outra", "--parent", &id_a.to_string()])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains(&format!("task {id_a} não encontrada")));
    assert!(
        !stderr.contains("project"),
        "não deve vazar 'project' ou tenant info"
    );
}

#[test]
fn add_emits_created_event() {
    let (base, cwd) = setup_tenant();
    add_task(base.path(), &cwd, &["evento", "--priority", "42"]).success();

    let conn = rusqlite::Connection::open(base.path().join("backlog.db")).unwrap();
    let (kind, payload): (String, String) = conn
        .query_row(
            "SELECT kind, payload FROM task_events ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(kind, "created");
    let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(parsed["priority"], 42);
    assert_eq!(parsed["title"], "evento");
}
