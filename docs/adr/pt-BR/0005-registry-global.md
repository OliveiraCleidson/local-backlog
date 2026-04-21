# ADR-0005 — Identificação de Projetos via Registro Global

- **Status:** Aceito
- **Data:** 2026-04-20

## Contexto

A tenancy estrita ([ADR-0001](0001-tenancy-estrita-por-projeto.md)) exige resolver "qual projeto é este?" a cada invocação do CLI. Três estratégias foram consideradas:

1. **Arquivo `.local-backlog` no repositório** com um `project_id`. Versionável no git; polui os repositórios; se não for commitado, desaparece.
2. **Hash do caminho do repositório git** (`git rev-parse --show-toplevel`). Zero configuração; um clone em outra máquina torna-se silenciosamente um "projeto diferente"; não funciona fora do git.
3. **Registro Global** em `~/.local-backlog/registry.toml` mapeando `caminho_absoluto → id_do_projeto`. Zero poluição no repositório; exige o `backlog relink` ao mover pastas; fácil de listar projetos.

## Decisão

Usar a opção 3: um único registro global.

### Estrutura

```toml
# ~/.local-backlog/registry.toml
[[projects]]
id = 1
name = "local-backlog"
root_path = "/Users/cleidson/github/personal/local-backlog"

[[projects]]
id = 2
name = "hub"
root_path = "/Users/cleidson/github/personal/hub"
```

O registro é um espelho canônico da tabela `projects` do SQLite — a fonte da verdade é o banco de dados; o arquivo TOML existe para inspeção humana e como um cache de busca rápida.

### Sincronização

As alterações nos metadados do projeto são aplicadas primeiro ao SQLite e depois persistidas no `registry.toml` através da reescrita atômica do arquivo:

- `backlog init` insere o projeto em `projects` e escreve a entrada correspondente no registro.
- `backlog projects relink <id|name> --path <novo>` atualiza `projects.root_path` e reescreve o registro.
- `backlog projects archive <id|name>` atualiza `projects.archived_at` e reescreve o registro.
- `backlog doctor` compara o SQLite e o `registry.toml`, relatando entradas ausentes, caminhos obsoletos, caminhos duplicados e IDs incompatíveis.

### Resolução

Para cada comando:

1. Canonizar o CWD (resolver links simbólicos, normalizar).
2. Subir a árvore de diretórios procurando por uma correspondência em `root_path`.
3. Se encontrado → usar o `project_id` correspondente.
4. Se não encontrado → erro via `miette` sugerindo `backlog init` ou `backlog projects relink`.

### Comandos de Metadados (O Único Namespace Cross-Tenant)

- `backlog projects list` — mostra todos os projetos registrados (id, nome, caminho, contagem de tarefas).
- `backlog projects show <id|name>` — metadados de um projeto.
- `backlog projects relink <id|name> --path <novo>` — atualiza o `root_path` quando o repositório muda de pasta.
- `backlog projects archive <id|name>` — marca um projeto como arquivado (não aparece na `list`); os dados são preservados.

## Consequências

**Positivas:**
- Nenhum arquivo `local-backlog` dentro dos repositórios — não polui o `git status`.
- Funciona fora do git (projetos sem controle de versão).
- `backlog projects list` fornece um inventário global instantâneo.
- Mudanças de pasta → um simples `relink` resolve; sem perda de dados.

**Negativas:**
- Perder `~/.local-backlog/` quebra os vínculos (banco de dados + registro vivem juntos). Mitigação: a pasta inteira é trivial de fazer backup; o usuário pode versioná-la em seus dotfiles se desejar.
- Dois checkouts diferentes do mesmo repositório em caminhos diferentes tornam-se projetos distintos (isso pode ser intencional ou um erro). Mitigação: `backlog init` detecta repositórios Git e pergunta se é um novo projeto ou uma duplicata; `doctor` sinaliza caminhos com a mesma `origin` mapeados para IDs diferentes.
- A portabilidade entre máquinas exige a sincronização de `~/.local-backlog/` (ou aceitar estados independentes por máquina). Aceito — a ferramenta é projetada para uma única máquina.

## Alternativas Consideradas

- **`.local-backlog` no repositório** — rejeitada: polui os repositórios do usuário, exige disciplina no `.gitignore` ou um commit com um ID acoplado.
- **Hash de `git remote get-url origin`** — rejeitada: falha em repositórios sem uma origem, em forks e em repositórios não-git.
- **Nome do diretório como chave** — rejeitada: dois repositórios chamados `api/` colidiriam.

## Relacionados

- [ADR-0001 — Tenancy](0001-tenancy-estrita-por-projeto.md) — o registro é o mecanismo que provê o ID do tenant a partir do CWD.
