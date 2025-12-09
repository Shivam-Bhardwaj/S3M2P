// Test quantum wave-particle ecosystem
// Simulates wavefunction collapse without rendering

const WIDTH = 960;
const HEIGHT = 540;
const PIXEL_COUNT = WIDTH * HEIGHT;
const TARGET_CELL_COUNT = 15000;
const SCALE = 2;

const EMPTY = 0;
const PREY = 1;
const PREDATOR = 2;

const NUM_PREY_WAVES = 8;
const NUM_PRED_WAVES = 4;

let grid = new Uint8Array(PIXEL_COUNT);
let energy = new Uint8Array(PIXEL_COUNT);
let preyCount = 0;
let predatorCount = 0;
let time = 0;
let sampleRate = 0.05;

// Wave class
class Wave {
  constructor(type) {
    this.type = type;
    this.freqX = 0.01 + Math.random() * 0.04;
    this.freqY = 0.01 + Math.random() * 0.04;
    this.phaseX = Math.random() * Math.PI * 2;
    this.phaseY = Math.random() * Math.PI * 2;
    this.omega = 0.002 + Math.random() * 0.008;
    this.amplitude = 1.0;
  }

  getValue(x, y, t) {
    return this.amplitude *
           Math.sin(this.freqX * x + this.phaseX) *
           Math.sin(this.freqY * y + this.phaseY) *
           Math.sin(this.omega * t);
  }
}

// Wave pulse class
class WavePulse {
  constructor(cx, cy, radius, amplitude, lifespan, type) {
    this.cx = cx;
    this.cy = cy;
    this.radius = radius;
    this.amplitude = amplitude;
    this.life = lifespan;
    this.maxLife = lifespan;
    this.type = type;
  }

  getValue(x, y) {
    const dx = x - this.cx;
    const dy = y - this.cy;
    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist > this.radius) return 0;

    const sigma = this.radius / 3;
    const factor = Math.exp(-(dist * dist) / (2 * sigma * sigma));
    const decayFactor = this.life / this.maxLife;
    return this.amplitude * factor * decayFactor;
  }

  update() {
    this.life--;
    return this.life > 0;
  }
}

// Initialize waves
const preyWaves = [];
const predatorWaves = [];
const pulses = [];

for (let i = 0; i < NUM_PREY_WAVES; i++) {
  preyWaves.push(new Wave('prey'));
}
for (let i = 0; i < NUM_PRED_WAVES; i++) {
  predatorWaves.push(new Wave('predator'));
}

const allWaves = [...preyWaves, ...predatorWaves];

// GOD controller
const GOD = {
  bhardwajConstant: 0.85,
  targets: {
    cellCount: TARGET_CELL_COUNT,
    preyRatio: { min: 0.55, max: 0.75 },
    minPredators: 50,
    minPrey: 100,
  },
  params: {
    predEatChance: { val: 0.7, min: 0.3, max: 0.95, base: 0.7 },
    predMoveChance: { val: 0.15, min: 0.05, max: 0.4, base: 0.15 },
    preyReproduce: { val: 0.02, min: 0.01, max: 0.06, base: 0.02 },
    maxEnergy: 200,
  },
  history: { preyRatios: [], windowSize: 60 },

  regulate(preyCount, predatorCount) {
    const total = preyCount + predatorCount;
    const preyRatio = total > 0 ? preyCount / total : 0.5;

    this.history.preyRatios.push(preyRatio);
    if (this.history.preyRatios.length > this.history.windowSize) {
      this.history.preyRatios.shift();
    }

    const avgPreyRatio = this.history.preyRatios.reduce((a, b) => a + b, 0) / this.history.preyRatios.length;

    const error = total - this.targets.cellCount;
    const errorPct = error / this.targets.cellCount;

    // Count control
    if (errorPct > 0.1) {
      for (const wave of allWaves) wave.amplitude *= 0.97;
      this.bhardwajConstant = Math.min(0.95, this.bhardwajConstant * 1.005);
    } else if (errorPct < -0.1) {
      for (const wave of allWaves) wave.amplitude *= 1.03;
      this.bhardwajConstant = Math.max(0.7, this.bhardwajConstant * 0.995);
    } else {
      this.bhardwajConstant += (0.85 - this.bhardwajConstant) * 0.02;
    }

    // Balance control
    if (avgPreyRatio > this.targets.preyRatio.max) {
      for (const wave of predatorWaves) wave.amplitude *= 1.04;
      for (const wave of preyWaves) wave.amplitude *= 0.99;
      this.params.predEatChance.val *= 1.02;
    } else if (avgPreyRatio < this.targets.preyRatio.min) {
      for (const wave of preyWaves) wave.amplitude *= 1.04;
      for (const wave of predatorWaves) wave.amplitude *= 0.99;
      this.params.predEatChance.val *= 0.98;
    }

    // Emergency
    if (predatorCount < this.targets.minPredators) {
      for (const wave of predatorWaves) wave.amplitude *= 1.5;
      this.bhardwajConstant *= 0.9;
    }
    if (preyCount < this.targets.minPrey) {
      for (const wave of preyWaves) wave.amplitude *= 1.5;
      this.bhardwajConstant *= 0.9;
    }

    // Clamp
    for (const wave of allWaves) {
      wave.amplitude = Math.max(0.3, Math.min(2.0, wave.amplitude));
    }
    this.bhardwajConstant = Math.max(0.7, Math.min(0.95, this.bhardwajConstant));
    this.params.predEatChance.val = Math.max(this.params.predEatChance.min, Math.min(this.params.predEatChance.max, this.params.predEatChance.val));
  }
};

