//! Bootstrap idempotente de `~/.local-backlog/`.
//!
//! Primeira execução cria o diretório base, `config.toml` default,
//! `registry.toml` vazio e `backlog.db` com migrations aplicadas.
//! Execuções subsequentes são no-op (arquivos existentes são preservados).

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::config::{ensure_base_dir, find_per_repo_config, Config};
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
    /// Razão do parse do `registry.toml` ter falhado, quando aplicável.
    /// Bootstrap continua com registro vazio; `doctor` reporta o erro.
    pub registry_corrupt: Option<String>,
}

impl App {
    /// Bootstrap usando `~/.local-backlog/` como base. Procura
    /// `.local-backlog.toml` subindo da `cwd` para alimentar a camada per-repo
    /// da cascata de config.
    pub fn bootstrap(cwd: &Path) -> Result<Self, BacklogError> {
        let base = ensure_base_dir()?;
        let per_repo = find_per_repo_config(cwd);
        Self::bootstrap_in_with(&base, per_repo.as_deref())
    }

    /// Bootstrap em `base_dir` arbitrário (usado em testes). Não aplica
    /// camada per-repo.
    pub fn bootstrap_in(base_dir: &Path) -> Result<Self, BacklogError> {
        Self::bootstrap_in_with(base_dir, None)
    }

    /// Bootstrap em `base_dir` com camada per-repo opcional.
    pub fn bootstrap_in_with(
        base_dir: &Path,
        per_repo: Option<&Path>,
    ) -> Result<Self, BacklogError> {
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

        let config = Config::load(Some(&config_path), per_repo)?;
        let (registry, registry_corrupt) = Registry::load_tolerant(&registry_path)?;
        if let Some(reason) = &registry_corrupt {
            tracing::warn!(
                path = %registry_path.display(),
                reason = %reason,
                "registry.toml inválido; seguindo com registro vazio (rode `backlog doctor`)"
            );
        }
        let conn = db::open(&db_path)?;

        Ok(Self {
            base_dir: base_dir.to_path_buf(),
            config,
            conn,
            registry,
            registry_path,
            config_path,
            db_path,
            registry_corrupt,
        })
    }

    /// Reescreve `registry.toml` a partir do estado em memória.
    pub fn save_registry(&self) -> Result<(), BacklogError> {
        self.registry.save_atomic(&self.registry_path)
    }
}
