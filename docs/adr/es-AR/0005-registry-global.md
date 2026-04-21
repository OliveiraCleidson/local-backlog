# ADR-0005 — Identificación de Proyecto mediante Registro Global

- **Estado:** Aceptado
- **Fecha:** 2026-04-20

## Contexto

La tenencia estricta ([ADR-0001](0001-tenancy-estrita-por-projeto.md)) requiere resolver "¿qué proyecto es este?" en cada invocación de la CLI. Se consideraron tres estrategias:

1. **Archivo `.local-backlog` en el repositorio** con un `project_id`. Versionable en git; ensucia los repositorios; si no se confirma (commit), desaparece.
2. **Hash de la ruta del repositorio git** (`git rev-parse --show-toplevel`). Cero configuración; un clon en otra máquina se convierte silenciosamente en un "proyecto diferente"; no funciona fuera de git.
3. **Registro Global** en `~/.local-backlog/registry.toml` mapeando `ruta_absoluta → project_id`. Cero desorden en el repositorio; requiere `backlog relink` al mover carpetas; fácil de listar proyectos.

## Decisión

Usar la opción 3: un único registro global.

### Estructura

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

El registro es un espejo canónico de la tabla `projects` de SQLite — la fuente de verdad es la base de datos; el archivo TOML existe para la inspección humana y como un caché de búsqueda rápida.

### Sincronización

Los cambios en los metadatos del proyecto se aplican primero a SQLite y luego se persisten en `registry.toml` mediante la reescritura atómica del archivo:

- `backlog init` inserta el proyecto en `projects` y escribe la entrada correspondiente en el registro.
- `backlog projects relink <id|name> --path <nuevo>` actualiza `projects.root_path` y reescribe el registro.
- `backlog projects archive <id|name>` actualiza `projects.archived_at` y reescribe el registro.
- `backlog doctor` compara SQLite y `registry.toml`, informando entradas faltantes, rutas obsoletas, rutas duplicadas e IDs que no coinciden.

### Resolución

Para cada comando:

1. Canonizar el CWD (resolver enlaces simbólicos, normalizar).
2. Subir por el árbol de directorios buscando una coincidencia en `root_path`.
3. Si se encuentra → usar el `project_id` correspondiente.
4. Si no se encuentra → error mediante `miette` sugiriendo `backlog init` o `backlog projects relink`.

### Comandos Meta (El Único Espacio de Nombres Cross-Tenant)

- `backlog projects list` — muestra todos los proyectos registrados (id, nombre, ruta, conteo de tareas).
- `backlog projects show <id|name>` — metadatos de un proyecto.
- `backlog projects relink <id|name> --path <nuevo>` — actualiza el `root_path` cuando el repositorio cambia de carpeta.
- `backlog projects archive <id|name>` — marca un proyecto como archivado (no aparece en la `list`); los datos se preservan.

## Consecuencias

**Positivas:**
- Sin archivos `local-backlog` dentro de los repositorios — no ensucia el `git status`.
- Funciona fuera de git (proyectos sin control de versiones).
- `backlog projects list` proporciona un inventario global instantáneo.
- Cambios de carpeta → un simple `relink` lo soluciona; sin pérdida de datos.

**Negativas:**
- Perder `~/.local-backlog/` rompe los enlaces (la base de datos + el registro viven juntos). Mitigación: la carpeta entera es trivial de respaldar; el usuario puede versionarla en sus dotfiles si lo desea.
- Dos checkouts diferentes del mismo repositorio en rutas diferentes se convierten en proyectos distintos (esto puede ser intencionado o un error). Mitigación: `backlog init` detecta repositorios Git y pregunta si es un nuevo proyecto o un duplicado; `doctor` marca rutas con el mismo `origin` mapeadas a diferentes IDs.
- La portabilidad entre máquinas requiere sincronizar `~/.local-backlog/` (o aceptar estados independientes por máquina). Aceptado — la herramienta es single-machine por diseño.

## Alternativas Consideradas

- **`.local-backlog` en el repositorio** — rechazado: ensucia los repositorios del usuario, requiere disciplina en el `.gitignore` o un commit con un ID acoplado.
- **Hash de `git remote get-url origin`** — rechazado: falla en repositorios sin un origen, en forks y en repositorios no-git.
- **Nombre del directorio como clave** — rechazado: dos repositorios llamados `api/` colisionarían.

## Relacionados

- [ADR-0001 — Tenencia](0001-tenancy-estrita-por-projeto.md) — el registro es el mecanismo que entrega el tenant-id a partir del CWD.
