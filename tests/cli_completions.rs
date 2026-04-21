use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("backlog").unwrap()
}

fn run_for(shell: &str) -> Vec<u8> {
    let base = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = bin()
        .current_dir(cwd.path())
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["completions", shell])
        .assert()
        .success();
    out.get_output().stdout.clone()
}

#[test]
fn completions_bash_emits_script_to_stdout() {
    let stdout = run_for("bash");
    let text = String::from_utf8(stdout).unwrap();
    assert!(text.contains("_backlog"));
    assert!(text.contains("COMPREPLY"));
}

#[test]
fn completions_zsh_emits_script_to_stdout() {
    let stdout = run_for("zsh");
    let text = String::from_utf8(stdout).unwrap();
    assert!(text.contains("#compdef backlog"));
}

#[test]
fn completions_fish_emits_script_to_stdout() {
    let stdout = run_for("fish");
    let text = String::from_utf8(stdout).unwrap();
    assert!(text.contains("complete -c backlog"));
}

#[test]
fn completions_powershell_emits_script_to_stdout() {
    let stdout = run_for("powershell");
    let text = String::from_utf8(stdout).unwrap();
    assert!(text.contains("Register-ArgumentCompleter"));
}

#[test]
fn completions_elvish_emits_script_to_stdout() {
    let stdout = run_for("elvish");
    let text = String::from_utf8(stdout).unwrap();
    assert!(!text.is_empty());
}

#[test]
fn completions_runs_without_bootstrap() {
    // Regressão P2: `completions` não pode depender de `~/.local-backlog/`.
    // Apontamos `LOCAL_BACKLOG_HOME` para um **arquivo regular**, que faria
    // o `App::bootstrap` falhar em `create_dir_all`. O comando deve emitir
    // o script mesmo assim — é stateless por construção.
    let home_file = tempfile::NamedTempFile::new().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = bin()
        .current_dir(cwd.path())
        .env("LOCAL_BACKLOG_HOME", home_file.path())
        .args(["completions", "bash"])
        .assert()
        .success();
    let text = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(text.contains("_backlog"));
}

#[test]
fn completions_rejects_invalid_shell() {
    let base = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    bin()
        .current_dir(cwd.path())
        .env("LOCAL_BACKLOG_HOME", base.path())
        .args(["completions", "tcsh"])
        .assert()
        .failure();
}
