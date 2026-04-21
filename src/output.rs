use std::fmt::Display;
use std::io::{self, Write};

/// Escreve dados em stdout (pipe-friendly). Use APENAS para saída que o
/// usuário (ou outro processo) consome como resultado do comando.
pub fn stdout_data(value: impl Display) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    // Ignoramos EPIPE (comum em `backlog list | head`).
    let _ = writeln!(handle, "{value}");
}

/// Escreve mensagens em stderr (logs, progresso, prompts). NUNCA para dados.
pub fn stderr_msg(value: impl Display) {
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    let _ = writeln!(handle, "{value}");
}
