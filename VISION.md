# VISION — Autocrate (Antimony Labs) in S3M2P

## What we are building

**Autocrate** is a standards-driven “compiler” for physical products: it turns structured intent into **deterministic manufacturing artifacts**.

In v1, the intent is a **crate specification** (dimensions, weight, shipping mode, compliance profile). The artifacts are:

- **STEP assembly (NX-importable, inches)**: a single MBD-first deliverable sufficient for manufacturing.
- **BOM (CSV)**: what to buy.
- **Cut List (CSV)**: what to cut.
- **Viewer scene**: what to see (the platform moat).

The contract is:

> `CrateSpec → CrateDesign → { STEP, BOM, Cut List, Viewer }`

## Why this matters (MBD-first)

The future of CAD is **Model-Based Definition (MBD)**: a single STEP file (AP242-style) can carry the assembly structure and the information needed downstream.

Autocrate adopts this stance:
- STEP is an **output engine**, not the “model.”
- The canonical model is our **CrateDesign** graph.
- STEP/BOM/Cut List are generated from the same graph, so they stay consistent.

## The moat: visualization

Most systems treat CAD exchange files as the source of truth and try to re-parse them for visualization.

We do the opposite:
- **CrateDesign** is the truth.
- We render **directly from CrateDesign**, without parsing STEP.

This makes visualization:
- **Faster** (no heavy CAD import pipeline),
- **More controllable** (semantic parts + metadata),
- **More defensible** (a design graph + rendering engine built together).

## The S3M2P architecture advantage (DNA → CORE → TOOLS)

S3M2P is intentionally layered to make AI-assisted iteration safer:

- **DNA**: pure algorithms, physics/math/data structures. Deterministic, testable, reusable.
- **CORE**: ergonomic engines that expose stable APIs for tools.
- **TOOLS**: user-facing apps (WASM) that render and export.

This separation is the “trust layer” for AI:
- AI can propose changes, but **DNA tests + deterministic outputs** catch regressions.
- Improvements compound because the core remains clean and reusable.

## Standards scope (v1)

We model standards as **parameterized profiles** (rules + limits), not copied text.

- **ASTM D6039**: crate profile selection and rule-driven sizing (v1 scope).
- **ASTM D6199**: wood member class captured as material/quality inputs.
- **ISPM 15**: export compliance metadata + required marking/decals as parts.

## What “done” looks like

- NX imports the generated STEP as an **assembly** at correct **inch scale** with deterministic names.
- Viewer shows the same assembly and supports part inspection (IDs, category, metadata).
- BOM/Cut List match the design graph and remain consistent with STEP.

## Roadmap (next wedges)

After Autocrate Lite v1:

- **Richer PMI / MBD**: property sets, IDs, and downstream-friendly naming conventions.
- **Rule profiles**: shipping severity knobs, more standards profiles, material availability constraints.
- **Catalog parts**: fasteners/connectors as parametric library items (visual + STEP semantics).
- **More products**: reuse DNA/CORE for other “physical compilers” beyond crates.


