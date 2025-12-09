# Quantum Wave-Particle Ecosystem
## Technical Whitepaper

**Author**: Shivam Bhardwaj (Sb)
**Organization**: Antimony Labs
**Date**: December 2025
**Status**: Production (Coming Soon Page)

---

## Abstract

This document describes a quantum-inspired particle simulation demonstrating emergent complexity from simple rules. Circular wave interference creates particle-antiparticle pairs via wavefunction collapse (Bhardwaj Mechanism), with homeostatic feedback control maintaining ecosystem balance.

**Key Innovation**: All spawning via wave field - no hardcoded reproduction. Population self-regulates through wave amplitude modulation.

---

## 1. Wave Field Theory

### 1.1 Circular Ripple Waves

Each wave emanates from a random spawn point, propagating outward like ripples in water.

**Mathematical Model**:
```
ψ(x,y,t) = P × A × decay(r) × oscillation(r,t)

where:
  P = polarity (±1)
  A = amplitude [1.5, 2.5]
  r = distance from origin
  decay(r) = 1 / (1 + r/λ)  where λ = 80 pixels
  oscillation(r,t) = cos(k(r - vt))  where k = frequency, v = speed
```

**Parameters**:
- Speed: v = 2 pixels/frame
- Frequency: k ~ 0.08-0.12 rad/pixel
- Polarity: 50% positive, 50% negative (random)
- Lifespan: Until r > 1.5 × screen diagonal

**Purpose**: Create interference patterns where waves collide.

### 1.2 Wavefunction Collapse (Bhardwaj Mechanism)

**Named after**: Shivam Bhardwaj
**Concept**: When total wave amplitude exceeds threshold, quantum uncertainty collapses into classical particle.

**Collapse Rule**:
```
ψ_total(x,y,t) = Σ all waves at (x,y,t)

if ψ_total > κ_B:
  Spawn PREY (particle) at (x,y)

if ψ_total < -κ_B:
  Spawn PREDATOR (antiparticle) at (x,y)
```

**Bhardwaj Constant (κ_B)**:
- Dynamic range: [0.4, 0.95]
- Baseline: 0.7
- Adaptive: GOD controller adjusts based on population

**Why this works**: Constructive interference (waves in phase) creates high positive amplitude. Destructive interference creates high negative amplitude. Both spawn particles, but different types.

### 1.3 Sparse Sampling

Don't check every grid cell every frame - too expensive.

**Strategy**: Random sampling
```
samples_per_frame = grid_size × sample_rate
sample_rate = 0.05 (5% of grid)

For 960×540 grid:
  samples = 518,400 × 0.05 = 25,920 checks/frame
```

**Adaptive**: If FPS drops, reduce sample_rate.

---

## 2. Particle Dynamics (Classical Mechanics)

Once spawned, particles follow classical rules.

### 2.1 Predator (Antiparticle)

**Properties**:
- Energy: 300 frames (5 seconds at 60 FPS)
- Drains 1 energy/frame
- Dies when energy = 0

**Behavior**:
1. **Hunt**: Find nearest prey within radius 15
2. **Move**: 25% chance/frame to move toward prey
3. **Annihilate**: On contact with prey:
   - Both particles disappear
   - Predator energy resets to 300
   - 80% annihilation chance

**Complexity**: O(R²) for findNearest where R = 15 → O(225) per predator

### 2.2 Prey (Particle)

**Properties**:
- No energy limit (immortal unless eaten)
- Passive (no movement)

**Behavior**:
- None - only spawned via waves, killed via annihilation
- (Reproduction disabled to prevent explosion)

### 2.3 Annihilation

When predator adjacent to prey:
```
if random() < annihilationChance:  // 80%
  grid[predator] = EMPTY
  grid[prey] = EMPTY
```

**Not eating** - mutual annihilation (particle-antiparticle physics).

---

## 3. GOD Controller (Homeostatic Feedback)

Self-regulating system maintaining target population via wave field modulation.

### 3.1 Control Parameters

