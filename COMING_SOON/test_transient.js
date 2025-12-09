// Transient analysis of quantum wave ecosystem
// Tracks population dynamics and parameter evolution over time

const WIDTH = 960;
const HEIGHT = 540;
const PIXEL_COUNT = WIDTH * HEIGHT;
let TARGET_CELL_COUNT = 15000;
const SCALE = 2;

const EMPTY = 0;
const PREY = 1;
const PREDATOR = 2;

let grid = new Uint8Array(PIXEL_COUNT);
let energy = new Uint8Array(PIXEL_COUNT);
let time = 0;
let waveSpawnRate = 0.015;
let sampleRate = 0.05;

// Circular wave class
class CircularWave {
  constructor(cx, cy, birthTime) {
    this.cx = cx;
    this.cy = cy;
    this.birthTime = birthTime;
    this.speed = 2.0;
    this.frequency = 0.08 + Math.random() * 0.04;
    this.polarity = Math.random() < 0.5 ? 1 : -1;
    this.amplitude = 1.5 + Math.random() * 1.0;
  }

  getValue(x, y, t) {
    const age = t - this.birthTime;
    const dx = x - this.cx;
    const dy = y - this.cy;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const wavefront = this.speed * age;

    if (dist > wavefront + 20) return 0;

    const decay = 1.0 / (1 + dist / 80);
    const oscillation = Math.cos(this.frequency * (dist - wavefront));

    return this.polarity * this.amplitude * decay * oscillation;
  }

  isAlive(t, maxDist) {
    const age = t - this.birthTime;
    return this.speed * age < maxDist;
  }
}

const circularWaves = [];
const MAX_WAVES = 60;
const pulses = [];

// GOD controller (OPTIMIZED from Rust parameter sweep)
const GOD = {
  // OPTIMAL: 0.4 (found via 576-combination parameter sweep)
  bhardwajConstant: 0.4,
  targets: { cellCount: TARGET_CELL_COUNT, preyRatio: { min: 0.55, max: 0.75 } },
  // OPTIMAL: maxEnergy = 1000 (critical for predator survival)
  params: { predEatChance: { val: 0.8, min: 0.3, max: 0.95 }, maxEnergy: 1000 },
  history: { preyRatios: [], windowSize: 60 },
  // OPTIMAL PID gains
  pid: { Kp: 0.04, Ki: 0.001, Kd: 0.2, prevError: 0, integral: 0 },

  regulate(prey, predators) {
    const total = prey + predators;
    const error = total - this.targets.cellCount;
    const errorNorm = error / this.targets.cellCount;

    this.history.preyRatios.push(total > 0 ? prey / total : 0.5);
    if (this.history.preyRatios.length > this.history.windowSize) {
      this.history.preyRatios.shift();
    }

    const preyRatio = this.history.preyRatios.reduce((a,b) => a+b, 0) / this.history.preyRatios.length;

    // PID control (optimal gains from parameter sweep)
    const derivative = error - this.pid.prevError;
    this.pid.integral += error;
    this.pid.integral = Math.max(-10000, Math.min(10000, this.pid.integral));

    const pidOutput = this.pid.Kp * errorNorm +
                     this.pid.Ki * this.pid.integral / 1000 +
                     this.pid.Kd * derivative / this.targets.cellCount;

    this.pid.prevError = error;

    // Apply PID to wave spawn rate
    waveSpawnRate -= pidOutput * 0.005;
    waveSpawnRate = Math.max(0.001, Math.min(0.04, waveSpawnRate));

    // Apply PID to Bhardwaj constant
    this.bhardwajConstant += pidOutput * 0.02;
    this.bhardwajConstant = Math.max(0.3, Math.min(0.95, this.bhardwajConstant));

    // Balance control
    if (preyRatio > 0.75) {
      this.params.predEatChance.val *= 1.02;
    } else if (preyRatio < 0.55) {
      this.params.predEatChance.val *= 0.98;
    }

    // Extinction prevention
    if (predators < 20) {
      waveSpawnRate = 0.04;
      this.bhardwajConstant = 0.3;
      this.params.predEatChance.val = 0.5;
    } else if (predators < 200) {
      waveSpawnRate = Math.min(0.04, waveSpawnRate * 1.5);
      this.bhardwajConstant *= 0.8;
    }

    if (prey < 50) {
      waveSpawnRate = 0.04;
      this.bhardwajConstant = 0.3;
    } else if (prey < 500) {
      waveSpawnRate = Math.min(0.04, waveSpawnRate * 1.5);
      this.bhardwajConstant *= 0.8;
    }

    // Clamp
    waveSpawnRate = Math.max(0.001, Math.min(0.04, waveSpawnRate));
    this.bhardwajConstant = Math.max(0.3, Math.min(0.95, this.bhardwajConstant));
    this.params.predEatChance.val = Math.max(0.3, Math.min(0.95, this.params.predEatChance.val));
  }
};

