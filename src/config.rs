//! Carregamento de configuração em camadas (figment):
//! defaults embutidos → `~/.local-backlog/config.toml` → `<repo>/.local-backlog.toml`
//! → env (`BACKLOG_*`) → flags.

use std::path::{Path, PathBuf};

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::error::BacklogError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub status: StatusConfig,
    pub task_type: TaskTypeConfig,
    pub priority: PriorityConfig,
    pub id: IdConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusConfig {
    pub values: Vec<String>,
    pub default: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTypeConfig {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityConfig {
    pub min: i64,
    pub max: i64,
    pub default: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdConfig {
    /// `integer` (default) ou `prefixed`. No MVP só `integer` é usado em leitura.
    pub display: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            status: StatusConfig {
                values: ["todo", "doing", "blocked", "done", "cancelled"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                default: "todo".to_string(),
            },
            task_type: TaskTypeConfig {
                values: ["feature", "bug", "debt", "ops", "admin", "idea"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            priority: PriorityConfig {
                min: 0,
                max: 3,
                default: 0,
            },
            id: IdConfig {
                display: "integer".to_string(),
            },
        }
    }
}

impl Config {
    /// Carrega a configuração aplicando a cascata de providers.
    pub fn load(global: Option<&Path>, per_repo: Option<&Path>) -> Result<Self, BacklogError> {
        let mut fig = Figment::from(Serialized::defaults(Config::default()));
        if let Some(g) = global {
            fig = fig.merge(Toml::file(g));
        }
        if let Some(r) = per_repo {
            fig = fig.merge(Toml::file(r));
        }
        fig = fig.merge(Env::prefixed("BACKLOG_").split("__"));
        fig.extract().map_err(BacklogError::from)
    }
}

/// Retorna o diretório base `~/.local-backlog/` garantindo existência.
pub fn ensure_base_dir() -> Result<PathBuf, BacklogError> {
    let home = dirs::home_dir().ok_or(BacklogError::HomeNotFound)?;
    let base = home.join(".local-backlog");
    std::fs::create_dir_all(&base).map_err(|source| BacklogError::Io {
        path: base.clone(),
        source,
    })?;
    Ok(base)
}
