# ADR-0001 — Tenencia estricta por proyecto

- **Estado:** Aceptado
- **Fecha:** 2026-04-20
- **Autor:** Cleidson Oliveira

## Contexto

`local-backlog` mantiene una sola base de datos SQLite en `~/.local-backlog/backlog.db` donde agrupa N proyectos del usuario. Dos enfoques eran posibles:

1. **Base de datos compartida con filtros opcionales** — los comandos por defecto muestran el proyecto actual, pero flags como `--all-projects` permiten una vista agregada.
2. **Tenencia estricta** — el proyecto inferido a partir del CWD (Directorio de Trabajo Actual) es el único alcance visible en cada operación de datos. No hay una superficie que agrupe tareas/etiquetas/enlaces entre proyectos.

La Opción 1 parece pragmática pero abre la puerta a bugs persistentes: ejecutar `backlog list` en un repo y ver tareas de otro, etiquetas que se pisan entre proyectos, enlaces accidentales entre tareas que no tienen nada que ver, o que la IA reciba contexto sucio durante la exportación.

## Decisión

Adoptar tenencia estricta basada en proyecto:

- Cada consulta de tarea, etiqueta, atributo, enlace y evento incluye `project_id = :current` inferido vía CWD → registro.
- `tags.(project_id, name)` es único; `#auth` en dos proyectos diferentes no se pisan ni comparten el registro.
- Las relaciones padre/hijo, tarea/etiqueta y enlaces tarea/tarea deben permanecer dentro del mismo `project_id` — esto lo refuerzan los triggers de SQL tanto en el insert como en el update.
- **No existe el flag `--all-projects`** para los comandos de datos (`list`, `show`, `export`, etc.).
- La única superficie cross-tenant es el espacio de nombres meta `backlog projects ...` (list, show, archive, relink). Este espacio de nombres nunca expone contenido de tareas o etiquetas — solo los metadatos del registro.
- `backlog doctor` chequea inconsistencias (tareas huérfanas, padres/etiquetas/enlaces cross-project) como parte del control de salud.

## Consecuencias

**Positivas:**
- Es imposible que se filtren datos de un proyecto a otro por pifiarle a un flag.
- Las etiquetas tienen un espacio de nombres natural por tenant — podés reutilizar nombres (ej: `#bug`, `#auth`) sin configuración extra.
- La exportación de contexto para la IA es segura por diseño: el JSON o Markdown emitido contiene solo el tenant actual.
- El modelo mental se alinea con Git: cada repositorio es su propio universo.

**Negativas:**
- No tenés una vista agregada de "todo lo que tengo pendiente en el mundo". Mitigación: `backlog projects list` muestra contadores por proyecto; los tableros cross-tenant están fuera de alcance (si necesitás un reporte puntual, podés usar SQL directamente sobre el `.db`).
- No hay un mecanismo nativo para que dos proyectos diferentes compartan contexto (ej: microservicios relacionados). Workaround aceptado: modelarlos como un único proyecto usando etiquetas como `#service-a` y `#service-b`.
- Los triggers de SQL agregan superficie de prueba (las migraciones tienen que validar que los triggers bloqueen inserts y updates inválidos).

## Alternativas Consideradas

- **`--all-projects` como un flag opt-in** — rechazado: introduce el modo cross-tenant en la superficie pública, y un flag bien intencionado en un script se convierte en una filtración permanente.
- **Base de datos por proyecto en `<repo>/.local-backlog.db`** — rechazado: rompe la premisa de "herramienta portátil, un `cargo install`, cero desorden en el repo"; los usuarios perderían el historial si olvidaran agregarlo al `.gitignore`.
- **Aplicar filtros solo en la capa de aplicación, sin triggers** — rechazado: un bug de consulta que olvide el `WHERE project_id` rompe la tenencia silenciosamente. Los triggers son la defensa en profundidad.

## Relacionados

- [ADR-0005 — Registro Global](0005-registry-global.md) define cómo se resuelve el tenant a partir del CWD.
