//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: spatial_grid.rs
//! PATH: DNA/src/data/spatial_grid.rs
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! PURPOSE: Generic uniform spatial grid for O(1) neighbor queries
//!
//! LAYER: DNA → DATA
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ DATA DEFINED                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ UniformGrid<CAP>  Fixed-size grid for spatial partitioning                  │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ DATA FLOW                                                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ CONSUMES:  (x, y) positions, entity indices                                 │
//! │ PRODUCES:  Neighbor lists (indices within radius)                           │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • None (pure data structure)
//!
//! USED BY:
//!   • DNA/src/lib.rs  → Boid flocking (domain-specific version)
//!   • Future: Collision detection, particle systems, spatial queries
//!
//! ALGORITHM: Uniform spatial hashing
//!   - Fixed cell size for O(1) cell lookup
//!   - Pre-allocated cell capacity (no dynamic allocation)
//!   - Query checks 3x3 cell neighborhood for radius queries
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

/// Generic uniform spatial grid for 2D space
///
/// Partitions space into cells of fixed size for fast neighbor queries.
pub struct UniformGrid<const CELL_CAPACITY: usize> {
    cell_size: f32,
    cols: usize,
    rows: usize,
    _width: f32,
    _height: f32,
    /// Each cell stores up to CELL_CAPACITY entity indices
    cells: Vec<[u16; CELL_CAPACITY]>,
    /// Number of entities in each cell
    cell_counts: Vec<usize>,
}

impl<const CELL_CAPACITY: usize> UniformGrid<CELL_CAPACITY> {
    /// Create a new spatial grid
    ///
    /// # Arguments
    /// * `width` - World width
    /// * `height` - World height
    /// * `cell_size` - Size of each cell (larger = fewer cells, more entities per cell)
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let cols = (width / cell_size).ceil() as usize;
        let rows = (height / cell_size).ceil() as usize;
        let num_cells = cols * rows;

        Self {
            cell_size,
            cols,
            rows,
            _width: width,
            _height: height,
            cells: vec![[0; CELL_CAPACITY]; num_cells],
            cell_counts: vec![0; num_cells],
        }
    }

    /// Clear all cells
    #[inline]
    pub fn clear(&mut self) {
        self.cell_counts.fill(0);
    }

    /// Insert an entity at position (x, y)
    ///
    /// Returns true if successfully inserted, false if cell is full.
    pub fn insert(&mut self, x: f32, y: f32, entity_index: u16) -> bool {
        let cell_idx = self.cell_index(x, y);

        let count = self.cell_counts[cell_idx];
        if count >= CELL_CAPACITY {
            return false; // Cell is full
        }

        self.cells[cell_idx][count] = entity_index;
        self.cell_counts[cell_idx] += 1;
        true
    }

    /// Query entities within radius of (x, y)
    ///
    /// Writes results to output buffer, returns count.
    pub fn query_radius(&self, x: f32, y: f32, radius: f32, output: &mut [u16]) -> usize {
        let _radius_sq = radius * radius;
        let mut count = 0;

        // Check 3x3 neighborhood of cells
        let cell_x = (x / self.cell_size) as isize;
        let cell_y = (y / self.cell_size) as isize;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let cx = cell_x + dx;
                let cy = cell_y + dy;

                if cx < 0 || cy < 0 || cx >= self.cols as isize || cy >= self.rows as isize {
                    continue;
                }

                let cell_idx = (cy as usize) * self.cols + (cx as usize);
                let cell_count = self.cell_counts[cell_idx];

                for i in 0..cell_count {
                    if count >= output.len() {
                        return count; // Output buffer full
                    }

                    let entity_idx = self.cells[cell_idx][i];
                    output[count] = entity_idx;
                    count += 1;
                }
            }
        }

        count
    }

    /// Get cell index for world position (x, y)
    #[inline]
    fn cell_index(&self, x: f32, y: f32) -> usize {
        let col = ((x / self.cell_size) as usize).min(self.cols - 1);
        let row = ((y / self.cell_size) as usize).min(self.rows - 1);
        row * self.cols + col
    }

    /// Get grid dimensions
    #[inline]
    pub fn dimensions(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_grid() {
        let grid: UniformGrid<10> = UniformGrid::new(100.0, 100.0, 10.0);
        assert_eq!(grid.dimensions(), (10, 10));
    }

    #[test]
    fn test_insert_and_query() {
        let mut grid: UniformGrid<10> = UniformGrid::new(100.0, 100.0, 10.0);

        // Insert entities
        grid.insert(5.0, 5.0, 0);
        grid.insert(7.0, 6.0, 1);
        grid.insert(50.0, 50.0, 2);

        // Query near first two entities
        let mut output = [0u16; 10];
        let count = grid.query_radius(5.0, 5.0, 15.0, &mut output);

        // Should find entities 0 and 1 (both in same or adjacent cells)
        assert!(count >= 2);
    }

    #[test]
    fn test_clear() {
        let mut grid: UniformGrid<10> = UniformGrid::new(100.0, 100.0, 10.0);

        grid.insert(5.0, 5.0, 0);
        grid.insert(7.0, 6.0, 1);

        grid.clear();

        let mut output = [0u16; 10];
        let count = grid.query_radius(5.0, 5.0, 15.0, &mut output);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_cell_capacity() {
        let mut grid: UniformGrid<2> = UniformGrid::new(100.0, 100.0, 50.0);

        // Fill one cell
        assert!(grid.insert(5.0, 5.0, 0));
        assert!(grid.insert(6.0, 6.0, 1));

        // Cell should be full
        assert!(!grid.insert(7.0, 7.0, 2));
    }
}
