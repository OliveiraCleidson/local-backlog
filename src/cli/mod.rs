use clap::Subcommand;
use miette::Result;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Registra o projeto atual.
    Init,
    /// Adiciona uma task no tenant atual.
    Add,
    /// Lista tasks do tenant atual.
    List,
    /// Mostra uma task.
    Show,
    /// Marca uma task como concluída.
    Done,
    /// Arquiva uma task.
    Archive,
}

pub fn dispatch(_cmd: Command) -> Result<()> {
    use crate::output::stderr_msg;
    stderr_msg("comando ainda não implementado");
    Ok(())
}
