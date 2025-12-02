# AutoCrate - ASTM Crate Generator

Parametric shipping crate design tool that generates crate specifications from product dimensions.

## Build & Run

```bash
trunk serve autocrate/index.html --open
trunk build --release autocrate/index.html
```

## Architecture

```
autocrate/
  src/
    lib.rs         # WASM entry, types
    constants.rs   # ASTM standards, lumber dimensions
    geometry.rs    # 3D geometry types
    calculator.rs  # Structural calculations
  index.html       # Entry point
  style.css        # Styles
```

## Core Types

### CrateSpec
Input specification:
- Product dimensions (L x W x H)
- Weight (lbs)
- Clearances (side, end, top)
- Lumber selections (skid, floorboard, cleat sizes)

### CrateGeometry
Generated output:
- Overall dimensions
- Skid positions
- Floorboard layout
- Panel geometries
- Cleat placements

## Constants (from ASTM)

- Lumber nominal vs actual dimensions
- Plywood sheet sizes (48" x 96")
- Forklift clearance requirements
- Fastener spacing rules

## Implementation Status

This is a Rust scaffold ported from TypeScript. Full implementation via issues:
- [x] Constants/standards ported
- [x] Geometry types defined
- [x] Basic calculator structure
- [ ] Full structural calculations
- [ ] Cleat placement algorithm
- [ ] Klimp/fastener placement
- [ ] Panel stop positioning
- [ ] 3D Canvas rendering
- [ ] Input UI
- [ ] Export (STEP/NX expressions)

## Original Source

TypeScript: https://github.com/Shivam-Bhardwaj/AutoCrate

Key files to port:
- `src/lib/nx-generator.ts` → `calculator.rs`
- `src/lib/crate-constants.ts` → `constants.rs` (done)
- `src/lib/cleat-calculator.ts` → TBD
- `src/lib/klimp-calculator.ts` → TBD
- `src/components/CrateVisualizer.tsx` → `render.rs`
