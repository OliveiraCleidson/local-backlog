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

    // Default oculta `done`/`cancelled` — precisamos de --include-archived
    // para que a seção `## done` apareça na ordem de config.
    let md = run(
        base.path(),
        &cwd,
        &["export", "--format", "markdown", "--include-archived"],
    );
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
fn markdown_hides_done_by_default() {
    let (base, cwd) = setup();
    let a = add(base.path(), &cwd, &["feita"]);
    let _b = add(base.path(), &cwd, &["aberta"]);
    bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["done", &a.to_string()])
        .assert()
        .success();

    let md_default = run(base.path(), &cwd, &["export", "--format", "markdown"]);
    assert!(!md_default.contains("## done"), "done vazou: {md_default}");
    assert!(!md_default.contains("feita"));
    assert!(md_default.contains("aberta"));

    // --status=done traz de volta mesmo sem --include-archived.
    let md_explicit = run(
        base.path(),
        &cwd,
        &["export", "--format", "markdown", "--status", "done"],
    );
    assert!(md_explicit.contains("## done"));
    assert!(md_explicit.contains("feita"));
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
    assert_eq!(v["project"]["name"], "proj");
    let tasks = v["tasks"].as_array().unwrap();
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
fn until_date_only_covers_whole_day() {
    let (base, cwd) = setup();
    // Task criada hoje aparece quando `--until` recebe a data de hoje (sem
    // hora), validando que a expansão para 23:59:59 cobre o dia inteiro.
    let _ = add(base.path(), &cwd, &["hoje"]);
    let today: String = chrono_now_date();

    let md = run(
        base.path(),
        &cwd,
        &["export", "--format", "markdown", "--until", &today],
    );
    assert!(
        md.contains("hoje"),
        "esperado task dentro do --until={today}, got: {md}"
    );
}

// Data de hoje sem dependência de crate de datas — usa `SystemTime`.
fn chrono_now_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    // Converte epoch → YYYY-MM-DD UTC (algoritmo howard hinnant).
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
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
    assert!(v["tasks"][0]["body"].is_null());

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
    assert_eq!(v2["tasks"][0]["body"], "corpo");
}
