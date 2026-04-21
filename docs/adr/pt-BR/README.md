# Architecture Decision Records (Registros de Decisão Arquitetural)

Registros de decisões arquiteturais do `local-backlog`. Formato curto, focado em **contexto**, **decisão** e **consequências**.

## Índice

- [ADR-0000 — Rust como linguagem de implementação com foco em aprendizado estratégico](0000-rust-como-linguagem-de-aprendizado.md)
- [ADR-0001 — Tenancy estrita por projeto](0001-tenancy-estrita-por-projeto.md)
- [ADR-0002 — Tabela `tasks` atômica com satélites extensíveis](0002-tasks-atomica-com-satelites.md)
- [ADR-0003 — Migrações Inline via `rusqlite_migration`](0003-migrations-inline.md)
- [ADR-0004 — Contrato de stdout/stderr e `--format` Universal](0004-output-contract.md)
- [ADR-0005 — Identificação de Projeto via Registro Global](0005-registry-global.md)

Use [`TEMPLATE.md`](TEMPLATE.md) como base para novos ADRs.

## Convenções

- Nome do arquivo: `NNNN-slug-kebab-case.md`.
- Status: `Proposto` → `Aceito` → `Substituído por ADR-NNNN` / `Depreciado`.
- Uma decisão por ADR. Decisões relacionadas referenciam umas às outras, não se fundem.
- Um ADR é imutável após ser `Aceito`. Alterações geram um novo ADR que o substitui.