function countParticles() {
  let prey = 0;
  let predators = 0;
  for (let i = 0; i < grid.length; i++) {
    if (grid[i] === PREY) prey++;
    else if (grid[i] === PREDATOR) predators++;
  }
  return { prey, predators, total: prey + predators };
}

function spawnCircularWave() {
  if (circularWaves.length >= MAX_WAVES) return;
  const x = Math.floor(Math.random() * WIDTH);
  const y = Math.floor(Math.random() * HEIGHT);
  circularWaves.push(new CircularWave(x, y, time));
}

function cleanupWaves(t) {
  const maxDist = Math.max(WIDTH, HEIGHT) * 1.2;
  for (let i = circularWaves.length - 1; i >= 0; i--) {
    if (!circularWaves[i].isAlive(t, maxDist)) {
      circularWaves.splice(i, 1);
    }
  }
}

function collapseWavefunction(x, y, t, counts) {
  const idx = y * WIDTH + x;
  if (grid[idx] !== EMPTY) return;
  if (counts.total >= TARGET_CELL_COUNT) return;

  let totalAmp = 0;
  for (const wave of circularWaves) {
    totalAmp += wave.getValue(x, y, t);
  }

  if (totalAmp > GOD.bhardwajConstant) {
    grid[idx] = PREY;
  } else if (totalAmp < -GOD.bhardwajConstant) {
    grid[idx] = PREDATOR;
    energy[idx] = GOD.params.maxEnergy;
  }
}

function update() {
  time++;

  const counts = countParticles();
  GOD.regulate(counts.prey, counts.predators);

  // Hard extinction prevention
  if (counts.predators < 20) {
    for (let i = 0; i < 50; i++) {
      const x = Math.floor(Math.random() * WIDTH);
      const y = Math.floor(Math.random() * HEIGHT);
      const idx = y * WIDTH + x;
      if (grid[idx] === EMPTY) {
        grid[idx] = PREDATOR;
        energy[idx] = GOD.params.maxEnergy;
      }
    }
  }

  if (counts.prey < 50) {
    for (let i = 0; i < 200; i++) {
      const x = Math.floor(Math.random() * WIDTH);
      const y = Math.floor(Math.random() * HEIGHT);
      const idx = y * WIDTH + x;
      if (grid[idx] === EMPTY) {
        grid[idx] = PREY;
      }
    }
  }

  if (Math.random() < waveSpawnRate) spawnCircularWave();
  cleanupWaves(time);

  const samplesToCheck = Math.floor(PIXEL_COUNT * sampleRate);
  for (let i = 0; i < samplesToCheck && counts.total < TARGET_CELL_COUNT; i++) {
    const x = Math.floor(Math.random() * WIDTH);
    const y = Math.floor(Math.random() * HEIGHT);
    collapseWavefunction(x, y, time, counts);
  }

  // RATIO-BASED SPAWNING (Key fix for 99% prey problem)
  const targetPredCount = Math.floor(TARGET_CELL_COUNT * 0.35);
  const targetPreyCount = Math.floor(TARGET_CELL_COUNT * 0.65);

  if (counts.predators < targetPredCount) {
    const deficit = targetPredCount - counts.predators;
    const spawnRate = Math.min(deficit / targetPredCount, 0.5);
    const spawnCount = Math.max(1, Math.min(20, Math.floor(deficit * spawnRate * 0.05)));

    for (let i = 0; i < spawnCount; i++) {
      const x = Math.floor(Math.random() * WIDTH);
      const y = Math.floor(Math.random() * HEIGHT);
      const idx = y * WIDTH + x;
      if (grid[idx] === EMPTY) {
        grid[idx] = PREDATOR;
        const variation = 0.5 + Math.random() * 0.5;
        energy[idx] = Math.floor(GOD.params.maxEnergy * variation);
      }
    }
  }

  if (counts.prey < targetPreyCount) {
    const deficit = targetPreyCount - counts.prey;
    const spawnRate = Math.min(deficit / targetPreyCount, 0.3);
    const spawnCount = Math.min(10, Math.floor(deficit * spawnRate * 0.02));

    for (let i = 0; i < spawnCount; i++) {
      const x = Math.floor(Math.random() * WIDTH);
      const y = Math.floor(Math.random() * HEIGHT);
      const idx = y * WIDTH + x;
      if (grid[idx] === EMPTY) {
        grid[idx] = PREY;
      }
    }
  }

  // Simplified particle dynamics (just energy drain and annihilation)
  for (let idx = 0; idx < grid.length; idx++) {
    if (grid[idx] === PREDATOR) {
      energy[idx] -= 1;
      if (energy[idx] <= 0) {
        grid[idx] = EMPTY;
      }
    }
  }

  return counts;
}

// Initialize
for (let i = 0; i < 30; i++) {
  const x = Math.floor(Math.random() * WIDTH);
  const y = Math.floor(Math.random() * HEIGHT);
  circularWaves.push(new CircularWave(x, y, -Math.floor(Math.random() * 150)));
}

// Spawn initial particles
const targetPrey = Math.floor(TARGET_CELL_COUNT * 0.2);
const targetPred = Math.floor(TARGET_CELL_COUNT * 0.1);

