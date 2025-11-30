# too.foo Field Manual

**The only document you need to read.**

Last updated: 2025-11-30

---

## Quick Start

```bash
# Reload shell (first time only)
source ~/.bashrc

# Start development
run toofoo          # Landing page (http://localhost:3000)
run website         # Personal site (http://localhost:3030)
mon                 # System monitor (btop)

# Work on GitHub issue
work 42             # Creates worktree, fetches issue details
```

---

## System Configuration

### Rust Toolchain (Stable)

```toml
# rust-toolchain.toml (S3M2P root)
[toolchain]
channel = "stable"          # Rust 1.91.1
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

**Why stable?** Production deploys require stability.
**Why wasm32?** All web apps compile to WebAssembly.

### Global Tools

| Tool | Version | Purpose |
|------|---------|---------|
| rustc | 1.91.1 | Rust compiler |
| cargo | 1.91.1 | Package manager |
| trunk | 0.21.14 | WASM bundler/server |
| zellij | 0.43.1 | Terminal multiplexer |

### Update Tools

```bash
# Update Rust
rustup update stable

# Update trunk
cargo install --locked trunk --force

# Update dependencies (safe)
cd ~/S3M2P && cargo update
```

---

## File Structure

### L1: Main Bubbles

```
S3M2P/
├── DNA/          # Core engine (algorithms, schemas, GPU)
├── MCAD/         # Mechanical CAD (open-source NX)
├── ECAD/         # Electronics CAD (open-source Altium)
├── HELIOS/       # Solar system simulation
├── SIMULATIONS/  # Other simulations (Chladni, Boids)
├── SHIVAM/       # About Me timeline
├── BLOG/         # Blog platform
└── LEARN/        # Interactive lessons
```

### L2: Projects per Bubble

```
MCAD/                       # Open-source NX in Rust
├── CORE/                   # B-rep kernel (from scratch)
├── AUTOCRATE/              # Shipping crate generator
├── GEARS/                  # Gear designer [scaffold]
├── CFD/                    # Fluid dynamics [scaffold, GPU]
├── STRESS/                 # FEA analysis [scaffold, GPU]
├── THERMAL/                # Thermal sim [scaffold, GPU]
├── CAM/                    # Toolpath/G-code gen
├── EXPORT/                 # STEP writer
├── CLI/                    # sbl-mcad command
└── WEB/                    # WASM viewer

ECAD/                       # Open-source Altium in Rust
├── CORE/                   # Schematic + PCB (from scratch)
├── POWER_SUPPLY/           # Power circuits [scaffold]
├── AMPLIFIERS/             # Amplifiers [scaffold]
├── EXPORT/                 # Gerber X2 writer (from scratch)
├── DRC/                    # Design rule checker
├── CLI/                    # sbl-ecad command
└── WEB/                    # WASM editor

DNA/src/
├── lib.rs                  # Core types
├── schema/                 # DNA code parser (.dna.toml files)
├── sim/                    # Simulation algorithms (boids, etc.)
├── compute/                # GPU abstraction (wgpu)
├── responsive.rs           # Mobile-first view rules
└── export/                 # STEP/Gerber serializers

SHIVAM/
├── CORE/                   # Timeline engine
├── quarters/               # 128 quarters of life data
└── WEB/                    # shivambhardwaj.com deploy
```

---

## DNA Code

Everything in too.foo is defined via DNA code - TOML configuration files.

### Part Example

```toml
# bracket.dna.toml
[part]
name = "bracket"
material = "aluminum_6061"

[[features]]
type = "box"
dims = [100.0, 50.0, 10.0]  # mm

[[features]]
type = "hole"
pos = [25.0, 25.0]
diameter = 5.0
through = true
```

### Machine Example

```toml
# cnc.dna.toml
[machine]
type = "cnc_router"
name = "3018"

[workspace]
x = 300.0
y = 180.0
z = 45.0

