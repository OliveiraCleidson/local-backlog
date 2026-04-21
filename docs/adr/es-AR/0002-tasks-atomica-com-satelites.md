# ADR-0002 — Tabla `tasks` atómica con satélites extensibles

- **Estado:** Aceptado
- **Fecha:** 2026-04-20

## Contexto

Los backlogs personales crecen en dimensiones imprevistas: estimaciones, área/servicio, esfuerzo, anclas de plan, enlaces de PR, referencias externas. Modelar todo como columnas en `tasks` obliga a una migración por cada nueva idea. Modelar todo como EAV (Entidad-Atributo-Valor) desde el inicio diluye el rendimiento de las consultas comunes y complica el filtrado.

El requisito explícito del usuario: arquitectura evolutiva — **empezar simple, permitir extensión sin modificación**.

## Decisión

`tasks` es la tabla central con el conjunto mínimo que cada tarea tiene: `id`, `project_id`, `title`, `body`, `status`, `priority`, `type`, `parent_id`, `archived_at`, `completed_at`, `created_at`, `updated_at`.

Cada dimensión más allá de este núcleo va a tablas satélite:

- **`tags` + `task_tags`** — etiquetado libre por tenant.
- **`task_attributes(task_id, key, value)`** — EAV para campos ad-hoc (estimaciones, servicio, área, referencias externas).
- **`task_links(from_id, to_id, kind)`** — relaciones tipadas (`blocks`, `relates`, `duplicates`, `spawned-from-plan`).
- **`task_events(task_id, ts, kind, payload)`** — registro append-only de cambios (cambio de estado, etiqueta agregada, sugerencia de IA, etc.). `payload` es `TEXT` que contiene **JSON serializado** (esquema libre por `kind`), nunca una cadena arbitraria; los consumidores pueden inspeccionarlo con `json_extract()` de SQLite sin necesidad de promover dimensiones.

Regla de Promoción: cuando una clave en `task_attributes` aparece en ≥80% de las tareas activas, o se convierte en un criterio de filtrado recurrente, se migra a una columna en `tasks` mediante una nueva migración. La promoción es una decisión consciente, no automática.

## Consecuencias

**Positivas:**
- Agregar una nueva dimensión es cero migración (nueva clave en `task_attributes`).
- Un núcleo pequeño mantiene las consultas comunes (`list`, `show`) rápidas y los índices simples.
- `task_events` proporciona auditoría y una base para métricas futuras (lead time, throughput) sin reequipamiento.
- Los satélites independientes evolucionan a su propio ritmo.

**Negativas:**
- EAV penaliza las consultas que filtran por claves raras (escaneo de `task_attributes`). Mitigación: crear un índice parcial cuando sea necesario; promover a una columna cuando demuestre su importancia.
- Más joins en consultas que necesitan reunir todo (aceptado — `show` realiza 4-5 joins, se ejecuta localmente en SQLite y responde en ms).
- Dos mecanismos de extensión (atributos vs. etiquetas) pueden ser confusos — **regla general:** las etiquetas son filtros categóricos libres; los atributos son pares clave-valor. Documentar en el README.

## Alternativas Consideradas

- **JSON en una sola columna** (`tasks.metadata JSON`) — rechazado: SQLite soporta JSON1 pero la indexación es frágil; EAV puro proporciona mejores diagnósticos y migración futura.
- **Schema-first estricto (cada dimensión se convierte en una columna)** — rechazado: viola la premisa evolutiva; cada nueva idea cuesta una migración + lanzamiento.
- **Almacén de documentos (ej: un archivo por tarea)** — rechazado: pierde las consultas relacionales y todo el sentido de SQLite.

## Relacionados

- [ADR-0001 — Tenencia](0001-tenancy-estrita-por-projeto.md) — todos los satélites heredan `project_id` vía `task_id`.
- [ADR-0003 — Migraciones Inline](0003-migrations-inline.md) — la promoción EAV → columna es una nueva migración.
