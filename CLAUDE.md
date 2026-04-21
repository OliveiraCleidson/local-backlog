# local-backlog — Contexto do projeto

CLI em Rust para gerenciar backlog pessoal em SQLite local, isolado por projeto (tenancy estrita). Um binário único, instalável via `cargo install`, que opera sobre `~/.local-backlog/backlog.db` agregando N projetos do usuário sem permitir vazamento entre eles.

## Princípios não-negociáveis

1. **Tenancy estrita.** O projeto inferido da CWD é o único escopo visível em comandos de dados. Nenhuma flag `--all-projects`. A única superfície cross-tenant é `backlog projects ...` (meta). Ver ADR-0001.
2. **`tasks` é atômico; extensões vão em satélites** (`tags`, `task_attributes`, `task_links`, `task_events`). Coluna nova em `tasks` só quando chave EAV prova peso (≥80% das tasks). Ver ADR-0002.
3. **Output contract.** Dados → `stdout`. Logs, progresso, prompts, erros → `stderr`. Nenhum `println!` direto em subcomandos — tudo passa pelo helper em `src/output.rs`. Ver ADR-0004.
4. **`--format=table|json` universal** em todo comando de leitura desde o dia 1. JSON com envelope `{ "schema_version": N, "data": ... }`. Ver ADR-0004.
5. **Migrations imutáveis após release.** Arquivos `.sql` em `migrations/` são embutidos via `include_str!` em `MIGRATIONS: &[M]`. Mudança de schema sempre aditiva; correção é nova migration. Ver ADR-0003.
6. **Banco real em teste.** `Connection::open_in_memory()` + mesmas migrations. Sem mock de DB.

## Stack

| Camada | Crate |
|---|---|
| CLI | `clap` (derive) + `clap-verbosity-flag` + `clap_complete` |
| SQLite | `rusqlite` (feature `bundled`) |
| Migrations | `rusqlite_migration` (inline via `include_str!`) |
| Config | `figment` + `toml` |
| Serde | `serde`, `serde_json`, `toml` |
| Erros | `thiserror` + `miette` |
| Logs | `tracing` + `tracing-subscriber` (→ `stderr`) |
| Cores | `owo-colors` + `is-terminal` |
| Prompts | `inquire` |
| Testes | `cargo test` + `assert_cmd` + `insta` |
| DX | `just` + `bacon` + `clippy` + `fmt` |

Release profile otimizado para tamanho: `opt-level='z'`, `lto=true`, `codegen-units=1`, `panic='abort'`, `strip=true`.

## Estrutura

```
src/
├── main.rs          # entry + clap + tracing subscriber
├── cli/             # subcomandos (init, add, list, show, done, archive, tag, link, ...)
├── output.rs        # stdout_data | stderr_msg — NUNCA println! direto em subcomandos
├── format.rs        # renderers table / json
├── config.rs        # figment
├── db/
│   ├── mod.rs
│   ├── migrations.rs  # const MIGRATIONS: &[M] com include_str!
│   └── repo/          # project_repo, task_repo, tag_repo
├── domain/          # Task, Project, Tag (structs puros, sem SQL)
└── error.rs         # thiserror + Diagnostic de miette

migrations/          # .sql revisável (embutido no binário via include_str!)
tests/               # assert_cmd + snapshots insta
docs/adr/            # ADRs multi-locale (canônico: pt-BR)
```

## Fluxos obrigatórios

- **Antes de alterar schema:** ler ADR-0003. Criar nova migration numerada, nunca editar migration publicada. Atualizar snapshot `insta` do schema.
- **Antes de adicionar comando:** garantir que tenancy é aplicada (filtro `project_id` na query ou falha explícita para tenant errado).
- **Antes de imprimir algo:** decidir — é dado (`stdout_data`) ou mensagem (`stderr_msg`)? Nada de `println!`/`eprintln!` cru.
- **Antes de expor nova saída de dado:** implementar `table` E `json` no renderer. JSON sempre com `schema_version`.

## Testes esperados

- Snapshot `insta` do schema resultante após todas as migrations.
- Teste de cada trigger de tenancy (parent_id, task_tags, task_links) bloqueando insert/update cross-project.
- `assert_cmd` + snapshot para output de cada comando de leitura (`list`, `show`, `export`, `projects list`), em ambos os formatos.
- Banco em memória em todos os testes de integração.

## Commits e versionamento

- **Idioma:** inglês.
- **Formato:** [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/), modo imperativo.
- Tipos usados: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `build`, `ci`, `perf`, `style`.
- Escopo opcional entre parênteses quando útil: `feat(cli): add projects list command`.
- Breaking change: sufixo `!` no tipo/escopo (`feat!: ...`) e rodapé `BREAKING CHANGE:`.
- Corpo do commit explica **o porquê**, não o quê (o diff mostra o quê).

Exemplos:
- `feat(db): add tenancy triggers for task_tags and task_links`
- `fix(output): route inquire prompts to stderr to preserve stdout pipe`
- `refactor(cli): extract format renderers into format.rs`
- `docs(adr): supersede ADR-0002 with new EAV promotion policy`

## ADRs

Toda decisão estruturante (irreversível ou cara de reverter) vira ADR em `docs/adr/`. Canônico em `pt-BR`; traduções `en` e `es-AR` acompanham no mesmo commit. Nunca editar ADR `Accepted` — criar novo que supersede.

Leitura sob demanda quando a tarefa tocar: schema/migrations (0001, 0002, 0003), I/O e formatos de saída (0004), identificação de projeto e registry (0005).

## Comandos comuns (`justfile`)

- `just check` — fmt + clippy (`-D warnings`) + test
- `just test` — `cargo test` com snapshots atualizáveis via `cargo insta review`
- `just build` — release otimizado
- `just install` — `cargo install --path .`
- `just bacon` — loop DX em background

## Armadilhas conhecidas

- **Esquecer `WHERE project_id = ?`** em query de leitura — tenancy é perdida silenciosamente. Padrão: toda função em `repo/` recebe `project_id` como primeiro parâmetro explícito; não existe variante "global" para dados de tasks.
- **`println!` numa rotina de erro** — quebra pipe. Erros vão via `miette::Result` do `main`, que renderiza em `stderr` automaticamente.
- **Editar migration já commitada** — proibido após release. Fix é migration nova.
- **Registry TOML fora de sincronia com tabela `projects`** — sempre gravar DB primeiro, depois reescrever TOML; `backlog doctor` detecta divergência.
