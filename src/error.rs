use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum BacklogError {
    #[error("não foi possível localizar o diretório home do usuário")]
    #[diagnostic(code(backlog::home_not_found))]
    HomeNotFound,

    #[error("erro de I/O em {path}")]
    #[diagnostic(code(backlog::io))]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    #[diagnostic(code(backlog::db))]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    #[diagnostic(code(backlog::migration))]
    Migration(#[from] rusqlite_migration::Error),

    #[error(transparent)]
    #[diagnostic(code(backlog::config))]
    Config(Box<figment::Error>),
}

impl From<figment::Error> for BacklogError {
    fn from(value: figment::Error) -> Self {
        BacklogError::Config(Box::new(value))
    }
}
