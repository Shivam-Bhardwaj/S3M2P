# Update DNA Architecture

Perform architectural sanity checks on the DNA/CORE/Project layering and optionally reorganize code.

## Usage

```
/update-dna [mode]
```

**Arguments:**
- (default) - `--dry-run`: Report violations without making changes
- `--execute` - Apply recommended changes (move files, update imports)

## Architecture Principles

### 3-Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│  PROJECTS (UI/Application Layer)                        │
│  WELCOME, HELIOS, BLOG, TOOLS/*, LEARN/*, SIMULATION/*  │
└────────────────────────┬────────────────────────────────┘
                         │ depends on
┌────────────────────────▼────────────────────────────────┐
│  CORE (Domain-Specific Engines)                         │
│  */CORE/*_ENGINE - Thin wrappers over DNA               │
└────────────────────────┬────────────────────────────────┘
                         │ depends on
┌────────────────────────▼────────────────────────────────┐
│  DNA (Foundation Layer)                                 │
│  Pure algorithms, zero domain-specific code             │
│  Zero workspace dependencies                            │
└─────────────────────────────────────────────────────────┘
```

### What Belongs Where

**DNA (Foundation)** - Pure, reusable algorithms:
- `physics/` - Mechanics, fields, solvers, electromagnetics
- `math/` - Matrix operations, random utilities
- `data/` - Arena, spatial grid, mesh, graph structures
- `world/` - Coordinate systems, topology, units
- `cad/` - B-Rep geometry kernel
- `export/` - PDF, Gerber format writers
- `pll/` - Pure PLL circuit algorithms
- `autocrate/` - Crate geometry algorithms (wrapped by CORE)

**CORE (Domain Engines)** - Domain-specific wrappers:
- `SIMULATION/CORE/SIMULATION_ENGINE` - Boid/particle simulation
- `SIMULATION/CORE/WAVE_ENGINE` - Wave physics (Chladni, FFT)
- `SIMULATION/CORE/SPICE_ENGINE` - Circuit simulation
- `TOOLS/CORE/PLL_ENGINE` - PLL design automation
- `TOOLS/CORE/AUTOCRATE_ENGINE` - Crate design automation
- `TOOLS/CORE/CAD_ENGINE` - Solid modeling
- `TOOLS/CORE/EXPORT_ENGINE` - File export pipeline

**Projects** - UI/Application code:
- Domain-specific rendering, event handling, WASM bindings

---

## Instructions

### Phase 1: Scan DNA for Domain Code

Scan `DNA/src/` for modules that contain domain-specific code:

```bash
# List all modules in DNA/src/
ls -la DNA/src/
```

**Domain Keywords to Flag:**
| Pattern | Domain | Target Location |
|---------|--------|-----------------|
| `heliosphere*` | HELIOS | `HELIOS/src/` |
| `solar_wind*` | HELIOS | `HELIOS/src/` |
| `spatial.rs` (astronomical) | HELIOS | `HELIOS/src/` |

**Foundation Keywords (Keep in DNA):**
| Pattern | Reason |
|---------|--------|
| `physics/` | Pure physics algorithms |
| `math/`, `data/`, `world/` | Generic primitives |
| `cad/`, `export/` | Foundation utilities |
| `pll/` | Pure circuit algorithms |
| `autocrate/` | Already wrapped by CORE |
| `sim/` | Generic simulation primitives |
| `color.rs`, `zones.rs` | Shared utilities |

### Phase 2: Check CORE Dependencies

For each CORE engine, verify it only depends on DNA:

```bash
# Check each CORE engine's Cargo.toml
cat SIMULATION/CORE/*/Cargo.toml | grep -E "^[a-z].*= "
cat TOOLS/CORE/*/Cargo.toml | grep -E "^[a-z].*= "
```

**Violations:**
- CORE engine depending on another CORE engine
- CORE engine with circular dependencies

### Phase 3: Check Orphan Engines

Find CORE engines with no projects using them:

```bash
# Search for engine usage in project Cargo.toml files
grep -r "simulation-engine\|wave-engine\|spice-engine\|pll-engine\|autocrate-engine\|cad-engine\|export-engine" \
  --include="Cargo.toml" \
  WELCOME/ HELIOS/ BLOG/ ARCH/ TOOLS/ LEARN/ SIMULATION/
```

### Phase 4: Generate Report

Output a markdown report:

```markdown
## DNA Architecture Report

Generated: {timestamp}

### Summary
| Check | Status | Count |
|-------|--------|-------|
| Domain code in DNA | {status} | {count} |
| CORE-to-CORE deps | {status} | {count} |
| Orphan engines | {status} | {count} |

### Domain Code Found in DNA

| Module | Classification | Recommendation |
|--------|----------------|----------------|
| heliosphere.rs | HELIOS domain | Move to HELIOS/src/ |
| heliosphere_model.rs | HELIOS domain | Move to HELIOS/src/ |
| solar_wind.rs | HELIOS domain | Move to HELIOS/src/ |
| spatial.rs | HELIOS domain | Move to HELIOS/src/ |

### CORE Engine Status

| Engine | Used By | Status |
|--------|---------|--------|
| simulation-engine | WELCOME | Active |
| wave-engine | CHLADNI | Active |
| spice-engine | (standalone) | Active |
| pll-engine | PLL | Active |
| autocrate-engine | AUTOCRATE | Active |
| cad-engine | (none) | Orphan |
| export-engine | (none) | Orphan |

### Recommended Actions

1. **Move HELIOS domain code** (if --execute):
   - `git mv DNA/src/heliosphere.rs HELIOS/src/`
   - `git mv DNA/src/heliosphere_model.rs HELIOS/src/`
   - `git mv DNA/src/solar_wind.rs HELIOS/src/`
   - `git mv DNA/src/spatial.rs HELIOS/src/`
   - Update `DNA/src/lib.rs` to remove exports
   - Update `HELIOS/src/lib.rs` to add modules
```

### Phase 5: Execute Changes (if --execute)

If `$ARGUMENTS` contains `--execute`:

1. **Backup**: Ensure git working tree is clean
   ```bash
   git status --porcelain
   ```

2. **Move files** for each recommended move:
   ```bash
   git mv {source} {destination}
   ```

3. **Update DNA/src/lib.rs**:
   - Remove `pub mod heliosphere;`
   - Remove `pub mod heliosphere_model;`
   - Remove `pub mod solar_wind;`
   - Remove `pub mod spatial;`
   - Remove corresponding `pub use` statements

4. **Update HELIOS/src/lib.rs** (or main.rs):
   - Add `mod heliosphere;`
   - Add `mod heliosphere_model;`
   - Add `mod solar_wind;`
   - Add `mod spatial;`

5. **Update imports** in HELIOS source files:
   - Find: `use dna::heliosphere`
   - Replace: `use crate::heliosphere`

6. **Validate**:
   ```bash
   cargo check --workspace
   ```

7. **Update ARCH data** (on success):
   ```bash
   node SCRIPTS/generate_workspace_data.js
   node SCRIPTS/scan_docs.js
   ```

8. **Report completion**:
   ```markdown
   ## Execution Complete

   - Files moved: {count}
   - Imports updated: {count}
   - Cargo check: PASSED
   - ARCH data: Updated
   ```

### Phase 6: Handle Errors

If `cargo check` fails after execution:

```markdown
## Execution Failed

Cargo check failed with errors:
{error_output}

### Manual Recovery
```bash
git checkout -- DNA/src/lib.rs
git checkout -- HELIOS/src/
git status
```
```

---

## Classification Rules

### Module Classification Function

For each file in `DNA/src/`:

1. **HELIOS Domain** (should move to HELIOS/src/):
   - `heliosphere.rs` - Heliospheric boundary physics
   - `heliosphere_model.rs` - Parker spiral, termination shock
   - `solar_wind.rs` - Solar wind particle modeling
   - `spatial.rs` - Astronomical spatial transforms

2. **Foundation** (stays in DNA):
   - `physics/` - All physics subdirectories
   - `math/` - Matrix, random
   - `data/` - Arena, spatial grid, mesh
   - `world/` - Coordinates, topology, units
   - `cad/` - B-Rep kernel
   - `export/` - Format writers
   - `pll/` - Circuit algorithms
   - `autocrate/` - Crate geometry
   - `sim/` - Boid arena, state machine
   - `color.rs`, `zones.rs`, `interaction.rs`, etc.

3. **Already Wrapped** (no action needed):
   - Modules already covered by CORE engines
   - `sim/chladni.rs` → WAVE_ENGINE
   - `wave_field/` → WAVE_ENGINE
   - `autocrate/` → AUTOCRATE_ENGINE

---

## Integration

After successful execution, this command automatically triggers:
- `/update-arch-data` workflow (via scripts)

This ensures:
- `ARCH/src/workspace_data.json` reflects new crate structure
- `ARCH/src/db.json` has updated documentation paths
