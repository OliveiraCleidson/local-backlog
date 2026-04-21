use std::fmt::Display;
use std::io::{self, Write};

/// Use somente para dados consumíveis (pipe-friendly).
pub fn stdout_data(value: impl Display) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    // Descarta EPIPE: `backlog list | head` fechando stdout cedo é esperado.
    let _ = writeln!(handle, "{value}");
}

/// Use para logs, progresso e prompts. Nunca para dados.
pub fn stderr_msg(value: impl Display) {
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    let _ = writeln!(handle, "{value}");
}
