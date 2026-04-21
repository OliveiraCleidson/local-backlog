# Architecture Decision Records (Registros de Decisión Arquitectónica)

Registros de decisiones arquitectónicas de `local-backlog`. Formato corto, centrado en el **contexto**, la **decisión** y las **consecuencias**.

## Índice

- [ADR-0000 — Rust como lenguaje de implementación con foco estratégico de aprendizaje](0000-rust-como-linguagem-de-aprendizado.md)
- [ADR-0001 — Tenencia estricta por proyecto](0001-tenancy-estrita-por-projeto.md)
- [ADR-0002 — Tabla `tasks` atómica con satélites extensibles](0002-tasks-atomica-com-satelites.md)
- [ADR-0003 — Migraciones Inline mediante `rusqlite_migration`](0003-migrations-inline.md)
- [ADR-0004 — Contrato de stdout/stderr y `--format` Universal](0004-output-contract.md)
- [ADR-0005 — Identificación de Proyecto mediante Registro Global](0005-registry-global.md)

Usá [`TEMPLATE.md`](TEMPLATE.md) como punto de partida para nuevos ADRs.

## Convenciones

- Nombre del archivo: `NNNN-slug-kebab-case.md`.
- Estado: `Propuesto` → `Aceptado` → `Reemplazado por ADR-NNNN` / `Depreciado`.
- Una decisión por ADR. Las decisiones relacionadas se referencian entre sí, no se fusionan.
- Un ADR es inmutable después de ser `Aceptado`. Los cambios generan un nuevo ADR que lo reemplaza.