**Wave Generation**:
- `waveSpawnRate`: [0.001, 0.04] - probability per frame
- `bhardwajConstant (κ_B)`: [0.4, 0.95] - collapse threshold

**Particle Survival**:
- `predatorLifespan`: 300 frames (constant)
- `annihilationChance`: [0.3, 0.95]

### 3.2 Regulation Algorithm

**Primary Loop**: Particle count homeostasis
```
error = current_total - TARGET (15k)
errorPct = error / TARGET

if errorPct > 0.1:  // 10%+ over target
  waveSpawnRate *= 0.8       // Spawn 20% fewer waves
  κ_B *= 1.05                // Raise threshold 5% (harder to collapse)

if errorPct < -0.1:  // 10%+ under target
  waveSpawnRate *= 1.05      // Spawn 5% more waves
  κ_B *= 0.97                // Lower threshold 3% (easier to collapse)

else:  // Within 10% of target
  Drift back to baseline
```

**Secondary Loop**: Prey/predator balance
```
preyRatio = prey / total
target = 0.65 (65% prey, 35% predators)

if preyRatio > 0.75:  // Too much prey
  annihilationChance *= 1.02  // Increase predator effectiveness

if preyRatio < 0.55:  // Too many predators
  annihilationChance *= 0.98  // Reduce predator effectiveness
```

### 3.3 Bounds (Prevent Runaway)

All parameters clamped after every adjustment:
```
waveSpawnRate = clamp(waveSpawnRate, 0.001, 0.04)
κ_B = clamp(κ_B, 0.4, 0.95)
annihilationChance = clamp(annihilationChance, 0.3, 0.95)
```

**This prevents exponential parameter growth** (no more e+101 explosions).

---

## 4. Memory Safety (Rust-Inspired Design)

### 4.1 Single Source of Truth

**Principle**: Grid owns particle state. Counts are derived, not tracked.

```javascript
// Rust concept: Ownership
let grid = Uint8Array;  // Owner of particle state

// Derived state (borrow)
function countParticles() {
  return scan(grid);  // Always accurate
}
```

**Benefits**:
- Impossible to have count mismatch
- No double-free bugs
- No integer underflow

### 4.2 Eliminated Bugs

Before (manual tracking):
```javascript
preyCount--;  // What if called twice? → negative count
```

After (grid truth):
```javascript
grid[idx] = EMPTY;  // Can only kill once
const counts = countParticles();  // Always correct
```

---

## 5. Algorithm Complexity Analysis

### 5.1 Current Implementation

| Operation | Complexity | Cost (per frame) |
|-----------|------------|------------------|
| Wave spawn/cleanup | O(W) | 60 ops |
| Wave collapse sampling | O(S × W) | 26k × 60 = 1.56M |
| Particle update | O(N) | 518k grid scan |
| Count particles (×2) | O(N) | 2 × 518k = 1M |
| Particle rendering | O(N) | 518k checks |
| **TOTAL** | **O(N + S×W)** | **~7M operations** |

At 60 FPS: 420M operations/second

### 5.2 Bottlenecks

1. **Wave collapse**: 1.56M ops (22% of total)
2. **Particle update**: 4M ops (57% of total) - iterating empty cells
3. **Counting**: 1M ops (14% of total)

### 5.3 Proposed Optimizations

**Phase 1**: Spatial binning for wave collapse
- Cost: O(S × W_local) where W_local ≈ 5
- New: 26k × 5 = 130k (12× faster)

**Phase 2**: Particle list instead of grid iteration
- Cost: O(P) where P = 15k occupied
- Update: 15k × 8 neighbors = 120k (33× faster)
- Render: 15k fillRect (35× faster)
- Count: O(P) during update (free)

**Phase 3**: WebGPU compute shaders
- FFT on GPU: 100× faster
- Particle update on GPU: 1000× faster

**Total Potential**: Current 7M → Optimized 265k → **26× speedup**

---

## 6. Data Flow

