# ADR-0000 — Rust como linguagem de implementação com foco em aprendizado estratégico

- **Status:** Aceito
- **Data:** 2026-04-20
- **Autor:** Cleidson Oliveira

## Contexto

O `local-backlog` nasce com duplo propósito explícito:

1. **Ferramenta real:** CLI que o autor vai usar em todos os seus repositórios pessoais para gerenciar backlog, instalar em diferentes máquinas via `cargo install` e consumir com agentes de IA.
2. **Veículo de aprendizado:** Rust é uma aposta de longo prazo na trajetória pública do autor (âncora Staff/Distinguished Engineer, 2–3 anos), junto a Go/AWS/IA. Um escopo pequeno, bem delimitado e de domínio conhecido é o ambiente ideal para aprender a linguagem sem pressão de prazo externo.

As alternativas avaliadas foram Go (binário único, já familiar, velocidade de iteração) e Python (protótipo rápido, distribuição inferior). Go entregaria antes; Python seria o caminho de menor fricção. Nenhum dos dois contribui para a lacuna declarada de profundidade em Rust.

## Decisão

Rust é a linguagem de implementação do `local-backlog`. A decisão tem três implicações operacionais:

1. **Ritmo favorece compreensão sobre entrega.** Não há deadline externo. Se uma feature exige entender lifetimes, traits genéricas ou `async` com detalhes, o tempo gasto é parte do retorno do projeto — não desvio.
2. **Escolhas de crate podem privilegiar valor de aprendizado** quando duas opções são equivalentes em capacidade. Exemplo: preferir `figment` sobre `config-rs` por expor conceitos de providers; preferir `thiserror`+`miette` híbrido sobre `anyhow` por ensinar o design de erros tipados.
3. **Código gerado/assistido por IA passa por revisão humana explicativa.** O objetivo não é apenas funcionar, é o autor entender cada bloco não-trivial. Revisão de código (humana ou adversarial via Codex) deve explicar idiomas quando opacos, não apenas aprovar.

Esta decisão é fundamental — todos os demais ADRs (0001–NNNN) assumem Rust como pressuposto.

## Consequências

**Positivas:**
- Projeto pequeno vira campo de treino controlado: `tasks` atômicas + satélites cobre `rusqlite`, migrations, triggers, serde, error handling tipado, CLI parsing — os idiomas centrais do ecossistema.
- Retorno do esforço é composto: ferramenta pessoal útil + profundidade em Rust + peça de portfólio pública alinhada à âncora declarada.
- Escopo fechado evita o anti-padrão "aprender Rust em produção crítica".

**Negativas:**
- Velocidade inicial menor que Go/Python. Mitigação: sem deadline; o custo é aceito por construção.
- Curva de aprendizado pode produzir decisões que um engenheiro Rust sênior reverteria depois. Mitigação: ADRs 0001–NNNN fixam os limites arquiteturais (tenancy, schema, output contract); dentro desses limites, iteração e reescrita são bem-vindas.
- Aceitável **over-engineering pedagógico** dentro dos limites: implementar uma trait própria para `Output` quando um `enum` bastaria, se isso ensinar trait design. Não aceitável fora dos limites: quebrar tenancy ou contrato de saída por curiosidade sintática.

## Alternativas Consideradas

- **Go** — rejeitada: reaproveitaria conhecimento existente sem contribuir para a lacuna estratégica declarada. Entregaria a ferramenta mais rápido, mas o custo de oportunidade é o aprendizado que não aconteceria.
- **Python** — rejeitada: protótipo rápido, mas distribuição (PyPI/pipx) inferior ao `cargo install` para o modelo de uso pretendido; e zero ganho estratégico de linguagem nova.
- **TypeScript/Node** — rejeitada pelos mesmos motivos de Python, com o agravante de runtime externo obrigatório.

## Relacionados

- [ADR-0001 — Tenancy estrita](0001-tenancy-estrita-por-projeto.md) — primeira decisão arquitetural que assume Rust como linguagem.
- [ADR-0003 — Migrations inline](0003-migrations-inline.md) — depende de recursos do ecossistema Rust (`include_str!`, `rusqlite_migration`).
