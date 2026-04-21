# ADR-0004 — Contrato de stdout/stderr e `--format` Universal

- **Status:** Aceito
- **Data:** 2026-04-20

## Contexto

Um CLI usado em pipelines deve ter uma disciplina rigorosa de I/O (Entrada/Saída). Dois tipos de bugs são extremamente difíceis de reverter uma vez que os usuários (ou scripts) começam a depender do comportamento:

1. **Mixagem de Canais** — imprimir logs/progresso no `stdout` quebra o `backlog list | grep X`.
2. **Formato Único** — a saída legível por humanos (tabelas coloridas) não pode ser analisada por outras ferramentas; os scripts começam a usar regex em códigos ANSI, e qualquer mudança estética quebra os consumidores.

Requisito explícito do projeto: consumir a saída via agentes de IA sem precisar refatorar a arquitetura depois. Isso exige um JSON estável desde o primeiro dia.

## Decisão

### Contrato de Canal

- **`stdout`**: exclusivamente **dados** do comando (tabela, JSON, tsv). Nada mais.
- **`stderr`**: tudo o que não for dado — logs (`tracing`), progresso, prompts interativos (`inquire`) e mensagens de erro (`miette`).
- Implementado através do módulo `src/output.rs`, expondo os auxiliares `stdout_data()` / `stderr_msg()`. Nenhum `println!` direto em subcomandos — verificações de linting/revisão bloqueiam isso.
- `tracing-subscriber` configurado para escrever no `stderr`.
- `is-terminal` detecta se o `stdout` é um terminal; se não for, ele desabilita automaticamente as cores ANSI.

### `--format` Universal

Cada comando de leitura (`list`, `show`, `export`, `projects list`) aceita o `--format`:

- `table` — padrão interativo, colorido, para humanos.
- `json` — estável, documentado, para scripts e agentes de IA.
- Possíveis adições futuras: `tsv`, `yaml`, `markdown`. Cada uma é aditiva, sem quebrar o `table`/`json`.

O esquema JSON segue as convenções `snake_case` e inclui `schema_version` no envelope para uma evolução controlada:

```json
{
  "schema_version": 1,
  "data": [ ... ]
}
```

## Consequências

**Positivas:**
- `backlog list | jq` e `backlog export --format=json` funcionam desde o primeiro dia.
- Agentes de IA consomem um JSON estável; mudanças visuais na `table` não os impactam.
- Logs detalhados (`-vv`) nunca quebram os pipes.
- Erros via `miette` permanecem visíveis no terminal mesmo quando o `stdout` está sendo capturado.

**Negativas:**
- Manter dois formatos dá mais trabalho por comando. Mitigação: renderizadores centralizados em `src/format.rs`; os subcomandos produzem `Vec<Struct>` e os entregam ao renderizador.
- A evolução do esquema JSON exige disciplina (incremento da `schema_version`, novo ADR quando houver quebra).
- Os desenvolvedores acostumados com `println!` precisam aprender a regra. Mitigação: documentar no `CLAUDE.md` do repositório e bloquear em revisões de código.

## Alternativas Consideradas

- **Apenas `--json` como uma flag booleana** — rejeitada: impede a adição de `tsv`/`markdown` sem uma nova flag; `--format=X` é extensível.
- **JSON como padrão, tabela como flag** — rejeitada: a UX interativa sofre; os usuários (humanos) são os principais consumidores no uso diário.
- **Sem separação de stderr/stdout (uso casual)** — rejeitada: o custo de corrigir isso mais tarde é extremamente alto.

## Relacionados

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — a saída nunca vaza entre tenants.
