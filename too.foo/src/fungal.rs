use glam::Vec2;
use web_sys::CanvasRenderingContext2d;
use wasm_bindgen::JsValue;

const MAX_NODES: usize = 2000;
const GROWTH_DISTANCE: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BranchType {
    EnergyHigh, // Gold - High power
    EnergyMed,  // Green - Standard organic
    EnergyLow,  // Brown - Decay/Root
    Poison,     // Purple - Corrupted
    Death,      // Dark Red - Thorns
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionResult {
    None,
    Nutrient(f32),
    Damage(f32),
    Death,
}

#[derive(Clone, Copy, Debug)]
pub struct FungalNode {
    pub pos: Vec2,
    pub parent_idx: Option<u16>,
    pub health: f32, // 0.0 - 1.0
    pub age: f32,
    pub active: bool,
    pub angle: f32, // Direction of growth
    pub branch_type: BranchType,
    pub curvature: f32, // New: Organic curve offset
    pub variant: u8,    // New: Shape variant (0: Smooth, 1: Jagged, 2: Leafy)
}

impl Default for FungalNode {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            parent_idx: None,
            health: 0.0,
            age: 0.0,
            active: false,
            angle: 0.0,
            branch_type: BranchType::EnergyMed,
            curvature: 0.0,
            variant: 0,
        }
    }
}

pub struct FungalNetwork {
    pub nodes: Vec<FungalNode>, 
    pub count: usize,
    pub width: f32,
    pub height: f32,
    pub growth_timer: f32,
    // Simple spatial binning for optimization
    // Cells store indices of nodes
    spatial_grid: Vec<Vec<u16>>,
    grid_cols: usize,
    grid_rows: usize,
    cell_size: f32,
}

impl FungalNetwork {
    pub fn new(width: f32, height: f32) -> Self {
        let cell_size = 50.0;
        let cols = (width / cell_size).ceil() as usize;
        let rows = (height / cell_size).ceil() as usize;
        
        Self {
            nodes: vec![FungalNode::default(); MAX_NODES],
            count: 0,
            width,
            height,
            growth_timer: 0.0,
            spatial_grid: vec![Vec::new(); cols * rows],
            grid_cols: cols,
            grid_rows: rows,
            cell_size,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.grid_cols = (width / self.cell_size).ceil() as usize;
            self.grid_rows = (height / self.cell_size).ceil() as usize;
            self.spatial_grid = vec![Vec::new(); self.grid_cols * self.grid_rows];
            // Resetting on resize to avoid grid misalignment
            self.count = 0; 
        }
    }

    fn get_cell_index(&self, pos: Vec2) -> Option<usize> {
        if pos.x < 0.0 || pos.x >= self.width || pos.y < 0.0 || pos.y >= self.height {
            return None;
        }
        let col = (pos.x / self.cell_size) as usize;
        let row = (pos.y / self.cell_size) as usize;
        Some(row * self.grid_cols + col)
    }

    fn add_to_grid(&mut self, idx: u16, pos: Vec2) {
        if let Some(cell_idx) = self.get_cell_index(pos) {
            if cell_idx < self.spatial_grid.len() {
                self.spatial_grid[cell_idx].push(idx);
            }
        }
    }

