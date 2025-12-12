# VISIÓN — Autocrate (Antimony Labs) en S3M2P

**Languages:** [English](VISION.md) | [हिन्दी](VISION.hi.md) | [中文](VISION.zh.md) | [Español](VISION.es.md) | [العربية](VISION.ar.md)

## Qué estamos construyendo

**Autocrate** es un "compilador" guiado por estándares para productos físicos: convierte una intención estructurada en **artefactos de fabricación deterministas**.

En v1, la intención es una **crate specification** (dimensiones, peso, modo de envío, perfil de cumplimiento). Los artefactos son:

- **STEP assembly (NX-importable, inches)**: un único entregable MBD-first suficiente para fabricar.
- **BOM (CSV)**: qué comprar.
- **Cut List (CSV)**: qué cortar.
- **Viewer scene**: qué ver (la fosa defensiva de la plataforma).

El contrato es:

> `CrateSpec → CrateDesign → { STEP, BOM, Cut List, Viewer }`

## Por qué importa (MBD-first)

El futuro del CAD es la **Model-Based Definition (MBD)**: un único archivo STEP (estilo AP242) puede contener la estructura del ensamblaje y la información necesaria aguas abajo.

Autocrate adopta esta postura:
- STEP es un **output engine**, no el "model".
- El modelo canónico es nuestro grafo **CrateDesign**.
- STEP/BOM/Cut List se generan a partir del mismo grafo, por lo que se mantienen consistentes.

## La fosa defensiva: visualización

La mayoría de los sistemas tratan los archivos de intercambio CAD como fuente de verdad e intentan re-parsearlos para la visualización.

Nosotros hacemos lo contrario:
- **CrateDesign** es la verdad.
- Renderizamos **directamente desde CrateDesign**, sin parsear STEP.

Esto hace que la visualización sea:
- **Más rápida** (sin un pipeline pesado de importación CAD),
- **Más controlable** (piezas semánticas + metadatos),
- **Más defendible** (un grafo de diseño + motor de render construidos juntos).

## La ventaja de arquitectura de S3M2P (DNA → CORE → TOOLS)

S3M2P está intencionalmente estratificado para que la iteración asistida por IA sea más segura:

- **DNA**: algoritmos puros, física/matemática/estructuras de datos. Determinista, testeable, reutilizable.
- **CORE**: motores ergonómicos que exponen APIs estables para tools.
- **TOOLS**: apps orientadas al usuario (WASM) que renderizan y exportan.

Esta separación es la "capa de confianza" para la IA:
- La IA puede proponer cambios, pero las **DNA tests + deterministic outputs** detectan regresiones.
- Las mejoras se acumulan porque el core permanece limpio y reutilizable.

## Alcance de estándares (v1)

Modelamos los estándares como **parameterized profiles** (rules + limits), no como texto copiado.

- **ASTM D6039**: crate profile selection y rule-driven sizing (alcance v1).
- **ASTM D6199**: wood member class capturada como entradas de material/calidad.
- **ISPM 15**: export compliance metadata + required marking/decals como parts.

## Qué significa "done"

- NX importa el STEP generado como un **assembly** a la escala correcta en **inch** con nombres deterministas.
- Viewer muestra el mismo assembly y permite inspección de parts (IDs, category, metadata).
- BOM/Cut List coinciden con el design graph y se mantienen consistentes con STEP.

## Hoja de ruta (próximas cuñas)

Después de Autocrate Lite v1:

- **Richer PMI / MBD**: property sets, IDs y downstream-friendly naming conventions.
- **Rule profiles**: shipping severity knobs, más standards profiles, material availability constraints.
- **Catalog parts**: fasteners/connectors como parametric library items (visual + STEP semantics).
- **More products**: reutilizar DNA/CORE para otros "physical compilers" más allá de crates.
