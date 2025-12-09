// Simulate the ecosystem without visuals
// MATCHES index.html exactly

const WIDTH = 960;  // Typical screen width / 2
const HEIGHT = 540; // Typical screen height / 2
const PIXEL_COUNT = WIDTH * HEIGHT;
const TARGET_DENSITY = 0.15;
let MAX_CELLS = Math.floor(PIXEL_COUNT * TARGET_DENSITY);

const EMPTY = 0;
const PREY = 1;
const PREDATOR = 2;

// TUNED PARAMETERS - more aggressive predators
const MAX_ENERGY = 200;
const PREDATOR_MOVE_CHANCE = 0.15;      // Move more often toward prey
const PREY_REPRODUCE_CHANCE = 0.04;     // Slower prey growth
const PREDATOR_REPRODUCE_CHANCE = 0.05; // More predator babies
const PREDATOR_ENERGY_DRAIN = 1;
const PREDATOR_EAT_CHANCE = 0.7;        // Higher catch rate

let grid = new Uint8Array(PIXEL_COUNT);
let energy = new Uint8Array(PIXEL_COUNT);
let preyCount = 0;
let predatorCount = 0;
let deathEvents = 0;
let bloomEvents = 0;

// Initialize with CLUSTERS - matching index.html
const numPreyClusters = 20;
const preyPerCluster = Math.floor(MAX_CELLS * 0.4 / numPreyClusters);
const initialPredators = Math.floor(MAX_CELLS * 0.15);

// Spawn prey in clusters (right side of screen, avoiding left panel)
for (let c = 0; c < numPreyClusters; c++) {
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

// Spawn predators scattered
for (let i = 0; i < initialPredators; i++) {
  const x = Math.floor(WIDTH * 0.4 + Math.random() * WIDTH * 0.55);
  const y = Math.floor(Math.random() * HEIGHT);
  const idx = y * WIDTH + x;
  if (grid[idx] === EMPTY) {
    grid[idx] = PREDATOR;
    energy[idx] = MAX_ENERGY;
    predatorCount++;
  }
}

console.log("Initial: prey=" + preyCount + ", predators=" + predatorCount + ", MAX_CELLS=" + MAX_CELLS);
console.log("Density target: " + (TARGET_DENSITY * 100).toFixed(1) + "%");
console.log("---");

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
  const cx = Math.floor(Math.random() * WIDTH);
  const cy = Math.floor(Math.random() * HEIGHT);
  const radius = 30 + Math.random() * 40;
  let killed = 0;

  for (let dy = -radius; dy <= radius; dy++) {
    for (let dx = -radius; dx <= radius; dx++) {
      if (dx * dx + dy * dy <= radius * radius) {
        const nx = Math.floor(cx + dx);
        const ny = Math.floor(cy + dy);
        if (nx >= 0 && nx < WIDTH && ny >= 0 && ny < HEIGHT) {
          const idx = ny * WIDTH + nx;
          if (grid[idx] === PREY) { preyCount--; killed++; }
          else if (grid[idx] === PREDATOR) { predatorCount--; killed++; }
          grid[idx] = EMPTY;
        }
      }
    }
  }
  deathEvents++;
  console.log("  DEATH FLASH: killed " + killed + " cells");
  return killed;
}

function triggerBloomFlash() {
  const cx = Math.floor(Math.random() * WIDTH);
  const cy = Math.floor(Math.random() * HEIGHT);
  const radius = 25 + Math.random() * 35;
  let spawnedPrey = 0;
  let spawnedPred = 0;

  for (let dy = -radius; dy <= radius; dy++) {
    for (let dx = -radius; dx <= radius; dx++) {
      if (dx * dx + dy * dy <= radius * radius) {
        const nx = Math.floor(cx + dx);
        const ny = Math.floor(cy + dy);
        if (nx >= 0 && nx < WIDTH && ny >= 0 && ny < HEIGHT) {
          const idx = ny * WIDTH + nx;
          if (grid[idx] === EMPTY && Math.random() < 0.4) {
            grid[idx] = PREY;
            preyCount++;
            spawnedPrey++;
          }
        }
      }
    }
  }

  // Spawn predators - proportional to prey spawned
  const predatorsToSpawn = Math.floor(spawnedPrey * 0.15); // 15% of prey count
  for (let i = 0; i < predatorsToSpawn; i++) {
    const angle = Math.random() * Math.PI * 2;
    const dist = Math.random() * radius;
    const nx = Math.floor(cx + Math.cos(angle) * dist);
    const ny = Math.floor(cy + Math.sin(angle) * dist);
    if (nx >= 0 && nx < WIDTH && ny >= 0 && ny < HEIGHT) {
      const idx = ny * WIDTH + nx;
      if (grid[idx] === EMPTY) {
        grid[idx] = PREDATOR;
        energy[idx] = MAX_ENERGY;
        predatorCount++;
        spawnedPred++;
      }
    }
  }
  bloomEvents++;
  console.log("  BLOOM FLASH: spawned " + spawnedPrey + " prey, " + spawnedPred + " predators");
  return spawnedPrey + spawnedPred;
}

