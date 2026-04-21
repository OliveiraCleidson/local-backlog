use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use miette::Result;

use local_backlog::cli;

#[derive(Parser, Debug)]
#[command(
    name = "backlog",
    about = "Gerenciador de backlog local por projeto.",
    version
)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    command: Option<cli::Command>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let level = cli
        .verbose
        .log_level_filter()
        .to_string()
        .to_ascii_lowercase();
    init_tracing(&level);

    match cli.command {
        Some(cmd) => cli::dispatch(cmd),
        None => Ok(()),
    }
}

fn init_tracing(default: &str) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_env("BACKLOG_LOG").unwrap_or_else(|_| EnvFilter::new(default));

    let _ = fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
