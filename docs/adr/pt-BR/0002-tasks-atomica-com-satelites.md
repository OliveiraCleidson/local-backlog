# ADR-0002 — Tabela `tasks` atômica com satélites extensíveis

- **Status:** Aceito
- **Data:** 2026-04-20

## Contexto

Os backlogs pessoais crescem em complexidade imprevista: estimativas, área/serviço, esforço, marcos de planejamento, links de PR, referências externas. Modelar tudo como colunas em `tasks` força uma migração para cada nova ideia. Modelar tudo como EAV (Entidade-Atributo-Valor) desde o início dilui a performance de consultas comuns e complica a filtragem.

O requisito explícito do usuário: arquitetura evolutiva — **começar simples, permitir extensão sem modificação**.

## Decisão

`tasks` é a tabela central com o conjunto mínimo que cada tarefa possui: `id`, `project_id`, `title`, `body`, `status`, `priority`, `type`, `parent_id`, `archived_at`, `completed_at`, `created_at`, `updated_at`.

Cada dimensão além desse núcleo vai para tabelas satélites:

- **`tags` + `task_tags`** — rotulagem livre por tenant.
- **`task_attributes(task_id, key, value)`** — EAV para campos ad-hoc (estimativas, serviço, área, referências externas).
- **`task_links(from_id, to_id, kind)`** — relacionamentos tipados (`blocks`, `relates`, `duplicates`, `spawned-from-plan`).
- **`task_events(task_id, ts, kind, payload)`** — log append-only de mudanças (mudança de status, tag adicionada, sugestão de IA, etc.). `payload` é `TEXT` contendo **JSON serializado** (schema livre por `kind`), nunca string arbitrária; consumidores podem inspecionar via `json_extract()` do SQLite sem precisar promover a colunas.

Regra de Promoção: quando uma chave em `task_attributes` aparece em ≥80% das tarefas ativas, ou se torna um critério de filtragem recorrente, ela é migrada para uma coluna em `tasks` através de uma nova migração. A promoção é uma decisão consciente, não automática.

## Consequências

**Positivas:**
- Adicionar uma nova dimensão é zero migração (nova chave em `task_attributes`).
- Um núcleo pequeno mantém as consultas comuns (`list`, `show`) rápidas e os índices simples.
- `task_events` fornece auditoria e uma base para métricas futuras (lead time, throughput) sem retrofit.
- Satélites independentes evoluem em seu próprio ritmo.

**Negativas:**
- EAV penaliza consultas que filtram por chaves raras (varredura de `task_attributes`). Mitigação: criar um índice parcial quando necessário; promover a coluna quando ela provar seu peso.
- Mais joins em consultas que precisam reunir tudo (aceito — `show` realiza 4-5 joins, roda localmente no SQLite e responde em ms).
- Dois mecanismos de extensão (atributos vs. tags) podem ser confusos — **regra prática:** tags são filtros categóricos livres; atributos são pares chave-valor. Documentar no README.

## Alternativas Consideradas

- **JSON em uma única coluna** (`tasks.metadata JSON`) — rejeitada: o SQLite suporta JSON1, mas a indexação é frágil; o EAV puro fornece melhores diagnósticos e migração futura.
- **Schema-first rígido (cada dimensão se torna uma coluna)** — rejeitada: viola a premissa evolutiva; cada nova ideia custa uma migração + release.
- **Armazenamento de documentos (ex: um arquivo por tarefa)** — rejeitada: perde consultas relacionais e todo o propósito do SQLite.

## Anexo: schema de payloads de `task_events`

Cada evento tem um `kind` e um `payload` JSON. O schema abaixo documenta o conjunto emitido pelo CLI na versão atual. Consumidores devem tolerar campos desconhecidos (forward-compat).

| `kind`           | Emitido por                         | Payload                                                      |
|------------------|-------------------------------------|--------------------------------------------------------------|
| `created`        | `backlog add`                       | `{ "title": string, "type": string, "priority": integer }`   |
| `status_changed` | `backlog done`                      | `{ "from": string, "to": string }`                           |
| `archived`       | `backlog archive`                   | `{}`                                                         |
| `field_changed`  | `backlog edit` (um evento por campo alterado) | `{ "field": string, "from": any\|null, "to": any\|null }` |
| `tag_added`      | `backlog tag add`                   | `{ "tag": string }`                                          |
| `tag_removed`    | `backlog tag remove`                | `{ "tag": string }`                                          |
| `link_added`     | `backlog link ... --kind X`         | `{ "to": integer, "kind": string }`                          |
| `link_removed`   | `backlog link ... --kind X --remove`| `{ "to": integer, "kind": string }`                          |
| `attr_set`       | `backlog attr set`                  | `{ "key": string, "from": string\|null, "to": string }`      |
| `attr_unset`     | `backlog attr unset`                | `{ "key": string }`                                          |

Invariantes:

- `payload` é sempre um objeto JSON (nunca escalar nem array no topo).
- Campos `from`/`to` em `field_changed` e `attr_set` podem ser `null` (zeragem ou campo previamente inexistente).
- Nomes de `kind` são estáveis: mudança quebra consumidores externos e exige ADR novo que supersede este.
- Novos `kind` são aditivos — `backlog events` ignora `kind` desconhecido sem erro.

## Relacionados

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — todos os satélites herdam `project_id` via `task_id`.
- [ADR-0003 — Migrações Inline](0003-migrations-inline.md) — a promoção EAV → coluna é uma nova migração.
