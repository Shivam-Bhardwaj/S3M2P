// Pathfinding algorithms for grid-based navigation
// A*, D* Lite foundations for dynamic replanning

use glam::Vec2;

/// Grid map with obstacle information
#[derive(Clone, Debug)]
pub struct GridMap {
    pub width: usize,
    pub height: usize,
    obstacles: Vec<bool>,
    costs: Vec<f32>, // Optional per-cell traversal cost
}

impl GridMap {
    /// Create empty grid
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            obstacles: vec![false; size],
            costs: vec![1.0; size],
        }
    }

    /// Create from obstacle array
    pub fn from_obstacles(width: usize, height: usize, obstacles: Vec<bool>) -> Self {
        assert_eq!(obstacles.len(), width * height);
        let size = width * height;
        Self {
            width,
            height,
            obstacles,
            costs: vec![1.0; size],
        }
    }

    #[inline]
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    /// Set cell as obstacle or clear
    pub fn set_obstacle(&mut self, x: usize, y: usize, is_obstacle: bool) {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.obstacles[idx] = is_obstacle;
        }
    }

    /// Check if cell is obstacle
    #[inline]
    pub fn is_obstacle(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.obstacles[idx]
        } else {
            true // Out of bounds is obstacle
        }
    }

    /// Check if cell is traversable
    #[inline]
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && !self.is_obstacle(x as usize, y as usize)
    }

    /// Set traversal cost for cell (1.0 = normal, higher = slower)
    pub fn set_cost(&mut self, x: usize, y: usize, cost: f32) {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.costs[idx] = cost.max(0.1);
        }
    }

    /// Get traversal cost for cell
    #[inline]
    pub fn cost(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.costs[idx]
        } else {
            f32::INFINITY
        }
    }

    /// Get 4-connected neighbors (N, S, E, W)
    pub fn neighbors_4(&self, x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> + '_ {
        const DIRS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        DIRS.iter()
            .map(move |(dx, dy)| (x + dx, y + dy))
            .filter(|&(nx, ny)| self.is_passable(nx, ny))
    }

    /// Get 8-connected neighbors (including diagonals)
    pub fn neighbors_8(&self, x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> + '_ {
        const DIRS: [(i32, i32); 8] = [
            (0, -1),
            (0, 1),
            (-1, 0),
            (1, 0),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ];
        DIRS.iter()
            .map(move |(dx, dy)| (x + dx, y + dy))
            .filter(|&(nx, ny)| self.is_passable(nx, ny))
    }

    /// Clear all obstacles
    pub fn clear(&mut self) {
        self.obstacles.fill(false);
        self.costs.fill(1.0);
    }
}

/// Simple node for pathfinding priority queue
#[derive(Clone, Copy, Debug)]
struct PathNode {
    x: i32,
    y: i32,
    g: f32, // Cost from start
    f: f32, // g + heuristic
}

