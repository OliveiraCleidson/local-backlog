//! Integração para `backlog doctor`.

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
        .args(["init", "--name", "d", "--yes"])
        .assert()
        .success();
    let _persist = cwd.keep();
    (base, canon)
}

#[test]
fn clean_install_reports_zero_problems() {
    let (base, cwd) = setup();
    let out = bin()
        .current_dir(&cwd)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["doctor"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("ok — zero problemas"));
}

#[test]
fn missing_path_raises_warning_fixable() {
    let base = tempfile::tempdir().unwrap();
    // Cria projeto em subdiretório persistente dentro do base.
    let proj_dir = base.path().join("proj-ghost");
    std::fs::create_dir(&proj_dir).unwrap();
    let canon = std::fs::canonicalize(&proj_dir).unwrap();

    bin()
        .current_dir(&canon)
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["init", "--name", "ghost", "--yes"])
        .assert()
        .success();

    // Remove o diretório — path no registry vira fantasma.
    std::fs::remove_dir_all(&canon).unwrap();

    let assert = bin()
        .current_dir(base.path())
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["doctor"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("warn:"), "stderr inesperado: {stderr}");

    // --fix --yes remove a entry; exit ainda é 1 porque houve fixable detectado.
    bin()
        .current_dir(base.path())
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["doctor", "--fix", "--yes"])
        .assert()
        .failure();

    // Passada seguinte detecta "ativo mas ausente do registry" (esperado:
    // --fix limpa o registry, mas o projeto ainda está na tabela).
    let assert2 = bin()
        .current_dir(base.path())
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["doctor"])
        .assert()
        .failure();
    let stderr2 = String::from_utf8(assert2.get_output().stderr.clone()).unwrap();
    assert!(stderr2.contains("ausente do registry"));
}

#[test]
fn orphan_project_without_registry_is_error() {
    let (base, _cwd) = setup();
    // Remove registry para simular divergência DB-registry.
    std::fs::remove_file(base.path().join("registry.toml")).unwrap();
    // Recria registry vazio via re-abertura do binário (ele recria no bootstrap).
    let assert = bin()
        .current_dir(base.path())
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["doctor"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("error:"));
    assert!(stderr.contains("ativo mas ausente do registry"));
}
