use local_backlog::bootstrap::{App, CONFIG_FILE, DB_FILE};
use local_backlog::registry::REGISTRY_FILE;

#[test]
fn bootstrap_creates_all_artifacts_on_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let app = App::bootstrap_in(tmp.path()).unwrap();

    assert!(app.base_dir.join(CONFIG_FILE).exists());
    assert!(app.base_dir.join(REGISTRY_FILE).exists());
    assert!(app.base_dir.join(DB_FILE).exists());
    assert!(app.registry.entries.is_empty());
}

#[test]
fn bootstrap_is_idempotent() {
    let tmp = tempfile::tempdir().unwrap();

    let first = App::bootstrap_in(tmp.path()).unwrap();
    let config_mtime = std::fs::metadata(&first.config_path)
        .unwrap()
        .modified()
        .unwrap();
    drop(first);

    // segunda execução não deve sobrescrever config.toml existente
    let second = App::bootstrap_in(tmp.path()).unwrap();
    let after_mtime = std::fs::metadata(&second.config_path)
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(
        config_mtime, after_mtime,
        "config.toml não deve ser reescrito"
    );
}

#[test]
fn bootstrap_preserves_existing_config() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join(CONFIG_FILE);
    std::fs::write(
        &config_path,
        r#"
[status]
values = ["todo", "wip", "done"]
default = "todo"

[task_type]
values = ["feature"]

[priority]
default = 50
order = "desc"

[id]
display = "prefixed"

[link]
kinds = ["blocks"]
"#,
    )
    .unwrap();

    let app = App::bootstrap_in(tmp.path()).unwrap();
    assert_eq!(app.config.priority.default, 50);
    assert_eq!(
        app.config.priority.order,
        local_backlog::config::PriorityOrder::Desc
    );
    assert_eq!(app.config.id.display, "prefixed");
}
