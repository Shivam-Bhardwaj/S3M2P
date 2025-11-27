use glam::Vec2;
use web_sys::CanvasRenderingContext2d;
use wasm_bindgen::JsValue;

const MAX_NODES: usize = 2000;
const GROWTH_DISTANCE: f32 = 15.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BranchType {
    EnergyHigh, // Gold - High nutrition
    EnergyMed,  // Green - Medium nutrition
    EnergyLow,  // Blue - Low nutrition
    Poison,     // Purple - Damages energy
    Death,      // Red - Instant kill
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
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        
        let pos = Vec2::new(x, y);
        if !self.is_space_occupied(pos, GROWTH_DISTANCE * 0.8) {
            // Roots are usually healthy
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

        self.nodes[idx] = FungalNode {
            pos,
            parent_idx,
            health: 1.0,
            age: 0.0,
            active: true,
            angle,
            branch_type,
        };
        self.add_to_grid(idx as u16, pos);
    }

    pub fn update(&mut self) {
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

        // Grow
        let mut new_nodes = Vec::new();

        if self.growth_timer % 5.0 == 0.0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            
            for i in 0..limit {
                if !self.nodes[i].active || self.nodes[i].health < 0.5 { continue; }
                
                if rng.gen::<f32>() < 0.05 {
                    let current_angle = self.nodes[i].angle;
                    let branches = rng.gen_range(1..=2); 
                    let parent_type = self.nodes[i].branch_type;
                    
                    for _ in 0..branches {
                        let offset_deg = rng.gen_range(-1..=1) as f32 * 30.0; 
                        let new_angle = current_angle + offset_deg.to_radians();
                        
                        let dir = Vec2::new(new_angle.cos(), new_angle.sin());
                        let new_pos = self.nodes[i].pos + dir * GROWTH_DISTANCE;
                        
                        if new_pos.x >= 0.0 && new_pos.x <= self.width && new_pos.y >= 0.0 && new_pos.y <= self.height {
                            if !self.is_space_occupied(new_pos, GROWTH_DISTANCE * 0.8) {
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

        // Decay
        for i in 0..limit {
            if !self.nodes[i].active { continue; }
            
            self.nodes[i].age += 1.0;
            if self.nodes[i].age > 3000.0 {
                self.nodes[i].health -= 0.0005;
            }
            
            if self.nodes[i].health < 0.95 {
                self.nodes[i].health -= 0.01; 
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
        ctx.set_line_cap("round");
        
        // Iterate up to limit to cover recycled nodes correctly
        let limit = if self.count < MAX_NODES { self.count } else { MAX_NODES };

        for i in 0..limit {
            if !self.nodes[i].active { continue; }
            
            // Skip drawing line if parent is invalid (recycled/dead)
            if let Some(parent_idx) = self.nodes[i].parent_idx {
                let parent = &self.nodes[parent_idx as usize];
                
                // FIX: Only draw connection if parent is active AND actually older (prevent linking to recycled slot that is now a new unrelated node)
                // Simple heuristic: check distance. If "parent" jumped across screen, don't draw line.
                if parent.active && parent.pos.distance_squared(self.nodes[i].pos) < (GROWTH_DISTANCE * 2.0).powi(2) {
                    let health = self.nodes[i].health;
                    let alpha = 0.2 + health * 0.6;
                    let width = (0.5 + health * 2.5) as f64;
                    
                    // Color based on type
                    let color = match self.nodes[i].branch_type {
                        BranchType::EnergyHigh => format!("hsla(50, 90%, 60%, {})", alpha), // Gold
                        BranchType::EnergyMed => format!("hsla(120, 70%, 50%, {})", alpha), // Green
                        BranchType::EnergyLow => format!("hsla(200, 70%, 50%, {})", alpha), // Blue
                        BranchType::Poison => format!("hsla(280, 80%, 40%, {})", alpha),    // Purple
                        BranchType::Death => format!("hsla(0, 90%, 40%, {})", alpha),       // Red
                    };

                    ctx.set_stroke_style(&JsValue::from_str(&color));
                    ctx.set_line_width(width);
                    
                    ctx.begin_path();
                    ctx.move_to(parent.pos.x as f64, parent.pos.y as f64);
                    ctx.line_to(self.nodes[i].pos.x as f64, self.nodes[i].pos.y as f64);
                    ctx.stroke();
                }
            } else {
                // Root
                let health = self.nodes[i].health;
                let color = match self.nodes[i].branch_type {
                    BranchType::EnergyHigh => format!("hsla(50, 90%, 60%, {})", 0.3 * health),
                    BranchType::EnergyMed => format!("hsla(120, 70%, 50%, {})", 0.3 * health),
                    BranchType::EnergyLow => format!("hsla(200, 70%, 50%, {})", 0.3 * health),
                    BranchType::Poison => format!("hsla(280, 80%, 40%, {})", 0.3 * health),
                    BranchType::Death => format!("hsla(0, 90%, 40%, {})", 0.3 * health),
                };
                
                ctx.set_fill_style(&JsValue::from_str(&color));
                ctx.begin_path();
                // Clamp radius to non-negative
                let r = (2.0 * health).max(0.0) as f64;
                ctx.arc(self.nodes[i].pos.x as f64, self.nodes[i].pos.y as f64, r, 0.0, std::f64::consts::TAU).unwrap();
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
        let mut net = FungalNetwork::new(100.0, 100.0);
        
        // Fill 'er up artifically
        for _ in 0..MAX_NODES {
            net.seed_at(Vec2::new(50.0, 50.0));
        }
        assert_eq!(net.count, MAX_NODES);
        
        // Kill a node manually
        net.nodes[0].active = false;
        
        // Try to add again - should succeed by recycling
        net.seed_at(Vec2::new(60.0, 60.0));
        
        assert!(net.nodes[0].active);
        assert_eq!(net.nodes[0].pos, Vec2::new(60.0, 60.0));
        // When recycling, count stays at MAX_NODES
        assert_eq!(net.count, MAX_NODES);
    }
}
