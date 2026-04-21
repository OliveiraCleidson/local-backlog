//! Renderers `table` e `json` para comandos de leitura (Fase 2+).
//!
//! JSON segue o envelope `{ "schema_version": N, "data": ... }` (ADR-0004).

use serde::Serialize;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Table,
    Json,
}

#[derive(Debug, Serialize)]
pub struct JsonEnvelope<T: Serialize> {
    pub schema_version: u32,
    pub data: T,
}

impl<T: Serialize> JsonEnvelope<T> {
    pub fn new(data: T) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            data,
        }
    }
}