    fn is_space_occupied(&self, pos: Vec2, radius: f32) -> bool {
        let cell_idx = match self.get_cell_index(pos) {
            Some(idx) => idx,
            None => return true, 
        };
        
        let row = cell_idx / self.grid_cols;
        let col = cell_idx % self.grid_cols;
        
        let radius_sq = radius * radius;

        for r in row.saturating_sub(1)..=(row + 1).min(self.grid_rows - 1) {
            for c in col.saturating_sub(1)..=(col + 1).min(self.grid_cols - 1) {
                let idx = r * self.grid_cols + c;
                for &node_idx in &self.spatial_grid[idx] {
                    let node = &self.nodes[node_idx as usize];
                    if node.active && node.pos.distance_squared(pos) < radius_sq {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn spawn_root(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0..self.width);
        let y = rng.gen_range(0.0..self.height);
        // Organic: Any angle, not just cardinal
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        
        let pos = Vec2::new(x, y);
        if !self.is_space_occupied(pos, GROWTH_DISTANCE * 0.8) {
            self.add_node(pos, None, angle, BranchType::EnergyMed);
        }
    }

    pub fn seed_at(&mut self, pos: Vec2) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        
        if !self.is_space_occupied(pos, GROWTH_DISTANCE * 0.5) {
            self.add_node(pos, None, angle, BranchType::EnergyMed);
        }
    }

    /// Seed at position, checking exclusion zones
    pub fn seed_at_safe(&mut self, pos: Vec2, exclusion_zones: &[crate::ExclusionZone]) {
        // Check exclusion zones
        for zone in exclusion_zones {
            if pos.distance(zone.center) < zone.radius {
                return;
            }
        }
        self.seed_at(pos);
    }

    /// Check if position is in any exclusion zone
    fn is_in_exclusion(&self, pos: Vec2, exclusion_zones: &[crate::ExclusionZone]) -> bool {
        for zone in exclusion_zones {
            if pos.distance(zone.center) < zone.radius {
                return true;
            }
        }
        false
    }

    /// Count active nodes in a cell for density calculation
    fn get_cell_density(&self, cell_idx: usize) -> usize {
        if cell_idx >= self.spatial_grid.len() {
            return 0;
        }
        self.spatial_grid[cell_idx].len()
    }

    fn determine_branch_type(&self, parent_type: BranchType) -> BranchType {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let roll = rng.gen::<f32>();

        // Inheritance with mutation (80% chance to inherit)
        if roll < 0.8 {
            return parent_type;
        }

        // Mutation
        let type_roll = rng.gen::<f32>();
        if type_roll < 0.6 {
            // Beneficial mutations
            match rng.gen_range(0..3) {
                0 => BranchType::EnergyHigh,
                1 => BranchType::EnergyMed,
                _ => BranchType::EnergyLow,
            }
        } else if type_roll < 0.9 {
            BranchType::Poison
        } else {
            BranchType::Death
        }
    }

    fn add_node(&mut self, pos: Vec2, parent_idx: Option<u16>, angle: f32, branch_type: BranchType) {
        let idx;
        
        if self.count < MAX_NODES {
            idx = self.count;
            self.count += 1;
        } else {
            // Recycle logic: Find a dead node to reuse
            match self.nodes.iter().position(|n| !n.active) {
                Some(i) => idx = i,
                None => return, // No space, stop growth
            }
        }

        // Random properties for organic diversity
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let curvature = (rng.gen::<f32>() - 0.5) * 20.0; // Bezier control point offset
        let variant = rng.gen_range(0..3);

        self.nodes[idx] = FungalNode {
            pos,
            parent_idx,
            health: 1.0,
            age: 0.0,
            active: true,
            angle,
            branch_type,
            curvature,
            variant,
        };
        self.add_to_grid(idx as u16, pos);
    }

    #[cfg(test)]
    // Helper for tests to advance state
    pub fn update(&mut self) {
        self.update_with_exclusions(&[]);
    }

    /// Update with exclusion zones - prevents growth in UI areas
    pub fn update_with_exclusions(&mut self, exclusion_zones: &[crate::ExclusionZone]) {
        use rand::Rng;
        self.growth_timer += 1.0;
        
        if self.growth_timer % 60.0 == 0.0 {
            self.spawn_root();
        }

        // Rebuild grid to handle recycling correctly
        for bin in &mut self.spatial_grid {
            bin.clear();
        }
        let limit = if self.count < MAX_NODES { self.count } else { MAX_NODES };
        for i in 0..limit {
             if self.nodes[i].active {
                 self.add_to_grid(i as u16, self.nodes[i].pos);
             }
        }

        // Grow (avoid exclusion zones)
        let mut new_nodes = Vec::new();

        if self.growth_timer % 5.0 == 0.0 {
            let mut rng = rand::thread_rng();
            
            for i in 0..limit {
                if !self.nodes[i].active || self.nodes[i].health < 0.5 { continue; }
                
                if rng.gen::<f32>() < 0.05 {
                    let current_angle = self.nodes[i].angle;
                    let branches = rng.gen_range(1..=2); 
                    let parent_type = self.nodes[i].branch_type;
                    
                    for _ in 0..branches {
                        // Organic logic: Slight deviations + occasional splits, no grid lock
                        // Deviation -45 to +45 degrees
                        let deviation = (rng.gen::<f32>() - 0.5) * 1.5; 
                        let new_angle = current_angle + deviation;
                        
                        let dir = Vec2::new(new_angle.cos(), new_angle.sin());
                        
                        // No quantization for organic look
                        let new_pos = self.nodes[i].pos + dir * GROWTH_DISTANCE;
                        
                        // Check bounds and exclusion zones
                        if new_pos.x >= 0.0 && new_pos.x <= self.width && new_pos.y >= 0.0 && new_pos.y <= self.height {
                            if !self.is_space_occupied(new_pos, GROWTH_DISTANCE * 0.6) 
                               && !self.is_in_exclusion(new_pos, exclusion_zones) {
                                let new_type = self.determine_branch_type(parent_type);
                                new_nodes.push((new_pos, i as u16, new_angle, new_type));
                            }
                        }
                    }
                }
            }
        }
        
        for (pos, parent, angle, b_type) in new_nodes {
            self.add_node(pos, Some(parent), angle, b_type);
        }

        // Decay with density-based acceleration
        let mut rng = rand::thread_rng();
        
        // Count active nodes for culling decision
        let active_count = self.nodes.iter().filter(|n| n.active).count();
        let near_capacity = active_count > (MAX_NODES * 8 / 10); // 80% full
        
        for i in 0..limit {
            if !self.nodes[i].active { continue; }
            
            let pos = self.nodes[i].pos;
            
            // Kill nodes in exclusion zones
            if self.is_in_exclusion(pos, exclusion_zones) {
                self.nodes[i].active = false;
                self.nodes[i].health = 0.0;
                continue;
            }
            
            self.nodes[i].age += 1.0;
            
            // Base age decay
            if self.nodes[i].age > 2000.0 {
                self.nodes[i].health -= 0.001;
            }
            
            // Density-based decay: nodes in crowded areas decay faster
            if let Some(cell_idx) = self.get_cell_index(pos) {
                let density = self.get_cell_density(cell_idx);
                if density > 8 {
                    // High density area - faster decay to prevent trapping
                    self.nodes[i].health -= 0.005 * (density as f32 / 8.0);
                }
            }
            
            // Stagnation decay: old nodes that haven't been eaten decay
            if self.nodes[i].health > 0.85 && self.nodes[i].age > 1500.0 {
                // Hasn't been interacted with - decay faster
                self.nodes[i].health -= 0.002;
            }
            
            // Random culling when near capacity
            if near_capacity && rng.gen::<f32>() < 0.001 {
                self.nodes[i].health -= 0.1;
            }
            
            // Standard decay for damaged nodes
            if self.nodes[i].health < 0.95 {
                self.nodes[i].health -= 0.008; 
            }
            
            if self.nodes[i].health <= 0.0 {
                self.nodes[i].active = false;
                self.nodes[i].health = 0.0;
            }
        }
    }

    // Boids interact with nodes they touch
    // Returns the strongest interaction effect found
    pub fn interact(&mut self, pos: Vec2, radius: f32) -> InteractionResult {
        let radius_sq = radius * radius;
        let mut result = InteractionResult::None;
        
        if let Some(center_idx) = self.get_cell_index(pos) {
            let row = center_idx / self.grid_cols;
            let col = center_idx % self.grid_cols;
            
            for r in row.saturating_sub(1)..=(row + 1).min(self.grid_rows - 1) {
                for c in col.saturating_sub(1)..=(col + 1).min(self.grid_cols - 1) {
                    let idx = r * self.grid_cols + c;
                    for &node_idx in &self.spatial_grid[idx] {
                        let i = node_idx as usize;
                        if !self.nodes[i].active { continue; }
                        
                        let dist_sq = self.nodes[i].pos.distance_squared(pos);
                        if dist_sq < radius_sq {
                            // Determine effect
                            let effect = match self.nodes[i].branch_type {
                                BranchType::EnergyHigh => InteractionResult::Nutrient(20.0),
                                BranchType::EnergyMed => InteractionResult::Nutrient(10.0),
                                BranchType::EnergyLow => InteractionResult::Nutrient(5.0),
                                BranchType::Poison => InteractionResult::Damage(5.0),
                                BranchType::Death => InteractionResult::Death,
                            };
                            
                            // Prioritize Death > Damage > Nutrient
                            match (result, effect) {
                                (InteractionResult::Death, _) => {},
                                (_, InteractionResult::Death) => result = InteractionResult::Death,
                                (InteractionResult::Damage(_), _) => {}, // Keep existing damage? Or accumulate? Simplified: keep first damage.
                                (_, InteractionResult::Damage(d)) => result = InteractionResult::Damage(d),
                                (InteractionResult::Nutrient(a), InteractionResult::Nutrient(b)) => result = InteractionResult::Nutrient(a.max(b)),
                                (InteractionResult::None, e) => result = e,
                                _ => {},
                            }

                            // Decay the node (it was eaten/touched)
                            if self.nodes[i].health > 0.9 {
                                self.nodes[i].health = 0.9; 
                            }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d) {
        // Draw organic branches
        ctx.set_line_cap("round"); // Round caps for organic look
        ctx.set_line_join("round");
        
        let limit = if self.count < MAX_NODES { self.count } else { MAX_NODES };

        for i in 0..limit {
            if !self.nodes[i].active { continue; }
            
            if let Some(parent_idx) = self.nodes[i].parent_idx {
                let parent = &self.nodes[parent_idx as usize];
                
                if parent.active && parent.pos.distance_squared(self.nodes[i].pos) < (GROWTH_DISTANCE * 2.5).powi(2) {
                    let health = self.nodes[i].health;
                    let alpha = 0.3 + health * 0.7;
                    // Vary width based on variant and health (tapering)
                    let width = (1.0 + health * 2.0) * if self.nodes[i].variant == 2 { 1.5 } else { 1.0 };
                    
                    // Organic Colors (Earth tones)
                    let color = match self.nodes[i].branch_type {
                        BranchType::EnergyHigh => format!("rgba(218, 165, 32, {})", alpha), // Goldenrod
                        BranchType::EnergyMed => format!("rgba(85, 107, 47, {})", alpha),   // Dark Olive Green
                        BranchType::EnergyLow => format!("rgba(139, 69, 19, {})", alpha),   // Saddle Brown
                        BranchType::Poison => format!("rgba(75, 0, 130, {})", alpha),       // Indigo
                        BranchType::Death => format!("rgba(128, 0, 0, {})", alpha),         // Maroon
                    };

                    ctx.set_stroke_style(&JsValue::from_str(&color));
                    ctx.set_line_width(width as f64);
                    
                    ctx.begin_path();
                    ctx.move_to(parent.pos.x as f64, parent.pos.y as f64);
                    
                    // Quadratic curve for organic feel
                    // Control point offset by curvature perpendicular to direction
                    let mid = (parent.pos + self.nodes[i].pos) * 0.5;
                    let dir = self.nodes[i].pos - parent.pos;
                    let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero();
                    let control = mid + perp * self.nodes[i].curvature;
                    
                    ctx.quadratic_curve_to(
                        control.x as f64, control.y as f64,
                        self.nodes[i].pos.x as f64, self.nodes[i].pos.y as f64
                    );
                    ctx.stroke();
                    
                    // Thorns / Leaves based on variant
                    if health > 0.5 {
                        if self.nodes[i].branch_type == BranchType::Death {
                            // Thorns
                            ctx.set_fill_style(&JsValue::from_str(&color));
                            ctx.begin_path();
                            ctx.move_to(mid.x as f64, mid.y as f64);
                            let thorn_tip = mid + perp * 5.0;
                            ctx.line_to(thorn_tip.x as f64, thorn_tip.y as f64);
                            ctx.line_to((mid.x + dir.x * 0.1) as f64, (mid.y + dir.y * 0.1) as f64);
                            ctx.fill();
                        } else if self.nodes[i].variant == 2 {
                            // Leaves (Simple oval)
                            ctx.set_fill_style(&JsValue::from_str(&format!("rgba(34, 139, 34, {})", alpha * 0.8)));
                            ctx.begin_path();
                            ctx.ellipse(
                                self.nodes[i].pos.x as f64, 
                                self.nodes[i].pos.y as f64, 
                                3.0, 6.0, 
                                self.nodes[i].angle as f64, 
                                0.0, 
                                std::f64::consts::TAU
                            ).unwrap_or(());
                            ctx.fill();
                        }
                    }
                }
            } else {
                // Root Node (Seed)
                let color = "rgba(107, 142, 35, 0.8)"; // Olive Drab
                
                ctx.set_fill_style(&JsValue::from_str(color));
                ctx.begin_path();
                ctx.arc(
                    self.nodes[i].pos.x as f64, 
                    self.nodes[i].pos.y as f64, 
                    4.0, 0.0, std::f64::consts::TAU
                ).unwrap();
                ctx.fill();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let net = FungalNetwork::new(100.0, 100.0);
        assert_eq!(net.count, 0);
        assert_eq!(net.nodes.len(), MAX_NODES);
    }

    #[test]
    fn test_seed_at() {
        let mut net = FungalNetwork::new(100.0, 100.0);
        net.seed_at(Vec2::new(50.0, 50.0));
        assert_eq!(net.count, 1);
        assert!(net.nodes[0].active);
        assert_eq!(net.nodes[0].health, 1.0);
        assert_eq!(net.nodes[0].branch_type, BranchType::EnergyMed); // Default
    }

    #[test]
    fn test_interaction_effects() {
        let mut net = FungalNetwork::new(100.0, 100.0);
        net.seed_at(Vec2::new(50.0, 50.0)); // Spawns EnergyMed by default
        
        // Interact
        let result = net.interact(Vec2::new(50.0, 50.0), 10.0);
        
        assert_eq!(result, InteractionResult::Nutrient(10.0));
        
        // Check decay
        net.update();
        assert!(net.nodes[0].health < 0.9);
    }

    #[test]
    fn test_continuous_growth_recycling() {
        let mut net = FungalNetwork::new(1000.0, 1000.0);
        
        // Fill 'er up artificially by directly adding nodes at different positions
        // (bypassing is_space_occupied check which prevents seeding at same location)
        for i in 0..MAX_NODES {
            let x = (i % 100) as f32 * 10.0;
            let y = (i / 100) as f32 * 10.0;
            net.add_node(Vec2::new(x, y), None, 0.0, BranchType::EnergyMed);
        }
        assert_eq!(net.count, MAX_NODES);
        
        // Kill a node manually
        net.nodes[0].active = false;
        
        // Try to add again - should succeed by recycling
        net.add_node(Vec2::new(500.0, 500.0), None, 0.0, BranchType::EnergyMed);
        
        assert!(net.nodes[0].active);
        assert_eq!(net.nodes[0].pos, Vec2::new(500.0, 500.0));
        // When recycling, count stays at MAX_NODES
        assert_eq!(net.count, MAX_NODES);
    }
}