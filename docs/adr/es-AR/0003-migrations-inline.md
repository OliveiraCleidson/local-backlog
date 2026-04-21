# ADR-0003 — Migraciones Inline mediante `rusqlite_migration`

- **Estado:** Aceptado
- **Fecha:** 2026-04-20

## Contexto

La CLI es un binario único distribuido mediante `cargo install`. Las migraciones de esquema deben:

1. Ejecutarse automáticamente en el primer uso y durante las actualizaciones.
2. No depender de una CLI externa (`diesel`, `sqlx`).
3. No requerir archivos SQL sueltos en el sistema de archivos del usuario.
4. Ser testeables contra una base de datos en memoria.

Opciones:

- `rusqlite_migration` inline (`const MIGRATIONS: &[M]`).
- `rusqlite_migration` por directorio (función `from-directory`).
- `refinery` con archivos SQL embebidos mediante `include_str!`.
- Script manual ejecutando PRAGMAs.

## Decisión

Usar `rusqlite_migration` en modo inline:

```rust
use rusqlite_migration::{Migrations, M};

const MIGRATIONS: &[M] = &[
    M::up(include_str!("../../migrations/0001_initial.sql")),
    // ...
];
```

Los archivos `.sql` viven en `migrations/` en el repositorio como una **referencia humana** (revisión, diff, snapshot) pero están embebidos en el binario mediante `include_str!`. La fuente de verdad en tiempo de ejecución es el slice constante.

El estado del esquema se controla mediante el `PRAGMA user_version` de SQLite — sin tabla auxiliar.

Las migraciones se ejecutan automáticamente en cada `backlog <cualquier comando>` mediante `Migrations::from_slice(...).to_latest(&mut conn)` durante el arranque de la conexión.

Un snapshot `insta` del resultado de `SELECT type, name, sql FROM sqlite_master ORDER BY name` valida el esquema final tras aplicar todas las migraciones.

## Consecuencias

**Positivas:**
- Binario autocontenido — el usuario nunca ve archivos SQL.
- Cero CLI externa; las actualizaciones del binario aplican el nuevo esquema de forma transparente.
- La prueba de migración es trivial: `Connection::open_in_memory()` + `to_latest`.
- El snapshot `insta` convierte "cambié una migración" en un diff de esquema revisable.

**Negativas:**
- Las migraciones ya publicadas no pueden modificarse — se requiere una nueva migración de ajuste. Regla: **una migración es inmutable después del lanzamiento.** Los cambios de esquema deben ser siempre aditivos o compensatorios.
- `include_str!` hace que los archivos .sql se conviertan en `&'static str` — muy pequeño en tiempo de ejecución, con un costo insignificante.

## Alternativas Consideradas

- **`from-directory`** — requeriría el envío de archivos junto con el binario (rompe el "binario único"); descartada.
- **`refinery`** — similar en capacidad, pero `rusqlite_migration` es más ligero y utiliza el `user_version` nativo, evitando una tabla auxiliar.
- **Migraciones ad-hoc mediante código Rust** — rechazado: pierde el contrato declarativo SQL que es fácil de revisar en PRs.

## Relacionados

- [ADR-0002 — Satélites](0002-tasks-atomica-com-satelites.md) — la promoción EAV → columna genera una nueva migración inmutable.
