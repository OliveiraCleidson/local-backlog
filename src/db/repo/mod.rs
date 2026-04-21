//! Toda função de dados (tasks/tags/attrs/links/events) recebe `project_id`
//! como primeiro parâmetro explícito — não existe variante "global"
//! (ADR-0001). A única exceção é `project_repo`, que administra o registry
//! cross-tenant.

pub mod project_repo;
pub mod tag_repo;
pub mod task_repo;
