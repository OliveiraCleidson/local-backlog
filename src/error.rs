use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum BacklogError {
    #[error("não foi possível localizar o diretório home do usuário")]
    #[diagnostic(
        code(backlog::io::home_not_found),
        help("defina `LOCAL_BACKLOG_HOME` apontando para um diretório gravável")
    )]
    HomeNotFound,

    #[error("erro de I/O em {path}")]
    #[diagnostic(
        code(backlog::io::fs),
        help("verifique permissões e existência do caminho")
    )]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    #[diagnostic(
        code(backlog::db::sqlite),
        help("se persistir, rode `backlog doctor` para verificar integridade")
    )]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    #[diagnostic(
        code(backlog::db::migration),
        help("migration inconsistente; rode `backlog doctor`")
    )]
    Migration(#[from] rusqlite_migration::Error),

    #[error(transparent)]
    #[diagnostic(
        code(backlog::config::parse),
        help("revise ~/.local-backlog/config.toml ou .backlog.toml do repo")
    )]
    Config(Box<figment::Error>),

    #[error("nenhum projeto registrado em {cwd} (ou ancestrais)")]
    #[diagnostic(
        code(backlog::tenant::not_registered),
        help("execute `backlog init` neste diretório para registrá-lo")
    )]
    ProjectNotRegistered { cwd: PathBuf },

    #[error("projeto '{name}' (id={id}) está arquivado")]
    #[diagnostic(
        code(backlog::tenant::archived),
        help("desarquive com `backlog projects archive {name} --restore`")
    )]
    ProjectArchived { id: i64, name: String },

    #[error("registry em {path} é inválido: {reason}")]
    #[diagnostic(
        code(backlog::tenant::registry_corrupt),
        help("corrija manualmente ou rode `backlog doctor --fix`")
    )]
    RegistryCorrupt { path: PathBuf, reason: String },

    #[error("falha ao escrever registry em {path}")]
    #[diagnostic(
        code(backlog::tenant::registry_write_failed),
        help("verifique permissões em ~/.local-backlog/ e rode `backlog doctor`")
    )]
    RegistryWriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("task {id} não encontrada")]
    #[diagnostic(
        code(backlog::input::task_not_found),
        help("confirme o id com `backlog list`")
    )]
    TaskNotFound { id: i64 },

    #[error("projeto '{name}' não encontrado")]
    #[diagnostic(
        code(backlog::input::project_not_found),
        help("veja projetos registrados com `backlog projects list`")
    )]
    ProjectNotFound { name: String },

    #[error("valor inválido para {field}: '{value}'. Aceitos: {allowed}")]
    #[diagnostic(code(backlog::input::invalid_enum))]
    InvalidEnum {
        field: &'static str,
        value: String,
        allowed: String,
    },

    #[error("entrada inválida: {0}")]
    #[diagnostic(code(backlog::input::invalid))]
    InvalidInput(String),
}

impl From<figment::Error> for BacklogError {
    fn from(value: figment::Error) -> Self {
        BacklogError::Config(Box::new(value))
    }
}