impl PathNode {
    fn new(x: i32, y: i32, g: f32, h: f32) -> Self {
        Self { x, y, g, f: g + h }
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap behavior
        other
            .f
            .partial_cmp(&self.f)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Heuristic functions for A*
#[derive(Clone, Copy, Debug)]
pub enum Heuristic {
    /// Manhattan distance (4-connected)
    Manhattan,
    /// Euclidean distance (8-connected)
    Euclidean,
    /// Chebyshev distance (8-connected, uniform cost)
    Chebyshev,
    /// No heuristic (Dijkstra)
    Zero,
}

impl Heuristic {
    fn compute(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
        let dx = (x2 - x1).abs() as f32;
        let dy = (y2 - y1).abs() as f32;
        match self {
            Heuristic::Manhattan => dx + dy,
            Heuristic::Euclidean => (dx * dx + dy * dy).sqrt(),
            Heuristic::Chebyshev => dx.max(dy),
            Heuristic::Zero => 0.0,
        }
    }
}

/// A* pathfinding result
#[derive(Clone, Debug)]
pub struct PathResult {
    /// Path from start to goal (empty if no path found)
    pub path: Vec<(i32, i32)>,
    /// Total cost of path
    pub cost: f32,
    /// Number of nodes explored
    pub nodes_explored: usize,
}

/// Find shortest path using A* algorithm
///
/// Returns path as list of (x, y) coordinates from start to goal
pub fn astar(
    map: &GridMap,
    start: (i32, i32),
    goal: (i32, i32),
    heuristic: Heuristic,
    use_diagonals: bool,
) -> PathResult {
    use std::collections::{BinaryHeap, HashMap};

    if !map.is_passable(start.0, start.1) || !map.is_passable(goal.0, goal.1) {
        return PathResult {
            path: Vec::new(),
            cost: f32::INFINITY,
            nodes_explored: 0,
        };
    }

    if start == goal {
        return PathResult {
            path: vec![start],
            cost: 0.0,
            nodes_explored: 1,
        };
    }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g_score: HashMap<(i32, i32), f32> = HashMap::new();

    let h = heuristic.compute(start.0, start.1, goal.0, goal.1);
    open.push(PathNode::new(start.0, start.1, 0.0, h));
    g_score.insert(start, 0.0);

    let mut nodes_explored = 0;

    while let Some(current) = open.pop() {
        nodes_explored += 1;
        let pos = (current.x, current.y);

        if pos == goal {
            // Reconstruct path
            let mut path = Vec::new();
            let mut curr = goal;
            while curr != start {
                path.push(curr);
                curr = came_from[&curr];
            }
            path.push(start);
            path.reverse();

            return PathResult {
                path,
                cost: current.g,
                nodes_explored,
            };
        }

        let current_g = g_score.get(&pos).copied().unwrap_or(f32::INFINITY);
        if current.g > current_g {
            continue; // Already found better path
        }

        let neighbors: Vec<_> = if use_diagonals {
            map.neighbors_8(pos.0, pos.1).collect()
        } else {
            map.neighbors_4(pos.0, pos.1).collect()
        };

        for (nx, ny) in neighbors {
            let move_cost = if use_diagonals && (nx - pos.0).abs() + (ny - pos.1).abs() == 2 {
                std::f32::consts::SQRT_2 // Diagonal
            } else {
                1.0
            };
            let cell_cost = map.cost(nx as usize, ny as usize);
            let tentative_g = current_g + move_cost * cell_cost;

            if tentative_g < g_score.get(&(nx, ny)).copied().unwrap_or(f32::INFINITY) {
                came_from.insert((nx, ny), pos);
                g_score.insert((nx, ny), tentative_g);
                let h = heuristic.compute(nx, ny, goal.0, goal.1);
                open.push(PathNode::new(nx, ny, tentative_g, h));
            }
        }
    }

    // No path found
    PathResult {
        path: Vec::new(),
        cost: f32::INFINITY,
        nodes_explored,
    }
}

/// Convert grid path to world coordinates
pub fn path_to_world(path: &[(i32, i32)], cell_size: f32, offset: Vec2) -> Vec<Vec2> {
    path.iter()
        .map(|&(x, y)| {
            Vec2::new(
                x as f32 * cell_size + offset.x,
                y as f32 * cell_size + offset.y,
            )
        })
        .collect()
}

/// Convert world position to grid cell
pub fn world_to_grid(pos: Vec2, cell_size: f32, offset: Vec2) -> (i32, i32) {
    let local = pos - offset;
    (
        (local.x / cell_size).floor() as i32,
        (local.y / cell_size).floor() as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        let map = GridMap::new(10, 10);
        let result = astar(&map, (0, 0), (9, 9), Heuristic::Manhattan, false);
        assert!(!result.path.is_empty());
        assert_eq!(result.path[0], (0, 0));
        assert_eq!(*result.path.last().unwrap(), (9, 9));
    }

    #[test]
    fn test_obstacle_avoidance() {
        let mut map = GridMap::new(5, 5);
        // Wall in the middle at x=2, from y=0 to y=3
        for y in 0..4 {
            map.set_obstacle(2, y, true);
        }

        let result = astar(&map, (0, 2), (4, 2), Heuristic::Manhattan, false);
        assert!(!result.path.is_empty());
        // Path should only go through passable cells
        assert!(result.path.iter().all(|&(x, y)| map.is_passable(x, y)));
        // Path should start and end at correct positions
        assert_eq!(result.path[0], (0, 2));
        assert_eq!(*result.path.last().unwrap(), (4, 2));
    }

    #[test]
    fn test_no_path() {
        let mut map = GridMap::new(5, 5);
        // Complete wall
        for y in 0..5 {
            map.set_obstacle(2, y, true);
        }

        let result = astar(&map, (0, 2), (4, 2), Heuristic::Manhattan, false);
        assert!(result.path.is_empty());
    }

    #[test]
    fn test_diagonal_movement() {
        let map = GridMap::new(10, 10);
        let result = astar(&map, (0, 0), (9, 9), Heuristic::Euclidean, true);

        // Diagonal path should be shorter
        assert!(!result.path.is_empty());
        // Cost should be approximately 9 * sqrt(2) ≈ 12.7
        assert!(result.cost < 14.0);
    }

    #[test]
    fn test_start_equals_goal() {
        let map = GridMap::new(5, 5);
        let result = astar(&map, (2, 2), (2, 2), Heuristic::Manhattan, false);
        assert_eq!(result.path.len(), 1);
        assert_eq!(result.cost, 0.0);
    }
}
