use clap::Parser;
use miette::Result;

use local_backlog::bootstrap::App;
use local_backlog::cli;
use local_backlog::cli_root::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let level = cli
        .verbose
        .log_level_filter()
        .to_string()
        .to_ascii_lowercase();
    init_tracing(&level);

    let Some(cmd) = cli.command else {
        use clap::CommandFactory;
        let mut stderr = std::io::stderr();
        let _ = Cli::command().write_help(&mut stderr);
        use std::io::Write as _;
        let _ = writeln!(stderr);
        return Ok(());
    };

    // Subcomandos stateless rodam antes de `App::bootstrap`: não precisam de
    // DB, registry nem `~/.local-backlog/`. Isso permite rodar o comando em
    // provisioning de dotfiles, imagens Docker e CI onde o home global ainda
    // não existe ou é read-only.
    if let cli::Command::Completions(args) = cmd {
        cli::completions::run(args)?;
        return Ok(());
    }

    let cwd = std::env::current_dir().map_err(|source| local_backlog::error::BacklogError::Io {
        path: std::path::PathBuf::from("."),
        source,
    })?;
    let mut app = App::bootstrap(&cwd)?;
    cli::dispatch(cmd, &mut app, &cwd)?;
    Ok(())
}

fn init_tracing(default: &str) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_env("BACKLOG_LOG").unwrap_or_else(|_| EnvFilter::new(default));

    let _ = fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
