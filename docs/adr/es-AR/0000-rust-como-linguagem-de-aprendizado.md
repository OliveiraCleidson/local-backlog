# ADR-0000 — Rust como lenguaje de implementación con foco estratégico de aprendizaje

- **Estado:** Aceptado
- **Fecha:** 2026-04-20
- **Autor:** Cleidson Oliveira

## Contexto

`local-backlog` nace con un doble propósito explícito:

1. **Herramienta real:** una CLI que el autor va a usar en todos sus repositorios personales para gestionar el backlog, portarla entre máquinas vía `cargo install` y consumirla con agentes de IA.
2. **Vehículo de aprendizaje:** Rust es una apuesta de largo plazo en la trayectoria pública del autor (ancla Staff/Distinguished Engineer, 2–3 años), junto a Go/AWS/IA. Un alcance chico, bien delimitado y en un dominio conocido es el ambiente ideal para aprender el lenguaje sin presión de deadline externo.

Las alternativas evaluadas fueron Go (binario único, ya familiar, velocidad de iteración) y Python (prototipo rápido, distribución inferior). Go entregaría antes; Python sería el camino de menor fricción. Ninguno contribuye a la brecha declarada de profundidad en Rust.

## Decisión

Rust es el lenguaje de implementación de `local-backlog`. La decisión tiene tres implicancias operativas:

1. **El ritmo favorece la comprensión sobre la entrega.** No hay deadline externo. Si una feature exige entender lifetimes, traits genéricas o `async` con detalle, el tiempo gastado es parte del retorno del proyecto — no un desvío.
2. **Las elecciones de crate pueden privilegiar el valor de aprendizaje** cuando dos opciones son equivalentes en capacidad. Ejemplos: preferir `figment` sobre `config-rs` por exponer conceptos de providers; preferir el híbrido `thiserror`+`miette` sobre `anyhow` por enseñar el diseño de errores tipados.
3. **El código generado o asistido por IA pasa por revisión humana explicativa.** El objetivo no es solo que funcione — el autor tiene que entender cada bloque no trivial. El code review (humano o adversarial vía Codex) debe explicar los idioms cuando sean opacos, no solo aprobar.

Esta decisión es foundational — todos los demás ADRs (0001–NNNN) asumen Rust como dado.

## Consecuencias

**Positivas:**
- Un proyecto chico se convierte en campo de entrenamiento controlado: `tasks` atómica + satélites cubre `rusqlite`, migrations, triggers, serde, manejo de errores tipados, parsing de CLI — los idioms centrales del ecosistema.
- El retorno del esfuerzo es compuesto: herramienta personal útil + profundidad en Rust + pieza de portfolio público alineada con el ancla declarada.
- Un alcance cerrado evita el anti-patrón "aprender Rust en producción crítica".

**Negativas:**
- Velocidad inicial menor que Go/Python. Mitigación: sin deadline; el costo se acepta por diseño.
- La curva de aprendizaje puede producir decisiones que un ingeniero Rust senior revisaría después. Mitigación: los ADRs 0001–NNNN fijan los límites arquitectónicos (tenancy, schema, contrato de salida); dentro de esos límites, iterar y reescribir es bienvenido.
- **Over-engineering pedagógico aceptable** dentro de los límites: implementar una trait propia para `Output` cuando un `enum` alcanzaría, si eso enseña diseño de traits. No aceptable fuera de los límites: romper la tenancy o el contrato de salida por curiosidad sintáctica.

## Alternativas Consideradas

- **Go** — rechazado: reutilizaría conocimiento existente sin contribuir a la brecha estratégica declarada. Entregaría la herramienta más rápido, pero el costo de oportunidad es el aprendizaje que no ocurriría.
- **Python** — rechazado: prototipo rápido, pero la distribución (PyPI/pipx) es inferior a `cargo install` para el modelo de uso pretendido; y cero ganancia estratégica de lenguaje nuevo.
- **TypeScript/Node** — rechazado por los mismos motivos que Python, con el agravante de un runtime externo obligatorio.

## Relacionados

- [ADR-0001 — Tenancy estricta](0001-tenancy-estrita-por-projeto.md) — primera decisión arquitectónica que toma Rust como dado.
- [ADR-0003 — Migrations inline](0003-migrations-inline.md) — depende de features del ecosistema Rust (`include_str!`, `rusqlite_migration`).
