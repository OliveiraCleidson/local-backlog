//! Root CLI parser. Vive em `lib` para que subcomandos (ex.: `completions`)
//! possam reconstruir o `clap::Command` sem depender de itens em `main.rs`.

use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};

use crate::cli::Command;

#[derive(Parser, Debug)]
#[command(
    name = "backlog",
    about = "Gerenciador de backlog local por projeto.",
    version
)]
pub struct Cli {
    #[command(flatten)]
    pub verbose: Verbosity<WarnLevel>,

    #[command(subcommand)]
    pub command: Option<Command>,
}
