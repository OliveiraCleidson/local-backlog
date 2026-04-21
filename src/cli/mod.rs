//! Subcomandos do binário `backlog`.
//!
//! Cada módulo expõe um `XxxArgs` (derive clap) + `run(args, app, cwd)`.

use std::path::Path;

use clap::Subcommand;

use crate::bootstrap::App;
use crate::error::BacklogError;

pub mod init;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Registra o projeto atual no registry global.
    Init(init::InitArgs),
}

pub fn dispatch(cmd: Command, app: &mut App, cwd: &Path) -> Result<(), BacklogError> {
    match cmd {
        Command::Init(args) => init::run(args, app, cwd),
    }
}
