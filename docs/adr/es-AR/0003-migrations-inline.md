# ADR-0003 — Migraciones Inline mediante `rusqlite_migration`

- **Estado:** Aceptado
- **Fecha:** 2026-04-20

## Contexto

La CLI es un binario único que se distribuye con `cargo install`. Las migraciones del esquema tienen que:

1. Correr solas la primera vez que se use la herramienta y en cada actualización.
2. No depender de una CLI externa (como `diesel` o `sqlx`).
3. No andar pidiendo archivos SQL sueltos por el sistema del usuario.
4. Poder testearse fácil contra una base de datos en memoria.

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

Los archivos `.sql` viven en `migrations/` dentro del repositorio para que sea fácil revisarlos (revisión, diff, snapshot), pero terminan metidos en el binario con `include_str!`. En tiempo de ejecución, la fuente de verdad es el slice constante.

El estado del esquema se controla con el `PRAGMA user_version` de SQLite — sin tablas auxiliares.

Las migraciones corren solas con cada comando de `backlog` usando `Migrations::from_slice(...).to_latest(&mut conn)` apenas se abre la conexión.

Un snapshot de `insta` del resultado de `SELECT type, name, sql FROM sqlite_master ORDER BY name` nos sirve para validar el esquema final después de aplicar todas las migraciones.

## Consecuencias

**Positivas:**
- Binario autocontenido — el usuario nunca ve archivos SQL.
- Cero CLI externa; el binario se encarga de aplicar el esquema nuevo de forma transparente.
- Testear la migración es una pavada: `Connection::open_in_memory()` + `to_latest`.
- El snapshot de `insta` hace que un cambio en una migración sea un diff de esquema fácil de revisar.

**Negativas:**
- Una vez que publicaste una migración, no la podés tocar — si hace falta un ajuste, tenés que meter una migración nueva. Regla: **una migración es inmutable después del release.** Los cambios en el esquema tienen que ser siempre para sumar o para arreglar algo de antes.
- `include_str!` hace que los archivos .sql se conviertan en `&'static str` — esto es muy liviano en runtime, casi no gasta nada.

## Alternativas Consideradas

- **`from-directory`** — requeriría el envío de archivos junto con el binario (rompe el "binario único"); descartada.
- **`refinery`** — similar en capacidad, pero `rusqlite_migration` es más ligero y utiliza el `user_version` nativo, evitando una tabla auxiliar.
- **Migraciones ad-hoc mediante código Rust** — rechazado: pierde el contrato declarativo SQL que es fácil de revisar en PRs.

## Relacionados

- [ADR-0002 — Satélites](0002-tasks-atomica-com-satelites.md) — la promoción EAV → columna genera una nueva migración inmutable.