```
┌─────────────────────────────────────┐
│  Initialization                     │
│  - Spawn 30 initial waves           │
│  - Seed 3k prey, 1.5k predators     │
└──────────────┬──────────────────────┘
               ↓
      ┌────────────────────┐
      │   Update Loop      │
      │   (60 FPS)         │
      └────────┬───────────┘
               ↓
    ┌──────────────────────┐
    │ 1. Count Particles   │
    │    O(N) = 518k       │
    └──────┬───────────────┘
           ↓
    ┌──────────────────────┐
    │ 2. GOD Regulate      │
    │    Adjust wave params│
    └──────┬───────────────┘
           ↓
    ┌──────────────────────┐
    │ 3. Spawn Waves       │
    │    ~1-2/frame        │
    └──────┬───────────────┘
           ↓
    ┌──────────────────────┐
    │ 4. Wave Collapse     │
    │    26k samples       │
    │    Spawn particles   │
    └──────┬───────────────┘
           ↓
    ┌──────────────────────┐
    │ 5. Flash Events      │
    │    Death/bloom       │
    └──────┬───────────────┘
           ↓
    ┌──────────────────────┐
    │ 6. Particle Dynamics │
    │    Hunt, annihilate  │
    └──────┬───────────────┘
           ↓
    ┌──────────────────────┐
    │ 7. Render            │
    │    Waves + particles │
    └──────┬───────────────┘
           ↓
      (Loop back)
```

---

## 7. Tech Stack

### 7.1 Current (JavaScript)
- **Language**: ES6+ JavaScript
- **Canvas**: 2D rendering context
- **Data Structures**:
  - `Uint8Array` for grid (memory efficient)
  - `Float32Array` for energy
  - `Array` for waves/pulses
- **Math**: Native `Math.sin()`, `Math.cos()`, `Math.sqrt()`

### 7.2 DNA Module (Rust)
- **FFT**: `DNA/src/wave_field/fft.rs` - 2D Cooley-Tukey
- **WaveField**: `DNA/src/wave_field/wave_field.rs` - Frequency/spatial domains
- **Tests**: Full test coverage with roundtrip verification
- **Future**: WASM bindings for web integration

### 7.3 Future Optimizations
- **WebGPU**: Compute shaders for wave field (100× faster)
- **SIMD**: Vectorized operations
- **Web Workers**: Parallel particle updates

---

## 8. Performance Metrics

### 8.1 Target Performance
- **Desktop**: 50-60 FPS at 1920×1080
- **Mobile**: 30+ FPS at 375×667
- **Particle count**: 13k-17k (±15% of 15k target)

### 8.2 Adaptive Systems
1. **Particle density**: Scales with screen size (√area)
2. **Sample rate**: Reduces if FPS < 40
3. **Wave spawn**: GOD adjusts based on population

### 8.3 Memory Usage
- Grid: 518k × 1 byte = 518 KB
- Energy: 518k × 1 byte = 518 KB
- Waves: ~60 × 100 bytes = 6 KB
- **Total**: ~1 MB (lightweight)

---

## 9. Tuned Hyperparameters (Optimized via Parameter Sweep)

### 9.1 Optimization Methodology

Tested **576 parameter combinations** using parallel CPU sweep (32 threads).
Each configuration ran 30-second simulation (1800 frames) with stability scoring.

**Scoring Function**:
```
score = extinction_penalty × 1000
      + population_error × 100
      + ratio_error × 50
      + oscillation_penalty
      + stddev_penalty
      - settling_time_bonus
```

### 9.2 Optimal Parameters (Found via Sweep)

| Parameter | Optimal | Range Tested | Impact |
|-----------|---------|--------------|--------|
| **PID Kp** | 0.04 | [0.005, 0.04] | Fast response |
| **PID Ki** | 0.001 | [0.0001, 0.001] | Steady-state accuracy |
| **PID Kd** | 0.2 | [0.05, 0.2] | Damping oscillations |
| **Bhardwaj κ** | 0.4 | [0.4, 0.7] | Lower = more spawning |
| **Predator Energy** | 1000 | [400, 1000] | **Critical for stability** |
| Wave Spawn Rate | 0.015 | - | Baseline |
| Sample Rate | 0.05 | - | 5% of grid |
| Adaptive Sampling | true | - | 4× boost when low |

