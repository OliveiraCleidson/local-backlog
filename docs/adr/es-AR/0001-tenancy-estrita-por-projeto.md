# ADR-0001 — Tenencia estricta por proyecto

- **Estado:** Aceptado
- **Fecha:** 2026-04-20
- **Autor:** Cleidson Oliveira

## Contexto

`local-backlog` mantiene una única base de datos SQLite en `~/.local-backlog/backlog.db` agregando N proyectos del usuario. Dos enfoques eran posibles:

1. **Base de datos compartida con filtros opcionales** — los comandos por defecto muestran el proyecto actual, pero flags como `--all-projects` permiten una vista agregada.
2. **Tenencia estricta** — el proyecto inferido a partir del CWD (Directorio de Trabajo Actual) es el único alcance visible en cada operación de datos. No existe una superficie que agregue tareas/etiquetas/enlaces entre proyectos.

La Opción 1 parece pragmática pero introduce clases de bugs persistentes: `backlog list` en un repo mostrando tareas de otro, etiquetas colisionando entre proyectos, enlaces accidentales entre tareas no relacionadas e IA recibiendo contexto filtrado durante la exportación.

## Decisión

Adoptar tenencia estricta basada en proyecto:

- Cada consulta de tarea, etiqueta, atributo, enlace y evento incluye `project_id = :current` inferido vía CWD → registro.
- `tags.(project_id, name)` es único; `#auth` en dos proyectos diferentes no colisiona ni comparte un registro.
- Las relaciones padre/hijo, tarea/etiqueta y enlaces tarea/tarea deben permanecer dentro del mismo `project_id` — reforzado por triggers SQL tanto en la inserción como en la actualización.
- **No existe el flag `--all-projects`** en comandos de datos (`list`, `show`, `export`, etc.).
- La única superficie cross-tenant es el espacio de nombres meta `backlog projects ...` (list, show, archive, relink). Este espacio de nombres nunca expone contenido de tarea/etiqueta — solo metadatos del registro.
- `backlog doctor` verifica inconsistencias (tareas huérfanas, padres/etiquetas/enlaces cross-project) como parte del control de salud.

## Consecuencias

**Positivas:**
- Imposible filtrar datos de un proyecto a otro debido a errores de flag.
- Las etiquetas tienen un espacio de nombres natural por tenant — reutilización trivial de nombres (ej: `#bug`, `#auth`) sin configuración extra.
- La exportación de contexto para IA es segura por diseño: el JSON/Markdown emitido contiene solo el tenant actual.
- El modelo mental se alinea con Git: cada repositorio es su propio universo.

**Negativas:**
- No hay una vista agregada de "todo lo que tengo pendiente en el mundo". Mitigación: `backlog projects list` muestra contadores por proyecto; los tableros cross-tenant están fuera de alcance (los usuarios pueden usar SQL directamente en el `.db` si necesitan un informe excepcional).
- No hay un mecanismo nativo para dos proyectos diferentes que quieran compartir contexto (ej: microservicios relacionados). Workaround aceptado: modelarlos como un único proyecto usando las etiquetas `#service-a` y `#service-b`.
- Los triggers SQL agregan superficie de prueba (las migraciones deben validar que los triggers bloqueen inserciones y actualizaciones inválidas).

## Alternativas Consideradas

- **`--all-projects` como un flag opt-in** — rechazado: introduce el modo cross-tenant en la superficie pública, y un flag bien intencionado en un script se convierte en una filtración permanente.
- **Base de datos por proyecto en `<repo>/.local-backlog.db`** — rechazado: rompe la premisa de "herramienta portátil, un `cargo install`, cero desorden en el repo"; los usuarios perderían el historial si olvidaran agregarlo al `.gitignore`.
- **Aplicar filtros solo en la capa de aplicación, sin triggers** — rechazado: un bug de consulta que olvide el `WHERE project_id` rompe la tenencia silenciosamente. Los triggers son la defensa en profundidad.

## Relacionados

- [ADR-0005 — Registro Global](0005-registry-global.md) define cómo se resuelve el tenant a partir del CWD.
