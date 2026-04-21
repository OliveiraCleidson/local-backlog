//! Migrations imutáveis após release; correção é migration nova (ADR-0003).

use rusqlite_migration::{Migrations, M};

const M0001: &str = include_str!("../../migrations/0001_initial.sql");

/// Versão que o `PRAGMA user_version` deve ter após todas as migrations aplicadas.
/// Mantida em sincronia com o número de migrations em `runner()`.
pub const EXPECTED_USER_VERSION: i64 = 1;

// `Migrations::new` não é `const fn`, por isso expomos via função.
pub fn runner() -> Migrations<'static> {
    Migrations::new(vec![M::up(M0001)])
}
