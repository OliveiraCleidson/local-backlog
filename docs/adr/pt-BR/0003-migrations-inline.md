# ADR-0003 — Migrações Inline via `rusqlite_migration`

- **Status:** Aceito
- **Data:** 2026-04-20

## Contexto

O CLI é um binário único distribuído via `cargo install`. As migrações de esquema devem:

1. Rodar automaticamente no primeiro uso e durante atualizações.
2. Não depender de uma CLI externa (`diesel`, `sqlx`).
3. Não exigir arquivos SQL soltos no sistema de arquivos do usuário.
4. Ser testáveis contra um banco de dados em memória.

Opções:

- `rusqlite_migration` inline (`const MIGRATIONS: &[M]`).
- `rusqlite_migration` por diretório (recurso `from-directory`).
- `refinery` com arquivos SQL embutidos via `include_str!`.
- Script manual executando PRAGMAs.

## Decisão

Usar `rusqlite_migration` no modo inline:

```rust
use rusqlite_migration::{Migrations, M};

const MIGRATIONS: &[M] = &[
    M::up(include_str!("../../migrations/0001_initial.sql")),
    // ...
];
```

Os arquivos `.sql` vivem em `migrations/` no repositório como uma **referência humana** (revisão, diff, snapshot), mas são embutidos no binário via `include_str!`. A fonte da verdade em tempo de execução é o slice constante.

O estado do esquema é controlado pelo `PRAGMA user_version` do SQLite — sem tabela auxiliar.

As migrações rodam automaticamente em cada `backlog <qualquer comando>` via `Migrations::from_slice(...).to_latest(&mut conn)` durante o bootstrap da conexão.

Um snapshot `insta` do resultado de `SELECT type, name, sql FROM sqlite_master ORDER BY name` valida o esquema final após a aplicação de todas as migrações.

## Consequências

**Positivas:**
- Binário auto-contido — o usuário nunca vê arquivos SQL.
- Zero CLI externa; atualizações do binário aplicam o novo esquema de forma transparente.
- O teste de migração é trivial: `Connection::open_in_memory()` + `to_latest`.
- O snapshot `insta` transforma "alterei uma migração" em um diff de esquema revisável.

**Negativas:**
- Migrações já publicadas não podem ser modificadas — uma nova migração de ajuste é necessária. Regra: **uma migração é imutável após o lançamento.** As mudanças de esquema devem ser sempre aditivas ou compensatórias.
- `include_str!` faz com que os arquivos .sql se tornem `&'static str` — muito pequeno em tempo de execução, com custo insignificante.

## Alternativas Consideradas

- **`from-directory`** — exigiria o envio de arquivos junto com o binário (quebra o "binário único"); descartada.
- **`refinery`** — similar em capacidade, mas o `rusqlite_migration` é mais leve e usa o `user_version` nativo, evitando uma tabela auxiliar.
- **Migrações ad-hoc via código Rust** — rejeitada: perde o contrato declarativo SQL que é fácil de revisar em PRs.

## Relacionados

- [ADR-0002 — Satélites](0002-tasks-atomica-com-satelites.md) — a promoção EAV → coluna gera uma nova migração imutável.