[controller]
firmware = "grbl"
```

---

## Algorithms (DNA/src/sim/)

### Boids Flocking
- Separation, Alignment, Cohesion
- Spatial grid for O(1) neighbor queries
- Used in: landing page, simulations

### Rendering (3Blue1Brown style)
- Canvas 2D primitives
- Animation loops
- Coordinate transforms

### Physics
- Collision detection
- Spatial partitioning
- Energy systems

---

## Development Workflow

### LLM-Centric (You Don't Read Code)

```
1. Create GitHub Issue
2. Open terminal in S3M2P root
3. Tell LLM: "work on issue 42"
4. LLM creates worktree, works, tests
5. LLM pushes to preview/issue-42
6. You check: preview-issue-42.too.foo
7. LLM creates PR → merges to main
8. Auto-deploys to too.foo
```

### Worktree Commands

```bash
# Create worktree for issue
git worktree add ~/worktrees/issue-42 -b issue-42

# List worktrees
git worktree list

# Remove when done
git worktree remove ~/worktrees/issue-42
```

### Validation Before Deploy

```bash
# LLM runs this before pushing
./SCRIPTS/validate.sh

# Which does:
cargo check --workspace
cargo test --workspace
cargo clippy --workspace
trunk build --release SIM/TOOFOO/index.html
```

---

## Run Commands (Case Insensitive)

```bash
run toofoo        # too.foo landing (port 3000)
run helios        # Solar system (port 3001)
run mcad          # MCAD viewer (port 3002)
run ecad          # ECAD editor (port 3003)
run simulations   # Simulations (port 3004)
run blog          # Blog (port 3005)
run learn         # Learning platform (port 3006)
run website       # shivambhardwaj.com (port 3030)
run list          # Show all projects
```

**Features:**
- Auto port selection (finds next free if busy)
- Case insensitive (`run TOOFOO` works)
- Opens browser automatically

---

## Terminal Setup

### Terminator (Tiled Terminal)

Already installed. Use for tiled layouts:

```bash
Ctrl+Shift+O     Split horizontal
Ctrl+Shift+E     Split vertical
Ctrl+Shift+W     Close pane
Ctrl+Shift+T     New tab
```

### btop (System Monitor)

```bash
mon              # Launch btop
q                # Quit btop
```

### Copy/Paste (X11 Selection)

- **Select with mouse** → Auto-copies
- **Middle-click** → Pastes (Ubuntu default)

---

## Build Commands

### Development

```bash
# Check (fast - no build)
cargo check --workspace

# Build WASM app
trunk build SIM/TOOFOO/index.html

# Serve with hot reload
trunk serve SIM/TOOFOO/index.html --port 3000 --open

# Run tests
cargo test --workspace
```

### Release

```bash
# Build optimized
trunk build --release SIM/TOOFOO/index.html

