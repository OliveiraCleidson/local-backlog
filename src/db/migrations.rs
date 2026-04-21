//! Migrations imutáveis após release; correção é migration nova (ADR-0003).

use rusqlite_migration::{Migrations, M};

const M0001: &str = include_str!("../../migrations/0001_initial.sql");

// `Migrations::new` não é `const fn`, por isso expomos via função.
pub fn runner() -> Migrations<'static> {
    Migrations::new(vec![M::up(M0001)])
}
