//! Cascata: defaults → global → per-repo → env `BACKLOG_*` → flags.

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
    pub link: LinkConfig,
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
    pub default: i64,
    pub order: PriorityOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriorityOrder {
    Asc,
    Desc,
}

impl PriorityOrder {
    pub fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdConfig {
    /// Valores aceitos: `integer`, `prefixed`.
    pub display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkConfig {
    pub kinds: Vec<String>,
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
                default: 100,
                order: PriorityOrder::Asc,
            },
            id: IdConfig {
                display: "integer".to_string(),
            },
            link: LinkConfig {
                kinds: ["blocks", "relates", "duplicates", "spawned-from-plan"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
        }
    }
}

impl Config {
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

pub const PER_REPO_CONFIG_FILE: &str = ".local-backlog.toml";

/// Sobe a árvore a partir de `cwd` procurando o primeiro `.local-backlog.toml`.
/// Retorna `None` se nenhum for encontrado até a raiz do filesystem.
pub fn find_per_repo_config(cwd: &Path) -> Option<PathBuf> {
    let mut cur: Option<&Path> = Some(cwd);
    while let Some(dir) = cur {
        let candidate = dir.join(PER_REPO_CONFIG_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }
        cur = dir.parent();
    }
    None
}

/// Resolve `~/.local-backlog/` ou a pasta indicada por
/// `LOCAL_BACKLOG_HOME` (usado em testes e para rodar em sandbox).
pub fn ensure_base_dir() -> Result<PathBuf, BacklogError> {
    let base = match std::env::var_os("LOCAL_BACKLOG_HOME") {
        Some(p) => PathBuf::from(p),
        None => {
            let home = dirs::home_dir().ok_or(BacklogError::HomeNotFound)?;
            home.join(".local-backlog")
        }
    };
    std::fs::create_dir_all(&base).map_err(|source| BacklogError::Io {
        path: base.clone(),
        source,
    })?;
    Ok(base)
}
