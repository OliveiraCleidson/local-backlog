//! Toda função de dados (tasks/tags/attrs/links/events) recebe `project_id`
//! como primeiro parâmetro explícito — não existe variante "global"
//! (ADR-0001). A única exceção é `project_repo`, que administra o registry
//! cross-tenant.

pub mod attr_repo;
pub mod link_repo;
pub mod project_repo;
pub mod tag_repo;
pub mod task_repo;

/// `event_repo` é o módulo `db::events` — mesmo contrato dos demais repos.
pub use crate::db::events as event_repo;
