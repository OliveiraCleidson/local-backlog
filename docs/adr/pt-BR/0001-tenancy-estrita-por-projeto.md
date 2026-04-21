# ADR-0001 — Tenancy estrita por projeto

- **Status:** Aceito
- **Data:** 2026-04-20
- **Autor:** Cleidson Oliveira

## Contexto

O `local-backlog` mantém um único banco de dados SQLite em `~/.local-backlog/backlog.db` agregando N projetos do usuário. Duas abordagens eram possíveis:

1. **Banco compartilhado com filtros opcionais** — os comandos por padrão mostram o projeto atual, mas flags como `--all-projects` permitem uma visão agregada.
2. **Tenancy estrita** — o projeto inferido a partir do CWD (Diretório de Trabalho Atual) é o único escopo visível em cada operação de dados. Não existe superfície que agregue tarefas/tags/links entre projetos.

A Opção 1 parece pragmática, mas introduz classes de bugs persistentes: `backlog list` em um repositório mostrando tarefas de outro, tags colidindo entre projetos, links acidentais entre tarefas não relacionadas e a IA recebendo contexto vazado durante a exportação.

## Decisão

Adotar tenancy estrita baseada em projeto:

- Cada query para tarefa, tag, atributo, link e evento inclui `project_id = :current` inferido via CWD → registro.
- `tags.(project_id, name)` é único; `#auth` em dois projetos diferentes não colide nem compartilha um registro.
- Relacionamentos pai/filho, tarefa/tag e links tarefa/tarefa devem permanecer dentro do mesmo `project_id` — reforçado por triggers SQL tanto na inserção quanto na atualização.
- **Não existe flag `--all-projects`** em comandos de dados (`list`, `show`, `export`, etc.).
- A única superfície cross-tenant é o namespace meta `backlog projects ...` (list, show, archive, relink). Este namespace nunca expõe conteúdo de tarefa/tag — apenas metadados do registro.
- `backlog doctor` verifica inconsistências (tarefas órfãs, pais/tags/links cross-project) como parte do health check.

## Consequências

**Positivas:**
- Impossível vazar dados de um projeto para outro devido a erros de flag.
- Tags têm um namespace natural por tenant — reuso trivial de nomes (ex: `#bug`, `#auth`) sem configuração extra.
- A exportação de contexto para IA é segura por design: o JSON/Markdown emitido contém apenas o tenant atual.
- O modelo mental se alinha ao Git: cada repositório é seu próprio universo.

**Negativas:**
- Não há uma visão agregada de "tudo que tenho pendente no mundo". Mitigação: `backlog projects list` mostra contadores por projeto; dashboards cross-tenant estão fora de escopo (os usuários podem usar SQL diretamente no `.db` se um relatório excepcional for necessário).
- Não há mecanismo nativo para dois projetos diferentes que queiram compartilhar contexto (ex: microserviços relacionados). Workaround aceito: modelá-los como um único projeto usando as tags `#service-a` e `#service-b`.
- Triggers SQL adicionam superfície de teste (as migrações devem validar que os triggers bloqueiam inserções e atualizações inválidas).

## Alternativas Consideradas

- **`--all-projects` como uma flag opt-in** — rejeitada: introduz o modo cross-tenant na superfície pública, e uma flag bem-intencionada em um script se torna um vazamento permanente.
- **Banco de dados por projeto em `<repo>/.local-backlog.db`** — rejeitada: quebra a premissa de "ferramenta portátil, um `cargo install`, zero bagunça no repositório"; os usuários perderiam o histórico se esquecessem de adicioná-lo ao `.gitignore`.
- **Aplicar filtros apenas na camada de aplicação, sem triggers** — rejeitada: um bug de query que esqueça o `WHERE project_id` quebra a tenancy silenciosamente. Triggers são a defesa em profundidade.

## Relacionados

- [ADR-0005 — Registro Global](0005-registry-global.md) define como o tenant é resolvido a partir do CWD.
