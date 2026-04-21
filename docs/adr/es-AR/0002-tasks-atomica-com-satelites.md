# ADR-0002 — Tabla `tasks` atómica con satélites extensibles

- **Estado:** Aceptado
- **Fecha:** 2026-04-20

## Contexto

Los backlogs personales crecen para cualquier lado: estimaciones, área/servicio, esfuerzo, anclas de plan, links de PR, referencias externas. Si modelás todo como columnas en `tasks`, vas a tener que meter una migración por cada idea nueva. Por otro lado, modelar todo como EAV (Entidad-Atributo-Valor) de entrada te hace perder performance en las consultas comunes y te complica los filtros.

Lo que el usuario pide explícitamente es una arquitectura evolutiva: **arrancar simple y permitir extensiones sin tener que andar tocando todo**.

## Decisión

`tasks` es la tabla central con el conjunto mínimo que cada tarea tiene: `id`, `project_id`, `title`, `body`, `status`, `priority`, `type`, `parent_id`, `archived_at`, `completed_at`, `created_at`, `updated_at`.

Cada dimensión más allá de este núcleo va a tablas satélite:

- **`tags` + `task_tags`** — etiquetado libre por tenant.
- **`task_attributes(task_id, key, value)`** — EAV para campos ad-hoc (estimaciones, servicio, área, referencias externas).
- **`task_links(from_id, to_id, kind)`** — relaciones tipadas (`blocks`, `relates`, `duplicates`, `spawned-from-plan`).
- **`task_events(task_id, ts, kind, payload)`** — registro append-only de cambios (cambio de estado, etiqueta agregada, sugerencia de IA, etc.). `payload` es `TEXT` que contiene **JSON serializado** (esquema libre por `kind`), nunca una cadena arbitraria; los consumidores pueden inspeccionarlo con `json_extract()` de SQLite sin necesidad de promover dimensiones.

Regla de Promoción: cuando una clave en `task_attributes` aparece en ≥80% de las tareas activas, o se convierte en algo por lo que filtrás todo el tiempo, se pasa a una columna en `tasks` con una migración nueva. La promoción es una decisión que se toma a conciencia, nada de automatismos.

## Consecuencias

**Positivas:**
- Sumar una nueva dimensión no te cuesta ninguna migración (es solo una clave nueva en `task_attributes`).
- Un núcleo chico mantiene las consultas comunes (`list`, `show`) rápidas y los índices simples.
- `task_events` te da una auditoría y una base para métricas futuras (lead time, throughput) sin tener que rearmar todo.
- Los satélites independientes evolucionan a su propio ritmo.

**Negativas:**
- EAV penaliza las consultas que filtran por claves raras (escaneo de `task_attributes`). Mitigación: crear un índice parcial cuando haga falta; promover a una columna cuando se vea que es importante.
- Más joins en consultas que necesitan juntar todo (aceptado — `show` hace 4-5 joins, corre localmente en SQLite y responde en milisegundos).
- Tener dos formas de extender (atributos vs. etiquetas) puede marear un poco — **regla general:** las etiquetas son filtros categóricos libres; los atributos son pares clave-valor. Documentar en el README.

## Alternativas Consideradas

- **JSON en una sola columna** (`tasks.metadata JSON`) — rechazado: SQLite soporta JSON1 pero la indexación es frágil; EAV puro proporciona mejores diagnósticos y migración futura.
- **Schema-first estricto (cada dimensión se convierte en una columna)** — rechazado: viola la premisa evolutiva; cada nueva idea cuesta una migración + lanzamiento.
- **Almacén de documentos (ej: un archivo por tarea)** — rechazado: pierde las consultas relacionales y todo el sentido de SQLite.

## Anexo: schema de payloads de `task_events`

Cada evento tiene un `kind` y un `payload` JSON. La tabla de abajo documenta el conjunto que el CLI emite en la versión actual. Los consumidores deben tolerar campos desconocidos (forward-compat).

| `kind`           | Emitido por                         | Payload                                                      |
|------------------|-------------------------------------|--------------------------------------------------------------|
| `created`        | `backlog add`                       | `{ "title": string, "type": string, "priority": integer }`   |
| `status_changed` | `backlog done`                      | `{ "from": string, "to": string }`                           |
| `archived`       | `backlog archive`                   | `{}`                                                         |
| `field_changed`  | `backlog edit` (un evento por campo modificado) | `{ "field": string, "from": any\|null, "to": any\|null }` |
| `tag_added`      | `backlog tag add`                   | `{ "tag": string }`                                          |
| `tag_removed`    | `backlog tag remove`                | `{ "tag": string }`                                          |
| `link_added`     | `backlog link ... --kind X`         | `{ "to": integer, "kind": string }`                          |
| `link_removed`   | `backlog link ... --kind X --remove`| `{ "to": integer, "kind": string }`                          |
| `attr_set`       | `backlog attr set`                  | `{ "key": string, "from": string\|null, "to": string }`      |
| `attr_unset`     | `backlog attr unset`                | `{ "key": string }`                                          |

Invariantes:

- `payload` siempre es un objeto JSON (nunca un escalar ni un array en el tope).
- Los campos `from`/`to` en `field_changed` y `attr_set` pueden ser `null` (campo vaciado o que no existía antes).
- Los nombres de `kind` son estables: cambiarlos rompe a los consumidores externos y obliga a un ADR nuevo que supersede este.
- Los `kind` nuevos son aditivos — `backlog events` ignora un `kind` desconocido sin error.

## Relacionados

- [ADR-0001 — Tenencia](0001-tenancy-estrita-por-projeto.md) — todos los satélites heredan `project_id` vía `task_id`.
- [ADR-0003 — Migraciones Inline](0003-migrations-inline.md) — la promoción EAV → columna es una nueva migración.
