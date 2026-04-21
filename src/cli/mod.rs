use clap::Subcommand;
use miette::Result;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Registra o projeto atual (stub — Fase 2).
    Init,
    /// Adiciona uma task no tenant atual (stub — Fase 2).
    Add,
    /// Lista tasks do tenant atual (stub — Fase 2).
    List,
    /// Mostra uma task (stub — Fase 2).
    Show,
    /// Marca uma task como concluída (stub — Fase 2).
    Done,
    /// Arquiva uma task (stub — Fase 2).
    Archive,
}

pub fn dispatch(_cmd: Command) -> Result<()> {
    use crate::output::stderr_msg;
    stderr_msg("comando ainda não implementado (Fase 1: scaffold)");
    Ok(())
}
