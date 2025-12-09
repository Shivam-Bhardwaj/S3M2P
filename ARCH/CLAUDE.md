# ARCH - Architecture Explorer

Interactive dependency graph visualization for the antimony-labs monorepo.

## Build & Run

```bash
# Development (hot reload)
trunk serve ARCH/index.html --open

# Production build
trunk build --release ARCH/index.html

# Output in ARCH/dist/
```

## Architecture

```
ARCH/
  src/
    lib.rs           # WASM entry, canvas rendering, event handlers
    graph.rs         # DependencyGraph, CrateInfo, CrateLayer types
    workspace_data.json  # Pre-computed workspace data (compile-time embedded)
  index.html         # Entry point with CSS styling
```

## Key Components

### lib.rs
- `AppState`: Canvas context, view transforms (pan/zoom), node positions
- `layout_nodes()`: Arranges crates in concentric rings by layer
- `render()`: Draws grid, layer rings, edges, nodes, legend, info panel
- `setup_events()`: Mouse drag, wheel zoom, click selection

### graph.rs
- `CrateLayer`: Dna, Core, Project, Tool - architecture hierarchy
- `CrateInfo`: Name, path, layer, dependencies for each crate
- `DependencyGraph`: Collection of crates with lookup methods

## Visualization Layers

| Layer | Ring | Color | Description |
|-------|------|-------|-------------|
| DNA | Center | Blue (#3b82f6) | Core algorithms |
| CORE | Inner | Teal (#14b8a6) | Domain engines |
| PROJECT | Middle | Purple (#a855f7) | Applications |
| TOOL | Outer | Amber (#f59e0b) | Utilities |

## Controls

| Action | Effect |
|--------|--------|
| Scroll | Zoom in/out |
| Drag | Pan view |
| Click node | Select and show info panel |

## Updating Workspace Data

The `workspace_data.json` file is embedded at compile time. To regenerate:

```bash
# From workspace root, parse all Cargo.toml files
python3 -c "
import os, json, toml

workspace_root = '.'
crates = []

# Read workspace Cargo.toml
ws = toml.load('Cargo.toml')
members = ws.get('workspace', {}).get('members', [])

for member in members:
    cargo_path = os.path.join(member, 'Cargo.toml')
    if os.path.exists(cargo_path):
        data = toml.load(cargo_path)
        pkg = data.get('package', {})
        deps = list(data.get('dependencies', {}).keys())

        # Determine layer from path
        if member == 'DNA' or member.startswith('DNA/'):
            layer = 'Dna'
        elif member.startswith('CORE/'):
            layer = 'Core'
        elif member in ['WELCOME', 'HELIOS', 'BLOG', 'ARCH']:
            layer = 'Project'
        else:
            layer = 'Tool'

        crates.append({
            'name': pkg.get('name', member.split('/')[-1].lower()),
            'path': member,
            'layer': layer,
            'dependencies': [d for d in deps if d in [c['name'] for c in crates]]
        })

print(json.dumps({'crates': crates}, indent=2))
" > ARCH/src/workspace_data.json
```

## Common Tasks

### Adding a new crate to the visualization
1. Add to workspace Cargo.toml
2. Regenerate workspace_data.json
3. Rebuild ARCH

### Changing node appearance
1. Edit `draw_nodes()` in lib.rs
2. Adjust colors in layer match statement
3. Modify radius, glow, or label formatting

### Changing layout
1. Edit `layout_nodes()` ring radii
2. Adjust spacing algorithm
3. Consider force-directed layout for complex graphs

## Dependencies

- wasm-bindgen: WASM bindings
- web-sys: Canvas 2D API
- serde/serde_json: Data serialization
- toml: Cargo.toml parsing (for data generation)
- console_error_panic_hook: Better WASM error messages
