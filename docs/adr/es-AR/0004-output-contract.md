# ADR-0004 — Contrato de stdout/stderr y `--format` Universal

- **Estado:** Aceptado
- **Fecha:** 2026-04-20

## Contexto

Una CLI que se usa en pipelines tiene que tener una disciplina de E/S (I/O) bien estricta. Hay dos tipos de errores que son un dolor de cabeza arreglar una vez que los usuarios (o los scripts) empiezan a depender de cómo se comporta la herramienta:

1. **Mezclar los canales** — si mandás logs o progreso a `stdout`, rompés el `backlog list | grep X`.
2. **Formato único** — la salida para humanos (tablas con colores) no sirve para que la procesen otras herramientas; los scripts empiezan a usar regex sobre códigos ANSI, y cualquier cambio estético termina rompiendo todo para los consumidores.

Lo que el proyecto pide explícitamente es poder consumir la salida con agentes de IA sin tener que rediseñar la arquitectura más adelante. Esto exige tener un JSON estable desde el primer día.

## Decisión

### Contrato de Canal

- **`stdout`**: solo para los **datos** del comando (tabla, JSON, tsv). Nada más.
- **`stderr`**: para todo lo que no sean datos — logs (`tracing`), progreso, prompts interactivos (`inquire`) y mensajes de error (`miette`).
- Implementado con el módulo `src/output.rs` que expone los helpers `stdout_data()` / `stderr_msg()`. Nada de `println!` directo en los subcomandos — los lints y las reviews lo van a bloquear.
- `tracing-subscriber` configurado para escribir en `stderr`.
- `is-terminal` detecta si `stdout` es una TTY; si no lo es, vuela automáticamente los colores ANSI.

### `--format` Universal

Cada comando de lectura (`list`, `show`, `export`, `projects list`) acepta `--format`:

- `table` — el default interactivo, con colores, para humanos.
- `json` — estable, documentado, para scripts y agentes de IA.
- Posibles agregados futuros: `tsv`, `yaml`, `markdown`. Cada uno se suma sin romper `table`/`json`.

El esquema JSON usa la convención `snake_case` e incluye una `schema_version` en el sobre para ir evolucionando sin drama:

```json
{
  "schema_version": 1,
  "data": [ ... ]
}
```

## Consecuencias

**Positivas:**
- `backlog list | jq` y `backlog export --format=json` andan de entrada.
- Los agentes de IA consumen un JSON estable; los cambios visuales en `table` no los afectan.
- Los logs detallados (`-vv`) nunca te van a romper un pipe.
- Los errores por `miette` siguen apareciendo en la terminal aunque estés capturando el `stdout`.

**Negativas:**
- Mantener dos formatos te lleva más laburo por comando. Mitigación: renderizadores centralizados en `src/format.rs`; los subcomandos tiran un `Vec<Struct>` y se lo pasan al renderizador.
- Evolucionar el esquema JSON requiere disciplina (ir subiendo la `schema_version` y meter un ADR nuevo cuando haya cambios que rompan todo).
- Los devs que estén acostumbrados al `println!` van a tener que aprenderse la regla. Mitigación: dejarlo escrito en el `CLAUDE.md` del repo y rebotarlo en el code review.

## Alternativas Consideradas

- **Solo `--json` como un flag booleano** — rechazado: impide agregar `tsv`/`markdown` sin un nuevo flag; `--format=X` es extensible.
- **JSON por defecto, tabla como flag** — rechazado: la experiencia de usuario interactiva se ve afectada; los usuarios humanos son los principales consumidores en el uso diario.
- **Sin separación de stderr/stdout (uso casual)** — rechazado: el costo de arreglar esto más tarde es extremadamente alto.

## Relacionados

- [ADR-0001 — Tenencia](0001-tenancy-estrita-por-projeto.md) — la salida nunca se filtra entre tenants.
