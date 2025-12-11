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
    main.rs             # WASM entry, event handlers, animation loop
    render.rs           # Canvas 2D rendering
    simulation.rs       # Solar wind, heliosphere updates
    star_data.rs        # UniverseDataManager - deterministic star rendering
    cca_projection.rs   # CelestialCamera - sun-centered spherical projection
    streaming.rs        # Data streaming utilities (Phase 2)
  index.html            # Entry point
  fonts/                # Webfonts (Just Sans)
```

## Development Phases

### Phase 1: Deterministic Star Rendering ✅
- **UniverseDataManager** (`star_data.rs`): Dataset-driven star rendering
  - Multi-wavelength bands (Gamma, X-Ray, UV, Optical, IR, Radio, CMB)
  - LOD adaptation based on frame time (auto-adjusts mag_limit)
  - Local StarDatabase integration (4000 stars/frame hard cap)
  - Performance monitoring (60-frame moving average)
- **CelestialCamera** (`cca_projection.rs`): Sun-centered spherical projection
  - Azimuth/elevation/scale parameterization
  - Target always at HCI origin (0,0,0)
  - Scale levels with magnitude limits
- **Mobile-First UI**: Touch-optimized controls
  - Band selector carousel
  - Dynamic filters (stars, constellations, grid)
  - Magnitude slider with real-time feedback
  - Device capability negotiation

### Phase 2: Server Architecture (Planned)
- **HEALPix Tile System** (`gen_tiles.rs`): Spherical tile generation
  - L4-8 hierarchical LOD
  - Bincode + zstd compression
  - Server-side tile storage
- **Streaming API** (`bin/server.rs`): Axum HTTP server
  - `/api/objects/tile` endpoint with filtering
  - WebSocket push updates
  - Bandwidth-aware quality adaptation
- **Client-Side Caching**: LRU tile cache
  - Progressive loading and prefetching
  - Offline capability

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
- **Star rendering** from UniverseDataManager
- Constellation overlay with deterministic edges
- HUD elements with performance metrics

### simulation.rs
- Integrates `dna::heliosphere_model`
- Solar wind particle spawning/aging
- Time acceleration (configurable)
- **UniverseDataManager** integration for star updates

### star_data.rs
- **UniverseDataManager**: Central star/constellation data manager
  - `Band` enum: 7 spectral wavelengths
  - `StarInstance`: Position, magnitude, color at time T
  - `update_view()`: Updates visible stars for camera/time/band
  - **LOD Adaptation**: Auto-adjusts magnitude limit based on frame time
  - **Performance**: 60-frame moving average, target 3-8ms/frame
- **Local Backend** (Phase 1): Uses `dna::world::stars::create_bright_stars()`
- **Server Backend** (Phase 2): HEALPix tile streaming

### cca_projection.rs
- **CelestialCamera**: Sun-centered spherical coordinate system
  - Camera always targets HCI origin (0,0,0)
  - Azimuth θ, elevation φ, distance r parameterization
  - `is_visible()`: Frustum culling for stars
  - `project_star()`: HCI → screen space projection
- **ScaleLevel**: Hierarchical scale with magnitude limits
  - Solar System (mag 0-2), Neighborhood (mag 3-4), etc.
  - Each level defines visible star magnitude cutoff

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

## Dependencies

### From DNA crate
```rust
use dna::world::stars::{Star, create_bright_stars};
use dna::spatial::{Vec3, project_to_screen};
use glam::DVec3;  // Double-precision 3D vectors for astronomy
```

### WASM bindings
```rust
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use js_sys::Date;
```

## Styling

- Primary font: Just Sans (loaded from `/fonts/`)
- Color scheme: Dark space theme with neon accents
- Responsive canvas scaling

## Common Tasks

### Adding a new spectral band
1. Add variant to `Band` enum in `star_data.rs`
2. Implement band-specific color/magnitude in `star_band_properties()`
3. Add UI button in `index.html` with `id="band-{name}"`
4. Update `band_buttons` array in `main.rs`

### Adding a new constellation
1. Look up HIP IDs of stars in constellation
2. Add `ConstellationEdge` entries in `update_constellations()` (`star_data.rs`)
3. Edges will render automatically in next frame

### Adjusting LOD performance targets
1. Modify `adjust_lod()` in `star_data.rs`
2. Current targets: 3-8ms/frame (120-330 FPS headroom)
3. Increase thresholds for mobile, decrease for desktop

### Adding a new HUD element
1. Create render function in `render.rs`
2. Position using canvas coordinates (0,0 = top-left)
3. Consider both UI modes
4. Add interactivity in `main.rs` if needed

## Testing

Visual testing via Playwright:
```bash
npx playwright test tests/helios.spec.ts
```

Manual testing checklist:
- [ ] Canvas resizes with window
- [ ] Touch events work on mobile (band selector, filters)
- [ ] Time controls respond
- [ ] Mode toggle switches UI
- [ ] Star rendering performs at 60fps
- [ ] Magnitude slider updates star count dynamically
- [ ] Band selector changes star colors
- [ ] LOD adaptation kicks in under load
- [ ] No WASM errors in console

## Performance Benchmarks

Target frame budget: 16.67ms (60 FPS)
- Star update: 3-8ms (auto-adaptive LOD)
- Rendering: 5-10ms (4000 stars maximum)
- Headroom: ~3ms for solar wind, UI, etc.

LOD Behavior:
- Frame time > 8ms → Increase mag_limit (show fewer stars)
- Frame time < 3ms → Decrease mag_limit (show more stars)
- Magnitude range: 0.0 (brightest) to 8.0 (faintest visible)