function collapseWavefunction(x, y, t) {
  const idx = y * WIDTH + x;
  if (grid[idx] !== EMPTY) return;
  if (preyCount + predatorCount >= TARGET_CELL_COUNT) return;

  let preyAmp = 0;
  let predAmp = 0;

  for (const wave of preyWaves) {
    preyAmp += wave.getValue(x, y, t);
  }
  for (const wave of predatorWaves) {
    predAmp += wave.getValue(x, y, t);
  }

  for (const pulse of pulses) {
    const pulseVal = pulse.getValue(x, y);
    if (pulse.type === 'bloom') {
      preyAmp += pulseVal;
      predAmp += pulseVal * 0.3;
    }
  }

  preyAmp = (preyAmp + NUM_PREY_WAVES) / (2 * NUM_PREY_WAVES);
  predAmp = (predAmp + NUM_PRED_WAVES) / (2 * NUM_PRED_WAVES);

  if (preyAmp > GOD.bhardwajConstant && preyCount < TARGET_CELL_COUNT * 0.8) {
    grid[idx] = PREY;
    preyCount++;
  } else if (predAmp > GOD.bhardwajConstant && predatorCount < TARGET_CELL_COUNT * 0.4) {
    grid[idx] = PREDATOR;
    energy[idx] = GOD.params.maxEnergy;
    predatorCount++;
  }
}

function getNeighborIndices(idx) {
  const x = idx % WIDTH;
  const y = Math.floor(idx / WIDTH);
  const neighbors = [];
  for (let dx = -1; dx <= 1; dx++) {
    for (let dy = -1; dy <= 1; dy++) {
      if (dx === 0 && dy === 0) continue;
      const nx = x + dx;
      const ny = y + dy;
      if (nx >= 0 && nx < WIDTH && ny >= 0 && ny < HEIGHT) {
        neighbors.push(ny * WIDTH + nx);
      }
    }
  }
  return neighbors;
}

function triggerDeathFlash() {
  const x = Math.floor(WIDTH * 0.4 + Math.random() * WIDTH * 0.5);
  const y = Math.floor(Math.random() * HEIGHT);
  const radius = 40 + Math.random() * 60;
  const lifespan = 60 + Math.random() * 240;

  // Kill in radius
  for (let dy = -radius; dy <= radius; dy++) {
    for (let dx = -radius; dx <= radius; dx++) {
      if (dx * dx + dy * dy <= radius * radius) {
        const nx = x + dx;
        const ny = y + dy;
        if (nx >= 0 && nx < WIDTH && ny >= 0 && ny < HEIGHT) {
          const idx = ny * WIDTH + nx;
          if (grid[idx] === PREY) preyCount--;
          else if (grid[idx] === PREDATOR) predatorCount--;
          grid[idx] = EMPTY;
        }
      }
    }
  }

  pulses.push(new WavePulse(x, y, radius, -2.0, lifespan, 'death'));
}

