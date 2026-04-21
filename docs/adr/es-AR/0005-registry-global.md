# ADR-0005 — Identificación de Proyecto mediante Registro Global

- **Estado:** Aceptado
- **Fecha:** 2026-04-20

## Contexto

La tenencia estricta ([ADR-0001](0001-tenancy-estrita-por-projeto.md)) exige resolver "¿qué proyecto es este?" cada vez que llames a la CLI. Se evaluaron tres estrategias:

1. **Archivo `.local-backlog` en el repositorio** con un `project_id`. Lo podés versionar en git, pero te ensucia los repositorios y si te olvidás de commitearlo, desaparece.
2. **Hash de la ruta del repositorio git** (`git rev-parse --show-toplevel`). Cero configuración, pero un clon en otra máquina se convierte silenciosamente en un "proyecto diferente"; además, no anda fuera de git.
3. **Registro Global** en `~/.local-backlog/registry.toml` que mapea `ruta_absoluta → project_id`. Cero desorden en el repositorio; requiere un `backlog relink` si movés las carpetas, pero es fácil de listar los proyectos.

## Decisión

Usar la opción 3: un solo registro global.

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

El registro es un reflejo de la tabla `projects` de SQLite — la posta está en la base de datos; el archivo TOML queda para que lo podamos ver nosotros y como un caché de búsqueda rápida.

### Sincronización

Los cambios en los metadatos del proyecto se aplican primero a SQLite y luego se persisten en `registry.toml` mediante la reescritura atómica del archivo:

- `backlog init` inserta el proyecto en `projects` y escribe la entrada correspondiente en el registro.
- `backlog projects relink <id|name> --path <nuevo>` actualiza `projects.root_path` y reescribe el registro.
- `backlog projects archive <id|name>` actualiza `projects.archived_at` y reescribe el registro.
- `backlog doctor` compara SQLite y `registry.toml`, informando entradas faltantes, rutas obsoletas, rutas duplicadas e IDs que no coinciden.

### Resolución

Para cada comando:

1. Canonizar el CWD (resolver symlinks, normalizar).
2. Subir por el árbol de carpetas buscando un `root_path` que coincida.
3. Si lo encuentra → usa el `project_id` que corresponde.
4. Si no lo encuentra → tira un error con `miette` y te sugiere hacer `backlog init` o `backlog projects relink`.

### Comandos Meta (El Único Espacio de Nombres Cross-Tenant)

- `backlog projects list` — muestra todos los proyectos registrados (id, nombre, ruta, cantidad de tareas).
- `backlog projects show <id|name>` — metadatos de un proyecto.
- `backlog projects relink <id|name> --path <nuevo>` — actualiza el `root_path` cuando el repositorio cambia de carpeta.
- `backlog projects archive <id|name>` — marca un proyecto como archivado (para que no aparezca en la `list`); los datos no se borran.

## Consecuencias

**Positivas:**
- No tenés archivos de `local-backlog` dentro de los repos — no te ensucia el `git status`.
- Funciona fuera de git (proyectos que no tengan control de versiones).
- `backlog projects list` te da un inventario global al toque.
- Si movés una carpeta, con un simple `relink` lo solucionás sin perder datos.

**Negativas:**
- Si perdés `~/.local-backlog/`, se te rompen los links (la base de datos y el registro van de la mano). Mitigación: la carpeta es una pavada de backupear y la podés versionar en tus dotfiles si querés.
- Si tenés dos checkouts diferentes del mismo repo en rutas distintas, se ven como proyectos distintos (esto puede ser lo que querés o un error). Mitigación: `backlog init` detecta si es un repo Git y te pregunta si es un proyecto nuevo o un duplicado; `doctor` te marca las rutas con el mismo `origin` mapeadas a diferentes IDs.
- La portabilidad entre máquinas exige sincronizar `~/.local-backlog/` (o aceptar que los estados son independientes por máquina). Aceptado — la herramienta está pensada para una sola máquina por diseño.

## Alternativas Consideradas

- **`.local-backlog` en el repositorio** — rechazado: ensucia los repositorios del usuario, requiere disciplina en el `.gitignore` o un commit con un ID acoplado.
- **Hash de `git remote get-url origin`** — rechazado: falla en repositorios sin un origen, en forks y en repositorios no-git.
- **Nombre del directorio como clave** — rechazado: dos repositorios llamados `api/` colisionarían.

## Relacionados

- [ADR-0001 — Tenencia](0001-tenancy-estrita-por-projeto.md) — el registro es el mecanismo que entrega el tenant-id a partir del CWD.
