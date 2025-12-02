//! Self-organized criticality models
//!
//! Implements sandpile and forest fire models that exhibit power law avalanche distributions.

use rand::Rng;

/// Cell state for forest fire model
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CellState {
    Empty = 0,
    Tree = 1,
    Burning = 2,
}

/// Bak-Tang-Wiesenfeld sandpile model
pub struct Sandpile<const W: usize, const H: usize> {
    /// Grains per cell
    pub grains: [[u8; W]; H],
    /// Topple queue (stack-allocated)
    topple_queue: Vec<(usize, usize)>,
    queue_head: usize,
    queue_tail: usize,
}

impl<const W: usize, const H: usize> Sandpile<W, H> {
    pub fn new() -> Self {
        Self {
            grains: [[0; W]; H],
            topple_queue: vec![(0, 0); W * H],
            queue_head: 0,
            queue_tail: 0,
        }
    }

    /// Drop grain and run to stability, returns avalanche size
    pub fn drop_grain(&mut self, x: usize, y: usize) -> usize {
        if x >= W || y >= H {
            return 0;
        }

        self.grains[y][x] = self.grains[y][x].saturating_add(1);

        let mut size = 0;
        self.queue_head = 0;
        self.queue_tail = 0;

        if self.grains[y][x] >= 4 {
            self.topple_queue[0] = (x, y);
            self.queue_tail = 1;
        }

        while self.queue_head < self.queue_tail {
            let batch_end = self.queue_tail;

            while self.queue_head < batch_end {
                let (cx, cy) = self.topple_queue[self.queue_head];
                self.queue_head += 1;

                if self.grains[cy][cx] >= 4 {
                    self.topple(cx, cy);
                    size += 1;
                }
            }
        }

        size
    }

    fn topple(&mut self, x: usize, y: usize) {
        self.grains[y][x] = self.grains[y][x].saturating_sub(4);

        // Distribute to neighbors
        if x > 0 {
            self.grains[y][x - 1] = self.grains[y][x - 1].saturating_add(1);
            if self.grains[y][x - 1] >= 4 && self.queue_tail < W * H {
                self.topple_queue[self.queue_tail] = (x - 1, y);
                self.queue_tail += 1;
            }
        }
        if x < W - 1 {
            self.grains[y][x + 1] = self.grains[y][x + 1].saturating_add(1);
            if self.grains[y][x + 1] >= 4 && self.queue_tail < W * H {
                self.topple_queue[self.queue_tail] = (x + 1, y);
                self.queue_tail += 1;
            }
        }
        if y > 0 {
            self.grains[y - 1][x] = self.grains[y - 1][x].saturating_add(1);
            if self.grains[y - 1][x] >= 4 && self.queue_tail < W * H {
                self.topple_queue[self.queue_tail] = (x, y - 1);
                self.queue_tail += 1;
            }
        }
        if y < H - 1 {
            self.grains[y + 1][x] = self.grains[y + 1][x].saturating_add(1);
            if self.grains[y + 1][x] >= 4 && self.queue_tail < W * H {
                self.topple_queue[self.queue_tail] = (x, y + 1);
                self.queue_tail += 1;
            }
        }
    }

    pub fn state(&self) -> &[[u8; W]; H] {
        &self.grains
    }
}

impl<const W: usize, const H: usize> Default for Sandpile<W, H> {
    fn default() -> Self {
        Self::new()
    }
}

/// Forest fire model
pub struct ForestFire<const W: usize, const H: usize> {
    pub cells: [[CellState; W]; H],
    next_cells: [[CellState; W]; H],
    pub tree_growth_prob: f64,
    pub lightning_prob: f64,
}

impl<const W: usize, const H: usize> ForestFire<W, H> {
    pub fn new(tree_growth_prob: f64, lightning_prob: f64) -> Self {
        Self {
            cells: [[CellState::Empty; W]; H],
            next_cells: [[CellState::Empty; W]; H],
            tree_growth_prob,
            lightning_prob,
        }
    }

    pub fn step(&mut self, rng: &mut impl Rng) -> usize {
        let mut trees_burned = 0;

        for y in 0..H {
            for x in 0..W {
                match self.cells[y][x] {
                    CellState::Empty => {
                        self.next_cells[y][x] = if rng.gen::<f64>() < self.tree_growth_prob {
                            CellState::Tree
                        } else {
                            CellState::Empty
                        };
                    }
                    CellState::Tree => {
                        let neighbor_burning = self.has_burning_neighbor(x, y);
                        let lightning = rng.gen::<f64>() < self.lightning_prob;

                        if neighbor_burning || lightning {
                            self.next_cells[y][x] = CellState::Burning;
                            trees_burned += 1;
                        } else {
                            self.next_cells[y][x] = CellState::Tree;
                        }
                    }
                    CellState::Burning => {
                        self.next_cells[y][x] = CellState::Empty;
                    }
                }
            }
        }

        std::mem::swap(&mut self.cells, &mut self.next_cells);
        trees_burned
    }

    fn has_burning_neighbor(&self, x: usize, y: usize) -> bool {
        let check = |nx: usize, ny: usize| -> bool {
            nx < W && ny < H && self.cells[ny][nx] == CellState::Burning
        };

        (x > 0 && check(x - 1, y))
            || (x < W - 1 && check(x + 1, y))
            || (y > 0 && check(x, y - 1))
            || (y < H - 1 && check(x, y + 1))
    }

    pub fn state(&self) -> &[[CellState; W]; H] {
        &self.cells
    }
}

impl<const W: usize, const H: usize> Default for ForestFire<W, H> {
    fn default() -> Self {
        Self::new(0.05, 0.0001)
    }
}