function triggerBloomFlash() {
  const x = Math.floor(WIDTH * 0.4 + Math.random() * WIDTH * 0.5);
  const y = Math.floor(Math.random() * HEIGHT);
  const radius = 40 + Math.random() * 60;
  const lifespan = 60 + Math.random() * 240;

  pulses.push(new WavePulse(x, y, radius, 1.5, lifespan, 'bloom'));
}

function update() {
  time++;
  const total = preyCount + predatorCount;
  const preyRatio = total > 0 ? preyCount / total : 0.5;

  // GOD regulate
  GOD.regulate(preyCount, predatorCount);

  // Wave collapse sampling
  const samplesToCheck = Math.floor(PIXEL_COUNT * sampleRate);
  for (let i = 0; i < samplesToCheck && total < TARGET_CELL_COUNT; i++) {
    const x = Math.floor(Math.random() * WIDTH);
    const y = Math.floor(Math.random() * HEIGHT);
    collapseWavefunction(x, y, time);
  }

  // Update pulses
  for (let i = pulses.length - 1; i >= 0; i--) {
    if (!pulses[i].update()) {
      pulses.splice(i, 1);
    }
  }

  // Trigger events
  const countError = total - TARGET_CELL_COUNT;
  if (countError > 3000 && Math.random() < 0.05) {
    triggerDeathFlash();
  } else if (countError < -3000 && Math.random() < 0.04) {
    triggerBloomFlash();
  }
  if (predatorCount < 50 && Math.random() < 0.1) triggerBloomFlash();
  if (preyCount < 100 && Math.random() < 0.1) triggerBloomFlash();

  // Particle dynamics
  let preyBorn = 0, preyDied = 0, predBorn = 0, predStarved = 0, predAte = 0;

  for (let idx = 0; idx < PIXEL_COUNT; idx++) {
    const cell = grid[idx];

    if (cell === PREDATOR) {
      energy[idx] -= 1;
      if (energy[idx] <= 0) {
        grid[idx] = EMPTY;
        predatorCount--;
        predStarved++;
        continue;
      }

      const neighbors = getNeighborIndices(idx);
      let ate = false;
      for (const nIdx of neighbors) {
        if (grid[nIdx] === PREY && Math.random() < GOD.params.predEatChance.val) {
          grid[nIdx] = EMPTY;
          preyCount--;
          preyDied++;
          energy[idx] = GOD.params.maxEnergy;
          ate = true;
          predAte++;
          break;
        }
      }
    }

    if (cell === PREY) {
      if (Math.random() < GOD.params.preyReproduce.val && preyCount < TARGET_CELL_COUNT * 0.8) {
        const neighbors = getNeighborIndices(idx);
        for (const nIdx of neighbors) {
          if (grid[nIdx] === EMPTY) {
            grid[nIdx] = PREY;
            preyCount++;
            preyBorn++;
            break;
          }
        }
      }
    }
  }

  return { preyBorn, preyDied, predBorn, predStarved, predAte };
}

// Initialize with target count
const targetPrey = Math.floor(TARGET_CELL_COUNT * 0.65);
const targetPred = Math.floor(TARGET_CELL_COUNT * 0.35);

// Spawn in clusters
const numClusters = 25;
const preyPerCluster = Math.floor(targetPrey / numClusters);

for (let c = 0; c < numClusters; c++) {
  const cx = Math.floor(WIDTH * 0.4 + Math.random() * WIDTH * 0.55);
  const cy = Math.floor(Math.random() * HEIGHT);
  const clusterRadius = 15 + Math.random() * 20;

  for (let i = 0; i < preyPerCluster; i++) {
    const angle = Math.random() * Math.PI * 2;
    const dist = Math.random() * clusterRadius;
    const x = Math.floor(cx + Math.cos(angle) * dist);
    const y = Math.floor(cy + Math.sin(angle) * dist);
    if (x >= 0 && x < WIDTH && y >= 0 && y < HEIGHT) {
      const idx = y * WIDTH + x;
      if (grid[idx] === EMPTY) {
        grid[idx] = PREY;
        preyCount++;
      }
    }
  }
}