function update() {
  const totalCells = preyCount + predatorCount;
  const density = totalCells / PIXEL_COUNT;

  // Population control - EXACT from index.html
  if (density > 0.18 && Math.random() < 0.05) triggerDeathFlash();
  if (density < 0.12 && Math.random() < 0.04) triggerBloomFlash();

  // If predators extinct, force bloom
  if (predatorCount < 10 && Math.random() < 0.1) triggerBloomFlash();

  // Random events
  if (Math.random() < 0.003) {
    if (Math.random() < 0.5) triggerDeathFlash();
    else triggerBloomFlash();
  }

  let preyBorn = 0;
  let preyDied = 0;
  let predBorn = 0;
  let predStarved = 0;
  let predAte = 0;

  // Process cells
  for (let idx = 0; idx < PIXEL_COUNT; idx++) {
    const cell = grid[idx];

    if (cell === PREDATOR) {
      energy[idx] -= PREDATOR_ENERGY_DRAIN;

      if (energy[idx] <= 0) {
        grid[idx] = EMPTY;
        predatorCount--;
        predStarved++;
        continue;
      }

      // Try to eat adjacent prey (matching index.html exactly)
      const x = idx % WIDTH;
      const y = Math.floor(idx / WIDTH);
      const neighbors = getNeighborIndices(idx);
      let ate = false;

      for (const nIdx of neighbors) {
        if (grid[nIdx] === PREY && Math.random() < PREDATOR_EAT_CHANCE) {
          // Eat prey
          grid[nIdx] = EMPTY;
          preyCount--;
          preyDied++;
          energy[idx] = MAX_ENERGY;
          ate = true;
          predAte++;

          // Chance to reproduce
          if (Math.random() < PREDATOR_REPRODUCE_CHANCE && predatorCount < MAX_CELLS * 0.4) {
            for (const n2 of neighbors) {
              if (grid[n2] === EMPTY) {
                grid[n2] = PREDATOR;
                energy[n2] = Math.floor(MAX_ENERGY * 0.7);
                predatorCount++;
                predBorn++;
                break;
              }
            }
          }
          break;
        }
      }

      // Drift toward prey if didn't eat
      if (!ate && Math.random() < PREDATOR_MOVE_CHANCE) {
        // Find nearest prey in radius 15
        let nearest = null;
        let minDist = 15 * 15;
        for (let dy = -15; dy <= 15; dy++) {
          for (let dx = -15; dx <= 15; dx++) {
            const nx = x + dx;
            const ny = y + dy;
            if (nx >= 0 && nx < WIDTH && ny >= 0 && ny < HEIGHT) {
              const nIdx = ny * WIDTH + nx;
              if (grid[nIdx] === PREY) {
                const dist = dx * dx + dy * dy;
                if (dist < minDist) {
                  minDist = dist;
                  nearest = { x: nx, y: ny };
                }
              }
            }
          }
        }
        if (nearest) {
          const dx = Math.sign(nearest.x - x);
          const dy = Math.sign(nearest.y - y);
          const newX = x + dx;
          const newY = y + dy;
          if (newX >= 0 && newX < WIDTH && newY >= 0 && newY < HEIGHT) {
            const newIdx = newY * WIDTH + newX;
            if (grid[newIdx] === EMPTY) {
              grid[newIdx] = PREDATOR;
              energy[newIdx] = energy[idx];
              grid[idx] = EMPTY;
            }
          }
        }
      }
    }

    if (cell === PREY) {
      if (Math.random() < PREY_REPRODUCE_CHANCE && preyCount < MAX_CELLS * 0.8) {
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

// Run simulation - longer to check stability
console.log("\nRunning 1000 frames...\n");

for (let frame = 0; frame < 1000; frame++) {
  const stats = update();

  if (frame % 100 === 0 || preyCount + predatorCount === 0) {
    const total = preyCount + predatorCount;
    const density = (total / PIXEL_COUNT * 100).toFixed(2);
    const preyRatio = total > 0 ? (preyCount / total * 100).toFixed(1) : 0;
    console.log("Frame " + frame + ": prey=" + preyCount + ", pred=" + predatorCount +
                ", density=" + density + "%, prey%=" + preyRatio + "%");
    console.log("  births: prey+" + stats.preyBorn + " pred+" + stats.predBorn +
                " | deaths: eaten=" + stats.preyDied + " starved=" + stats.predStarved +
                " | fed=" + stats.predAte);

    if (preyCount + predatorCount === 0) {
      console.log("\nEXTINCTION at frame " + frame);
      break;
    }
  }
}

console.log("\n=== FINAL RESULTS ===");
console.log("Prey: " + preyCount + ", Predators: " + predatorCount);
console.log("Death events: " + deathEvents + ", Bloom events: " + bloomEvents);

console.log("\n=== PROBLEM ANALYSIS ===");
const avgBloomArea = Math.PI * 30 * 30;
const avgBloomSpawn = avgBloomArea * 0.4;
console.log("Avg bloom area: ~" + Math.floor(avgBloomArea) + " pixels");
console.log("Avg bloom spawn (40% fill): ~" + Math.floor(avgBloomSpawn) + " prey");
console.log("Predator lifespan without food: " + MAX_ENERGY + " frames");
console.log("Prey reproduce chance: " + (PREY_REPRODUCE_CHANCE * 100) + "% per frame");

if (predatorCount > preyCount * 2) {
  console.log("\nISSUE: Too many predators consuming prey faster than reproduction");
}
if (preyCount === 0) {
  console.log("\nISSUE: Prey extinct - predators eat faster than prey reproduce");
}
