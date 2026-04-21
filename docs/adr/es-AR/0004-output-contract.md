# ADR-0004 — Contrato de stdout/stderr y `--format` Universal

- **Estado:** Aceptado
- **Fecha:** 2026-04-20

## Contexto

Una CLI utilizada en pipelines debe tener una disciplina de E/S (I/O) estricta. Dos tipos de errores son extremadamente difíciles de revertir una vez que los usuarios (o scripts) comienzan a depender del comportamiento:

1. **Canales Mezclados** — imprimir logs/progreso en `stdout` rompe `backlog list | grep X`.
2. **Formato Único** — la salida humana (tablas coloreadas) no es procesable por otras herramientas; los scripts comienzan a usar regex sobre códigos ANSI, y cualquier cambio estético rompe a los consumidores.

Requisito explícito del proyecto: consumir la salida a través de agentes de IA sin tener que rediseñar la arquitectura después. Esto requiere un JSON estable desde el primer día.

## Decisión

### Contrato de Canal

- **`stdout`**: exclusivamente **datos** del comando (tabla, JSON, tsv). Nada más.
- **`stderr`**: todo lo que no sean datos — logs (`tracing`), progreso, prompts interactivos (`inquire`) y mensajes de error (`miette`).
- Implementado a través del módulo `src/output.rs` que expone los ayudantes `stdout_data()` / `stderr_msg()`. Sin `println!` directo en los subcomandos — los controles de linting/revisión lo bloquean.
- `tracing-subscriber` configurado para escribir en `stderr`.
- `is-terminal` detecta si `stdout` es una TTY; si no, desactiva automáticamente los colores ANSI.

### `--format` Universal

Cada comando de lectura (`list`, `show`, `export`, `projects list`) acepta `--format`:

- `table` — predeterminado interactivo, coloreado, para humanos.
- `json` — estable, documentado, para scripts y agentes de IA.
- Posibles adiciones futuras: `tsv`, `yaml`, `markdown`. Cada una es aditiva, sin romper `table`/`json`.

El esquema JSON sigue las convenciones `snake_case` e incluye `schema_version` en el sobre para una evolución controlada:

```json
{
  "schema_version": 1,
  "data": [ ... ]
}
```

## Consecuencias

**Positivas:**
- `backlog list | jq` y `backlog export --format=json` funcionan desde el primer día.
- Los agentes de IA consumen un JSON estable; los cambios visuales en `table` no los afectan.
- Los logs detallados (`-vv`) nunca rompen los pipes.
- Los errores a través de `miette` permanecen visibles en la terminal incluso cuando se está capturando `stdout`.

**Negativas:**
- Mantener dos formatos implica más trabajo por comando. Mitigación: renderizadores centralizados en `src/format.rs`; los subcomandos producen `Vec<Struct>` y los entregan al renderizador.
- La evolución del esquema JSON requiere disciplina (incremento de `schema_version`, nuevo ADR cuando haya cambios disruptivos).
- Los desarrolladores acostumbrados a `println!` necesitan aprender la regla. Mitigación: documentar en el `CLAUDE.md` del repositorio y bloquear en las revisiones de código.

## Alternativas Consideradas

- **Solo `--json` como un flag booleano** — rechazado: impide agregar `tsv`/`markdown` sin un nuevo flag; `--format=X` es extensible.
- **JSON por defecto, tabla como flag** — rechazado: la experiencia de usuario interactiva se ve afectada; los usuarios humanos son los principales consumidores en el uso diario.
- **Sin separación de stderr/stdout (uso casual)** — rechazado: el costo de arreglar esto más tarde es extremadamente alto.

## Relacionados

- [ADR-0001 — Tenencia](0001-tenancy-estrita-por-projeto.md) — la salida nunca se filtra entre tenants.