for (let i = 0; i < targetPred; i++) {
  const x = Math.floor(WIDTH * 0.4 + Math.random() * WIDTH * 0.55);
  const y = Math.floor(Math.random() * HEIGHT);
  const idx = y * WIDTH + x;
  if (grid[idx] === EMPTY) {
    grid[idx] = PREDATOR;
    energy[idx] = GOD.params.maxEnergy;
    predatorCount++;
  }
}

// Parse command line args
const args = process.argv.slice(2);
const duration = args[0] ? parseInt(args[0]) : 1200; // Default 20 sec @ 60fps
const reportInterval = args[1] ? parseInt(args[1]) : 100;

console.log("=== QUANTUM WAVE-PARTICLE ECOSYSTEM TEST ===");
console.log("Duration: " + duration + " frames (" + (duration/60).toFixed(1) + " seconds @ 60fps)");
console.log("Initial: prey=" + preyCount + ", pred=" + predatorCount + ", total=" + (preyCount + predatorCount));
console.log("Target: " + TARGET_CELL_COUNT);
console.log("");

const startTime = Date.now();

for (let frame = 0; frame < duration; frame++) {
  const stats = update();

  if (frame % reportInterval === 0 || preyCount + predatorCount === 0) {
    const total = preyCount + predatorCount;
    const preyPct = total > 0 ? (preyCount / total * 100).toFixed(1) : 0;
    const avgPreyWave = preyWaves.reduce((s, w) => s + w.amplitude, 0) / preyWaves.length;
    const avgPredWave = predatorWaves.reduce((s, w) => s + w.amplitude, 0) / predatorWaves.length;

    console.log("Frame " + frame + ": prey=" + preyCount + ", pred=" + predatorCount +
                ", total=" + total + ", prey%=" + preyPct + "%");
    console.log("  Ψ_prey=" + avgPreyWave.toFixed(3) + ", Ψ_pred=" + avgPredWave.toFixed(3) +
                ", κ=" + GOD.bhardwajConstant.toFixed(3) +
                ", sample=" + (sampleRate*100).toFixed(1) + "%");
    console.log("  births:prey+" + stats.preyBorn + " | deaths:eaten=" + stats.preyDied +
                " starved=" + stats.predStarved + " | fed=" + stats.predAte);

    if (total === 0) {
      console.log("\n!!! EXTINCTION at frame " + frame + " !!!");
      break;
    }
  }
}

const endTime = Date.now();
const elapsed = ((endTime - startTime) / 1000).toFixed(2);

console.log("\n=== FINAL RESULTS ===");
console.log("Prey: " + preyCount + ", Predators: " + predatorCount);
console.log("Total: " + (preyCount + predatorCount) + " / " + TARGET_CELL_COUNT);
console.log("Prey ratio: " + (preyCount / (preyCount + predatorCount) * 100).toFixed(1) + "%");
console.log("Elapsed time: " + elapsed + " seconds");
console.log("");

// Check bounds
const avgPreyWave = preyWaves.reduce((s, w) => s + w.amplitude, 0) / preyWaves.length;
const avgPredWave = predatorWaves.reduce((s, w) => s + w.amplitude, 0) / predatorWaves.length;

console.log("=== PARAMETER STABILITY CHECK ===");
console.log("Prey wave amplitude: " + avgPreyWave.toFixed(3) + " (should be in [0.3, 2.0])");
console.log("Pred wave amplitude: " + avgPredWave.toFixed(3) + " (should be in [0.3, 2.0])");
console.log("Bhardwaj κ: " + GOD.bhardwajConstant.toFixed(3) + " (should be in [0.7, 0.95])");
console.log("Pred eat chance: " + GOD.params.predEatChance.val.toFixed(3) + " (should be in [0.3, 0.95])");

const preyWaveOK = avgPreyWave >= 0.3 && avgPreyWave <= 2.0;
const predWaveOK = avgPredWave >= 0.3 && avgPredWave <= 2.0;
const bhardwajOK = GOD.bhardwajConstant >= 0.7 && GOD.bhardwajConstant <= 0.95;

if (preyWaveOK && predWaveOK && bhardwajOK && preyCount > 0 && predatorCount > 0) {
  console.log("\n✓ PASS: All parameters bounded, no extinction");
} else {
  console.log("\n✗ FAIL: Parameters out of bounds or extinction occurred");
}
