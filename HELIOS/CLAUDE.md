# Helios - Solar System Visualization

Interactive heliosphere visualization using Rust/WASM and HTML5 Canvas.

## Build & Run

```bash
# Development (hot reload)
trunk serve helios/index.html --open

# Production build
trunk build --release helios/index.html

# Output in helios/dist/
```

## Architecture

```
helios/
  src/
    main.rs        # WASM entry, event handlers, animation loop
    render.rs      # Canvas 2D rendering
    simulation.rs  # Solar wind, heliosphere updates
    streaming.rs   # Data streaming utilities
  index.html       # Entry point
  fonts/           # Webfonts (Just Sans)
```

## Key Components

### main.rs
- `start()`: WASM entry point, sets up canvas and event listeners
- Animation loop using `requestAnimationFrame`
- Keyboard/mouse/touch event handlers
- UI state management (kid/researcher mode)

### render.rs
- `Renderer`: Holds canvas context, draws all layers
- Parker spiral visualization
- Termination shock boundary
- Solar wind particles
- Constellation overlay
- HUD elements

### simulation.rs
- Integrates `core::heliosphere_model`
- Solar wind particle spawning/aging
- Time acceleration (configurable)

## UI Modes

| Mode | Description |
|------|-------------|
| Kid Mode | Simplified UI, larger elements, tooltips |
| Researcher Mode | Full data, coordinate overlays, controls |

Toggle with `M` key or UI button.

## Controls

| Key | Action |
|-----|--------|
| Space | Pause/resume |
| +/- | Time scale |
| M | Toggle mode |
| C | Toggle constellations |
| G | Toggle grid |

## Dependencies from Core

```rust
use core::heliosphere_model::{HeliosphereModel, ParkerSpiral};
use core::solar_wind::{SolarWindParticle, SolarWindField};
use core::coordinates::{EclipticCoord, HeliographicCoord};
```

## Styling

- Primary font: Just Sans (loaded from `/fonts/`)
- Color scheme: Dark space theme with neon accents
- Responsive canvas scaling

## Common Tasks

### Adding a new celestial object
1. Add data struct (position, size, color)
2. Add render function in `render.rs`
3. Call from main render loop
4. Add toggle if optional

### Adding a new HUD element
1. Create render function in `render.rs`
2. Position using canvas coordinates (0,0 = top-left)
3. Consider both UI modes
4. Add interactivity in `main.rs` if needed

### Changing time scale behavior
1. Modify `simulation.rs` time update
2. Update HUD display format
3. Test at extreme values (fast forward, slow motion)

## Testing

Visual testing via Playwright:
```bash
npx playwright test tests/helios.spec.ts
```

Manual testing checklist:
- [ ] Canvas resizes with window
- [ ] Touch events work on mobile
- [ ] Time controls respond
- [ ] Mode toggle switches UI
- [ ] No WebGL errors in console