# Deploy
./SCRIPTS/deploy.sh toofoo --publish
```

---

## Philosophy

### "Let AI Design, Humans Build"

- LLM writes all code
- You review previews, approve PRs
- Everything is DNA code (TOML)
- Deterministic, reproducible, scriptable

### Minimal Dependencies

**Write from scratch:**
- B-rep kernel (MCAD)
- STEP exporter (ISO 10303-21)
- Gerber X2 generator (ECAD)
- G-code generator (CAM)

**Only use external for:**
- GPU access (wgpu)
- Math (glam)
- WASM bindings (wasm-bindgen)
- Serialization (serde)

### Mobile-First

50% of users are on mobile:
- Touch targets minimum 48px
- Single column on mobile
- All bubbles scale to viewport
- Test mobile FIRST, then desktop

---

## Deployment

### Preview vs Production

```
main branch         → too.foo (production)
preview/issue-X     → preview-issue-X.too.foo (auto-deploy)
feature/X           → localhost only
```

### Cloudflare Pages

- Push to `preview/*` branch → auto-deploys preview
- Merge to `main` → auto-deploys production
- Rollback: revert commit

---

## Security

### Audit Commands

```bash
# Run weekly
./SCRIPTS/audit.sh

# Manual check
cargo deny check advisories
cargo audit
cargo outdated
```

### deny.toml Policy

```toml
[advisories]
vulnerability = "deny"    # Block vulnerable crates
unmaintained = "warn"     # Warn on unmaintained

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
```

---

## Performance

### Why 1000+ Packages Compile?

```
Your code
  └─> leptos
       ├─> tokio (async runtime)
       │    └─> 50+ crates
       ├─> serde (serialization)
       │    └─> 30+ crates
       └─> 100+ more dependencies
```

**This is normal.** Rust compiles all dependencies from source for safety.

**Good news:**
- First compile: slow (10-20 min)
- Subsequent: fast (only changed code)
- Cached in `target/` folder

### Speed Up Builds

```bash
# Use sccache (build cache)
cargo install sccache
export RUSTC_WRAPPER=sccache

# Or use mold linker (faster linking)
sudo apt install mold
```

---

## Troubleshooting

### Compilation Errors

```bash
# Clean build
cargo clean
cargo build

# Update lockfile
cargo update

# Check specific package
cargo check --package mcad-core
```

### Port Already in Use

`run` command auto-increments port if busy:
```
Port 3000 busy → tries 3001
Port 3001 busy → tries 3002
etc.
```

### Worktree Issues

```bash
# List worktrees
git worktree list

# Remove broken worktree
git worktree remove ~/worktrees/issue-X --force

# Prune stale
git worktree prune
```

---

## Keyboard Shortcuts Summary

### Zellij

| Key | Action |
|-----|--------|
| `Alt+t` | New tab |
| `Alt+n` | New pane |
| `Alt+d` | Detach |
| `Alt+h/j/k/l` | Navigate |
| `Alt+1/2/3` | Switch tabs |

### Run Script

| Command | Opens |
|---------|-------|
| `run toofoo` | http://localhost:3000 |
| `run website` | http://localhost:3030 |
| `run list` | Show all projects |

### Git Workflow

| Command | Action |
|---------|--------|
| `git worktree add ~/worktrees/issue-X -b issue-X` | Create worktree |
| `git push -u origin preview/issue-X` | Deploy preview |
| `gh pr create` | Create PR |

---

## Architecture Principles

### 1. DNA Code (TOML)
Everything is configuration:
- Parts → STEP files
- Machines → G-code profiles
- PCBs → Gerber files
- Lessons → Interactive content

### 2. LLM-Friendly
- Text-based inputs
- Structured JSON outputs
- Small composable functions
- Well-documented types

### 3. Platform-Agnostic
Works on Mac, PC, iPad, Mobile:
- Responsive by default
- WASM compiles everywhere
- No platform-specific code

### 4. From Scratch
Minimize external dependencies:
- Own B-rep kernel
- Own file exporters
- Full control over stack

---

## Vision

### MCAD = Open-Source NX
- Parametric CAD
- CAM/Toolpaths
- CFD (GPU)
- FEA (GPU)
- Thermal (GPU)

### ECAD = Open-Source Altium
- Schematic capture
- PCB layout
- Gerber export
- DRC checking

### LEARN = Beyond Brilliant
- Interactive lessons
- LLM-generated content
- Totally free
- 3Blue1Brown style animations

### SHIVAM = Life Timeline
- 128 quarters from birth
- Celestial events per quarter
- Blog post links
- Journey through memory

---

## Contributing (LLM Instructions)

When told "work on issue X":

1. Check worktree: `ls ~/worktrees/issue-X`
2. Create if needed: `git worktree add ~/worktrees/issue-X -b issue-X`
3. Fetch issue: `gh issue view X`
4. Work on implementation
5. Validate: `./SCRIPTS/validate.sh`
6. Push preview: `git push -u origin preview/issue-X`
7. Report: "Preview at preview-issue-X.too.foo"
8. Create PR when approved

---

## Shortcuts Cheat Sheet

```bash
# Development
run <project>              # Start dev server
cargo check --workspace    # Type check
cargo test --workspace     # Run tests
trunk build --release      # Production build

# Session
zellij                     # Start multiplexer
Alt+d                      # Detach
zellij attach              # Reattach

# Git
git worktree add ~/worktrees/issue-X -b issue-X
git push -u origin preview/issue-X
gh pr create

# Maintenance
./SCRIPTS/audit.sh         # Security check
cargo update               # Update deps
cargo outdated             # Check versions
```

---

## External Resources

- Rust Book: https://doc.rust-lang.org/book/
- wgpu Guide: https://wgpu.rs/
- Trunk Docs: https://trunkrs.dev/
- Zellij Docs: https://zellij.dev/

---

**Remember:** This is an LLM-centric codebase. You don't read the code - the LLM does. You read this manual, create issues, and review previews.

**Vision:** Text → Manufacturing. Let AI design, humans build.
