# WELCOME - antimony-labs Landing Page

Interactive landing page constellation for antimony-labs (SbL) - "too.foo" domain.
Displays project navigation with particle background simulation.

## Build & Run

```bash
# Development (hot reload)
trunk serve WELCOME/index.html --open

# Production build
trunk build --release WELCOME/index.html

# Output in WELCOME/dist/
```

## Architecture

```
WELCOME/
  src/
    main.rs      # WASM entry, particle sim, event loop
  index.html     # Entry point with constellation layout
  assets/        # Project icons and static assets
```

## Purpose

This is the **landing page** for antimony-labs, not a simulation project.
- Domain: too.foo (the name has no meaning, just a URL)
- Project: antimony-labs (SbL)
- Purpose: Navigation hub to all projects

## Visual Components

### Constellation Layout
- Circular arrangement of project icons
- Central "Antimony" logo
- Hover effects and animations
- Responsive design (mobile-first)

### Background Simulation
- Particle system using DNA/sim
- Interactive boid behavior
- Creates dynamic atmosphere
- Performance-optimized for landing page

### Telemetry Bar
- Population stats
- Generation counter
- FPS monitor
- Tech stack info
- Commit info (last deploy)

### Project Links
1. HELIOS - Solar system simulation
2. X (Twitter) - @LazyShivam
3. Blog - Technical writing
4. Learn - ML/AI curriculum
5. Simulations - Chladni, etc.
6. Software - AutoCrate, tools
7. About Me - Portfolio

## Navigation

| Element | Destination |
|---------|-------------|
| Helios Icon | helios.too.foo |
| Blog Icon | blog.too.foo |
| Learn Icon | learn.too.foo |
| Simulations Icon | chladni.too.foo |
| Software Icon | autocrate.too.foo |
| About Me Icon | portfolio.too.foo |
| X Icon | x.com/LazyShivam |

## Field Manual

Accessible via "FIELD MANUAL" link in telemetry bar.
- Quick start commands
- Vision statement
- File structure
- Development workflow
- Toolchain info

## Common Tasks

### Updating project links
1. Edit `index.html` - find `.monolith` anchor tags
2. Update `href` attribute
3. Ensure icon exists in `assets/islands/`

### Adding a new project to constellation
1. Add icon SVG to `assets/islands/`
2. Add `.monolith` element in `index.html`
3. Position using `.pos-N` class (adjust rotation degrees)
4. Update Field Manual content

### Changing particle simulation
1. Modify `src/main.rs` - uses DNA/sim
2. Adjust population, forces, colors
3. Keep performance in mind (landing page, not showcase)

## Performance

- Target: 60 FPS on mobile
- Keep boid count low (~200-400)
- Optimize for first paint (landing page UX)
- Background simulation should enhance, not distract

## Deployment

Deployed to too.foo (root domain).
Build command in deployment pipeline:
```bash
trunk build WELCOME/index.html --release
```

Output served from `WELCOME/dist/`.

## Testing

Visual testing:
```bash
npx playwright test tests/welcome.spec.ts
```

Manual checklist:
- [ ] All project links work
- [ ] Constellation responsive on mobile
- [ ] Particle sim runs smoothly
- [ ] Field Manual opens correctly
- [ ] Telemetry shows correct stats
- [ ] Commit info displays latest deploy
