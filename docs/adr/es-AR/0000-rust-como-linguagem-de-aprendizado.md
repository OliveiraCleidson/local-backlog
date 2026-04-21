# ADR-0000 — Rust como lenguaje de implementación con foco estratégico de aprendizaje

- **Estado:** Aceptado
- **Fecha:** 2026-04-20
- **Autor:** Cleidson Oliveira

## Contexto

`local-backlog` nace con un doble propósito explícito:

1. **Herramienta real:** una CLI que el autor va a usar en todos sus repos personales para gestionar el backlog, portarla entre máquinas vía `cargo install` y consumirla con agentes de IA.
2. **Vehículo de aprendizaje:** Rust es una apuesta de largo plazo en la trayectoria pública del autor (ancla Staff/Distinguished Engineer, 2–3 años), junto a Go/AWS/IA. Un alcance acotado, bien delimitado y en un dominio conocido es el escenario ideal para aprender el lenguaje sin la presión de un deadline externo.

Las alternativas evaluadas fueron Go (binario único, ya familiar, velocidad de iteración) y Python (prototipo rápido, distribución inferior). Go entregaría antes; Python sería el camino de menor fricción. Ninguno ayuda a achicar la brecha de conocimiento en Rust que se busca cubrir.

## Decisión

Rust es el lenguaje de implementación de `local-backlog`. La decisión tiene tres implicancias operativas:

1. **El ritmo prioriza la comprensión por sobre la entrega.** No hay un deadline externo. Si una feature te exige entender lifetimes, traits genéricas o `async` a fondo, el tiempo que le metas es parte de la inversión del proyecto — no un desvío.
2. **Las elecciones de crates pueden privilegiar el valor de aprendizaje** cuando hay dos opciones equivalentes. Ejemplos: preferir `figment` sobre `config-rs` por cómo expone los conceptos de providers; preferir el combo `thiserror` + `miette` sobre `anyhow` para aprender a diseñar errores tipados.
3. **El código generado o asistido por IA pasa por una revisión humana explicativa.** El objetivo no es solo que ande — el autor tiene que entender cada bloque que no sea trivial. El code review (humano o adversarial vía Codex) tiene que explicar los idioms cuando no queden claros, no solamente dar el OK.

Esta decisión es foundational — todos los demás ADRs (0001–NNNN) asumen Rust como dado.

## Consecuencias

**Positivas:**
- Un proyecto chico se convierte en un campo de entrenamiento controlado: `tasks` atómica + satélites cubre `rusqlite`, migrations, triggers, serde, manejo de errores tipados, parsing de CLI — los idioms centrales del ecosistema.
- El retorno del esfuerzo es compuesto: una herramienta personal útil + profundidad en Rust + una pieza de portfolio público alineada con el ancla declarada.
- Un alcance bien cerrado evita el anti-patrón de "aprender Rust con fuego real" (en una producción crítica).

**Negativas:**
- Velocidad inicial menor que con Go/Python. Mitigación: no hay deadline; el costo se acepta por diseño.
- La curva de aprendizaje puede llevarte a tomar decisiones que un ingeniero de Rust senior revisaría después. Mitigación: los ADRs 0001–NNNN fijan los límites arquitectónicos (tenancy, schema, contrato de salida); dentro de esos límites, iterar y reescribir es bienvenido.
- **Over-engineering pedagógico aceptable** dentro de los límites: implementar un trait propio para `Output` cuando un `enum` alcanzaría, si eso sirve para aprender diseño de traits. No es aceptable fuera de los límites: romper la tenancy o el contrato de salida por una curiosidad sintáctica.

## Alternativas Consideradas

- **Go** — rechazado: reutilizaría conocimiento existente sin contribuir a la brecha estratégica declarada. Entregaría la herramienta más rápido, pero el costo de oportunidad es el aprendizaje que no ocurriría.
- **Python** — rechazado: prototipo rápido, pero la distribución (PyPI/pipx) es inferior a `cargo install` para el modelo de uso pretendido; y cero ganancia estratégica de lenguaje nuevo.
- **TypeScript/Node** — rechazado por los mismos motivos que Python, con el agravante de un runtime externo obligatorio.

## Relacionados

- [ADR-0001 — Tenencia estricta](0001-tenancy-estrita-por-projeto.md) — primera decisión arquitectónica que toma Rust como dado.
- [ADR-0003 — Migrations inline](0003-migrations-inline.md) — depende de features del ecosistema Rust (`include_str!`, `rusqlite_migration`).
