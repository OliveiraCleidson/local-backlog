//! Espelho TOML de `projects.root_path` para resolver tenant a partir da CWD.
//!
//! Contrato: DB é a verdade; o registry é cache sincronizado por `init`,
//! `relink`, `archive`. Escrita atômica via `.tmp` + `rename`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::BacklogError;

pub const REGISTRY_FILE: &str = "registry.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default, rename = "projects")]
    pub entries: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: i64,
    pub name: String,
    pub root_path: PathBuf,
}

impl Registry {
    /// Carrega de `path`. Ausente → vazio. TOML inválido → erro.
    pub fn load(path: &Path) -> Result<Self, BacklogError> {
        match fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| BacklogError::RegistryCorrupt {
                path: path.to_path_buf(),
                reason: e.to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(BacklogError::Io {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    /// Escreve atomicamente: write em `.tmp` + `rename`.
    pub fn save_atomic(&self, path: &Path) -> Result<(), BacklogError> {
        let text = toml::to_string_pretty(self).map_err(|e| BacklogError::RegistryCorrupt {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

        let tmp = path.with_extension("toml.tmp");
        let mut file =
            fs::File::create(&tmp).map_err(|source| BacklogError::RegistryWriteFailed {
                path: tmp.clone(),
                source,
            })?;
        file.write_all(text.as_bytes())
            .map_err(|source| BacklogError::RegistryWriteFailed {
                path: tmp.clone(),
                source,
            })?;
        file.sync_all()
            .map_err(|source| BacklogError::RegistryWriteFailed {
                path: tmp.clone(),
                source,
            })?;
        drop(file);

        fs::rename(&tmp, path).map_err(|source| BacklogError::RegistryWriteFailed {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Insere ou atualiza entry por `id`.
    pub fn upsert(&mut self, entry: RegistryEntry) {
        if let Some(slot) = self.entries.iter_mut().find(|e| e.id == entry.id) {
            *slot = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn remove(&mut self, id: i64) {
        self.entries.retain(|e| e.id != id);
    }

    /// Ancestral de maior profundidade cujo `root_path` canonizado é prefixo
    /// de `cwd` (já canonizado pelo chamador).
    ///
    /// Retorna `None` se nenhum registro cobrir a CWD. Entradas com paths
    /// que não existem mais no filesystem são ignoradas (podem ser limpas
    /// por `backlog doctor --fix`).
    pub fn find_ancestor(&self, cwd: &Path) -> Option<&RegistryEntry> {
        self.entries
            .iter()
            .filter_map(|e| {
                let canon = fs::canonicalize(&e.root_path).ok()?;
                if cwd.starts_with(&canon) {
                    Some((e, canon.components().count()))
                } else {
                    None
                }
            })
            .max_by_key(|(_, depth)| *depth)
            .map(|(e, _)| e)
    }
}
