//! Repositórios por tabela. Toda função de leitura/escrita de dados de
//! projeto recebe `project_id` como primeiro parâmetro explícito — não
//! existe variante "global" para dados de tasks (ver ADR-0001).
//!
//! Implementações reais entram na Fase 2.

pub mod project_repo {}
pub mod task_repo {}
pub mod tag_repo {}
