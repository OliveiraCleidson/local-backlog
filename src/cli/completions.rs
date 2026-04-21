//! `backlog completions <shell>` — emite script de completion para o shell
//! solicitado em `stdout`. Comando **stateless**: não bootstrappa
//! `~/.local-backlog/`, não resolve tenant, não abre DB. Isso é proposital:
//! `completions` roda em provisioning de dotfiles, Docker base images e CI
//! onde o estado global ainda não existe (ou o home é read-only).

use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};

use crate::cli_root::Cli;
use crate::error::BacklogError;

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Shell-alvo.
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run(args: CompletionsArgs) -> Result<(), BacklogError> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}
