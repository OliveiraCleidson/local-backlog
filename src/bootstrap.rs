//! Bootstrap idempotente de `~/.local-backlog/`.
//!
//! Primeira execução cria o diretório base, `config.toml` default,
//! `registry.toml` vazio e `backlog.db` com migrations aplicadas.
//! Execuções subsequentes são no-op (arquivos existentes são preservados).

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::config::{ensure_base_dir, Config};
use crate::db;
use crate::error::BacklogError;
use crate::registry::{Registry, REGISTRY_FILE};

pub const CONFIG_FILE: &str = "config.toml";
pub const DB_FILE: &str = "backlog.db";

/// Contexto carregado da `~/.local-backlog/` (ou equivalente em teste).
pub struct App {
    pub base_dir: PathBuf,
    pub config: Config,
    pub conn: Connection,
    pub registry: Registry,
    pub registry_path: PathBuf,
    pub config_path: PathBuf,
    pub db_path: PathBuf,
}

impl App {
    /// Bootstrap usando `~/.local-backlog/` como base.
    pub fn bootstrap() -> Result<Self, BacklogError> {
        let base = ensure_base_dir()?;
        Self::bootstrap_in(&base)
    }

    /// Bootstrap em `base_dir` arbitrário (usado em testes).
    pub fn bootstrap_in(base_dir: &Path) -> Result<Self, BacklogError> {
        std::fs::create_dir_all(base_dir).map_err(|source| BacklogError::Io {
            path: base_dir.to_path_buf(),
            source,
        })?;

        let config_path = base_dir.join(CONFIG_FILE);
        let registry_path = base_dir.join(REGISTRY_FILE);
        let db_path = base_dir.join(DB_FILE);

        if !config_path.exists() {
            let default = Config::default();
            let text =
                toml::to_string_pretty(&default).map_err(|e| BacklogError::RegistryCorrupt {
                    path: config_path.clone(),
                    reason: e.to_string(),
                })?;
            std::fs::write(&config_path, text).map_err(|source| BacklogError::Io {
                path: config_path.clone(),
                source,
            })?;
            tracing::info!(path = %config_path.display(), "config.toml default criado");
        }

        if !registry_path.exists() {
            Registry::default().save_atomic(&registry_path)?;
            tracing::info!(path = %registry_path.display(), "registry.toml vazio criado");
        }

        let config = Config::load(Some(&config_path), None)?;
        let registry = Registry::load(&registry_path)?;
        let conn = db::open(&db_path)?;

        Ok(Self {
            base_dir: base_dir.to_path_buf(),
            config,
            conn,
            registry,
            registry_path,
            config_path,
            db_path,
        })
    }

    /// Reescreve `registry.toml` a partir do estado em memória.
    pub fn save_registry(&self) -> Result<(), BacklogError> {
        self.registry.save_atomic(&self.registry_path)
    }
}