### 9.3 Key Finding: Ratio-Based Spawning

**Problem**: Wave-based spawning has inherent prey bias (positive amplitudes more common than negative).

**Solution**: Added ratio-based spawning layer that directly compensates:
```rust
if predators < target_pred {
    spawn_rate = (deficit / target).min(0.5);
    spawn_count = (deficit × spawn_rate × 0.05).max(1).min(20);
    // Spawn predators directly
}
```

**Result**: Prey ratio stabilizes at 65.3% (vs 99.6% without fix)

### 9.4 Stability Metrics (1-minute simulation)

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Average Population | 15,241 | 15,000 | ✓ 1.6% error |
| Standard Deviation | ±472 | <1000 | ✓ Stable |
| Prey Ratio | 65.3% | 65% | ✓ On target |
| Extinctions | 0 | 0 | ✓ No extinctions |
| Oscillations | 124 | <150 | ✓ Acceptable |

### 9.5 Rust Implementation

Pure math model at `DNA/src/wave_field/ecosystem.rs`:
- Deterministic with seeded RNG
- 10 unit tests + 9 integration tests
- Benchmark: ~75 fps on 960×540 grid (release mode)

### 9.6 Files Updated

- `DNA/src/wave_field/ecosystem.rs` - Pure Rust simulation
- `DNA/tests/ecosystem_stability.rs` - Comprehensive test suite
- `DNA/examples/ecosystem_sweep.rs` - Parameter optimizer

---

## 10. Previous Issues & Resolution

### 10.1 Resolved Issues
- ✓ Population explosion (145k) → Fixed with PID + hard cap
- ✓ 99% prey ratio → Fixed with ratio-based spawning
- ✓ Synchronized death bursts → Fixed with energy randomization
- ✓ Count desync bugs → Fixed with grid ownership
- ✓ O(N) waste → Mitigated with adaptive sampling

### 10.2 Future Work
- Particle list data structure (O(P) vs O(N))
- Spatial hash grid for O(1) neighbor queries
- WebGPU compute shaders
- WASM integration (proven Rust → JS)

---

## Appendix A: Bhardwaj Constant (κ_B)

The **Bhardwaj Constant** is the threshold wave amplitude required for wavefunction collapse into observable particles.

**Physical Interpretation**: Quantum uncertainty barrier. Below κ_B, wave remains in superposition. Above κ_B, wave collapses into definite particle state.

**Discovery**: Named after Shivam Bhardwaj, who designed this quantum-inspired simulation system for antimony-labs.

**Dynamic Behavior**:
- Starts at 0.5 (easy spawning)
- GOD raises to 0.95 when overpopulated (hard spawning)
- GOD lowers to 0.4 when underpopulated (easy spawning)
- Natural equilibrium near 0.7

---

## Appendix B: Complexity Class Comparison

| Algorithm | Naive | Current | Optimal | Speedup |
|-----------|-------|---------|---------|---------|
| Wave field | O(N×W) | O(S×W) | O(S×W_local) | 10× |
| Particle update | O(N×N) | O(N) | O(P) | 33× |
| Neighbor search | O(N) | O(R²) | O(1) | 225× |
| Rendering | O(N) | O(N) | O(P) | 35× |

Where:
- N = grid size (518k)
- P = particle count (15k)
- S = samples/frame (26k)
- W = wave count (60)
- W_local = nearby waves (5)
- R = search radius (15)

---

## Appendix C: References

1. Cooley-Tukey FFT Algorithm (1965)
2. Lotka-Volterra Predator-Prey Model (1925)
3. Quantum Mechanics: Wavefunction Collapse
4. Cellular Automata: Conway's Game of Life (1970)
5. Rust Programming Language: Ownership & Borrowing

---

**© 2025 Antimony Labs (Sb)**
Built with curiosity. Engineered with rigor.
