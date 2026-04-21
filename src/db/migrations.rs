//! Migrations embutidas via `include_str!` (ADR-0003).
//!
//! Cada migration é imutável após release; correção de bug em schema é
//! sempre uma nova migration aditiva.

use rusqlite_migration::{Migrations, M};

const M0001: &str = include_str!("../../migrations/0001_initial.sql");

/// Constrói o runner de migrations. Função (em vez de `const`) porque
/// `Migrations::new` não é `const fn`.
pub fn runner() -> Migrations<'static> {
    Migrations::new(vec![M::up(M0001)])
}