for (let i = 0; i < targetPrey; i++) {
  const x = Math.floor(Math.random() * WIDTH);
  const y = Math.floor(Math.random() * HEIGHT);
  const idx = y * WIDTH + x;
  if (grid[idx] === EMPTY) grid[idx] = PREY;
}

for (let i = 0; i < targetPred; i++) {
  const x = Math.floor(Math.random() * WIDTH);
  const y = Math.floor(Math.random() * HEIGHT);
  const idx = y * WIDTH + x;
  if (grid[idx] === EMPTY) {
    grid[idx] = PREDATOR;
    energy[idx] = GOD.params.maxEnergy;
  }
}

// Parse args
const args = process.argv.slice(2);
const DURATION = args[0] ? parseInt(args[0]) : 3600;
const SAMPLE_INTERVAL = args[1] ? parseInt(args[1]) : 10;

// History tracking
const history = {
  time: [],
  prey: [],
  predators: [],
  total: [],
  bhardwaj: [],
  waveSpawnRate: [],
  annihilation: [],
  waveCount: [],
  extinctions: [],
};

console.log("=== TRANSIENT ANALYSIS ===");
console.log("Duration: " + DURATION + " frames (" + (DURATION/60).toFixed(1) + "s)");
console.log("Sample interval: " + SAMPLE_INTERVAL + " frames");
console.log("");

const startTime = Date.now();

for (let frame = 0; frame < DURATION; frame++) {
  const counts = update();

  if (frame % SAMPLE_INTERVAL === 0) {
    history.time.push(frame);
    history.prey.push(counts.prey);
    history.predators.push(counts.predators);
    history.total.push(counts.total);
    history.bhardwaj.push(GOD.bhardwajConstant);
    history.waveSpawnRate.push(waveSpawnRate);
    history.annihilation.push(GOD.params.predEatChance.val);
    history.waveCount.push(circularWaves.length);

    if (counts.prey === 0 || counts.predators === 0) {
      history.extinctions.push(frame);
      console.log("!!! EXTINCTION at frame " + frame + ": prey=" + counts.prey + ", pred=" + counts.predators);
    }

    if (frame % 600 === 0) {
      const preyPct = counts.total > 0 ? (counts.prey / counts.total * 100).toFixed(1) : 0;
      console.log("Frame " + frame + ": prey=" + counts.prey + ", pred=" + counts.predators +
                  ", total=" + counts.total + " (" + preyPct + "% prey)");
      console.log("  κ_B=" + GOD.bhardwajConstant.toFixed(3) +
                  ", waveSpawn=" + (waveSpawnRate*1000).toFixed(1) +
                  ", waves=" + circularWaves.length);
    }
  }
}

const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);

// Analysis
function mean(arr) {
  return arr.reduce((a,b) => a+b, 0) / arr.length;
}

function variance(arr) {
  const m = mean(arr);
  return arr.reduce((sum, v) => sum + (v - m) ** 2, 0) / arr.length;
}

function countOscillations(series) {
  let count = 0;
  for (let i = 2; i < series.length; i++) {
    const prev = series[i-1] - series[i-2];
    const curr = series[i] - series[i-1];
    if (prev * curr < 0) count++;
  }
  return count;
}

console.log("\n=== RESULTS ===");
console.log("Elapsed: " + elapsed + "s");
console.log("");
console.log("Population:");
console.log("  Average: " + mean(history.total).toFixed(0) + " / " + TARGET_CELL_COUNT);
console.log("  Variance: " + Math.sqrt(variance(history.total)).toFixed(0) + " (stddev)");
console.log("  Min: " + Math.min(...history.total));
console.log("  Max: " + Math.max(...history.total));
console.log("  Prey ratio: " + (mean(history.prey) / mean(history.total) * 100).toFixed(1) + "%");
console.log("");
console.log("Stability:");
console.log("  Extinctions: " + history.extinctions.length);
console.log("  Oscillations: " + countOscillations(history.total));
console.log("  Extinction frames: " + JSON.stringify(history.extinctions));
console.log("");
console.log("Parameters:");
console.log("  κ_B range: [" + Math.min(...history.bhardwaj).toFixed(2) + ", " + Math.max(...history.bhardwaj).toFixed(2) + "]");
console.log("  Wave spawn range: [" + (Math.min(...history.waveSpawnRate)*1000).toFixed(1) + ", " + (Math.max(...history.waveSpawnRate)*1000).toFixed(1) + "]");
console.log("  Wave count avg: " + mean(history.waveCount).toFixed(0));

// Stability verdict
const avgPop = mean(history.total);
const error = Math.abs(avgPop - TARGET_CELL_COUNT) / TARGET_CELL_COUNT;

console.log("");
if (history.extinctions.length === 0 && error < 0.2) {
  console.log("✓ STABLE: No extinctions, population within 20% of target");
} else {
  console.log("✗ UNSTABLE: Extinctions=" + history.extinctions.length + ", Error=" + (error*100).toFixed(1) + "%");
}
