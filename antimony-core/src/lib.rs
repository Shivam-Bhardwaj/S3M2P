use glam::Vec2;
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub center: Vec2,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Genome {
    pub max_speed: f32,
    pub sensor_radius: f32,
    pub color: u32,
    pub metabolism_efficiency: f32, // Range 0.8-1.2
}

#[derive(Clone, Debug)]
pub struct Boid {
    pub position: Vec2,
    pub velocity: Vec2,
    pub genes: Genome,
    pub energy: f32,
    pub age: f32,
    pub generation: u32,
}

impl Boid {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        
        // Random position in 0.0 to 1.0 range
        let position = Vec2::new(
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
        );
        
        // Random velocity
        let velocity = Vec2::new(
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
        );
        
        // Default genome with random metabolism efficiency (0.8-1.2)
        let genes = Genome {
            max_speed: rng.gen_range(1.5..=3.5),
            sensor_radius: 0.1,
            color: 0xFFFFFF,
            metabolism_efficiency: rng.gen_range(0.8..=1.2),
        };
        
        Self {
            position,
            velocity,
            genes,
            energy: 100.0,
            age: 0.0,
            generation: 0,
        }
    }
    
    pub fn update(&mut self, dt: f32, width: f32, height: f32) {
        // Update position based on velocity
        self.position += self.velocity * dt;
        
        // Wrap around screen edges (toroidal space)
        if self.position.x < 0.0 {
            self.position.x += width;
        } else if self.position.x >= width {
            self.position.x -= width;
        }
        
        if self.position.y < 0.0 {
            self.position.y += height;
        } else if self.position.y >= height {
            self.position.y -= height;
        }
        
        // Metabolism: decrease energy based on movement
        self.energy -= self.velocity.length() * 0.01 * self.genes.metabolism_efficiency;
        
        // Aging
        self.age += dt;
    }
    
    /// Returns true if the boid is dead (energy <= 0)
    pub fn is_dead(&self) -> bool {
        self.energy <= 0.0
    }
    
    /// Feed the boid, increasing energy (capped at 200.0)
    pub fn feed(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(200.0);
    }
    
    /// Attempt to reproduce. Returns a child if energy > 150.0
    pub fn reproduce(&mut self) -> Option<Boid> {
        if self.energy > 150.0 {
            let mut rng = rand::thread_rng();
            
            // Reduce parent energy (cost of reproduction)
            self.energy -= 60.0;
            
            // Create child with mutated genome (+/- 5%)
            let mut mutate = |value: f32| -> f32 {
                let mutation = rng.gen_range(-0.05..=0.05);
                value * (1.0 + mutation)
            };
            
            let child_genes = Genome {
                max_speed: mutate(self.genes.max_speed),
                sensor_radius: mutate(self.genes.sensor_radius),
                color: self.genes.color, // Color inherited directly
                metabolism_efficiency: mutate(self.genes.metabolism_efficiency).clamp(0.8, 1.2),
            };
            
            Some(Boid {
                position: self.position, // Same position as parent
                velocity: Vec2::new(
                    rng.gen_range(-1.0..=1.0),
                    rng.gen_range(-1.0..=1.0),
                ),
                genes: child_genes,
                energy: 100.0, // Child starts with full energy
                age: 0.0,
                generation: self.generation + 1, // Increment generation
            })
        } else {
            None
        }
    }

    pub fn cohesion(&self, boids: &[Boid], vision_radius: f32) -> Vec2 {
        let mut center_of_mass = Vec2::ZERO;
        let mut count = 0;

        for other in boids {
            let distance = self.position.distance(other.position);
            if distance > 0.0 && distance < vision_radius {
                center_of_mass += other.position;
                count += 1;
            }
        }

        if count > 0 {
            center_of_mass /= count as f32;
            center_of_mass - self.position
        } else {
            Vec2::ZERO
        }
    }

    pub fn alignment(&self, boids: &[Boid], vision_radius: f32) -> Vec2 {
        let mut avg_velocity = Vec2::ZERO;
        let mut count = 0;

        for other in boids {
            let distance = self.position.distance(other.position);
            if distance > 0.0 && distance < vision_radius {
                avg_velocity += other.velocity;
                count += 1;
            }
        }

        if count > 0 {
            avg_velocity /= count as f32;
            avg_velocity
        } else {
            Vec2::ZERO
        }
    }

    pub fn separation(&self, boids: &[Boid], vision_radius: f32) -> Vec2 {
        let mut steer = Vec2::ZERO;
        let mut count = 0;

        for other in boids {
            let distance = self.position.distance(other.position);
            if distance > 0.0 && distance < vision_radius {
                let diff = self.position - other.position;
                steer += diff / distance; // Weight by 1.0 / distance
                count += 1;
            }
        }

        if count > 0 {
            steer /= count as f32;
            steer
        } else {
            Vec2::ZERO
        }
    }

    pub fn avoid_obstacles(&self, obstacles: &[Obstacle]) -> Vec2 {
        let mut force = Vec2::ZERO;
        let buffer = 50.0; // Buffer zone around obstacles

        for obs in obstacles {
            let d = self.position.distance(obs.center);
            
            if d < obs.radius + buffer {
                // Calculate repulsion vector pointing from obstacle center to boid position
                let repulsion = self.position - obs.center;
                
                if repulsion.length_squared() > 0.0 {
                    // Weight the force by 1.0 / d (closer = stronger push)
                    force += repulsion.normalize() * (1.0 / d);
                }
            }
        }

        force
    }
    
    /// Returns an HSL color string representing the boid's genetic traits and health.
    /// - Hue (Species): Maps max_speed to 0-360 degrees (Blue/slow → Red/fast)
    /// - Saturation (Efficiency): Maps metabolism_efficiency to 50-100% (Efficient=greyer, Wasteful=brighter)
    /// - Lightness (Health): Maps energy to 20-80% (Dying=dark, Healthy=bright)
    pub fn get_color_string(&self) -> String {
        // Hue: Map max_speed (1.5-3.5) to hue (240-0, blue to red)
        // Slow boids are blue (240°), fast boids are red (0°)
        let speed_normalized = ((self.genes.max_speed - 1.5) / 2.0).clamp(0.0, 1.0);
        let hue = ((1.0 - speed_normalized) * 240.0).clamp(0.0, 360.0) as u16;
        
        // Saturation: Map metabolism_efficiency (0.8-1.2) to saturation (100-50%)
        // More efficient (lower) = greyer (50%), more wasteful (higher) = brighter (100%)
        let efficiency_normalized = ((self.genes.metabolism_efficiency - 0.8) / 0.4).clamp(0.0, 1.0);
        let saturation = (50.0 + efficiency_normalized * 50.0).clamp(50.0, 100.0) as u8;
        
        // Lightness: Map energy (0-200) to lightness (20-80%)
        // Dying (low energy) = dark (20%), healthy (high energy) = bright (80%)
        let energy_normalized = (self.energy / 200.0).clamp(0.0, 1.0);
        let lightness = (20.0 + energy_normalized * 60.0).clamp(20.0, 80.0) as u8;
        
        format!("hsl({}, {}%, {}%)", hue, saturation, lightness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separation() {
        let mut boid1 = Boid::new();
        boid1.position = Vec2::new(10.0, 10.0);
        
        let mut boid2 = Boid::new();
        boid2.position = Vec2::new(10.1, 10.1); // Very close

        let boids = vec![boid2];
        let separation = boid1.separation(&boids, 1.0);

        // Separation should point away from boid2
        assert!(separation.length() > 0.0);
        // boid1 is at (10.0, 10.0), boid2 is at (10.1, 10.1)
        // Separation vector (boid1 - boid2) should be negative in both x and y
        assert!(separation.x < 0.0);
        assert!(separation.y < 0.0);
    }

    #[test]
    fn test_alignment() {
        let mut boid1 = Boid::new();
        boid1.position = Vec2::new(10.0, 10.0);
        boid1.velocity = Vec2::new(1.0, 0.0); // Moving Right

        let mut boid2 = Boid::new();
        boid2.position = Vec2::new(10.1, 10.0); // Close neighbor
        boid2.velocity = Vec2::new(0.0, 1.0); // Moving Up

        let boids = vec![boid2];
        let alignment = boid1.alignment(&boids, 1.0);

        // Alignment should match neighbor's velocity (Up)
        assert!(alignment.length() > 0.0);
        assert!(alignment.y > 0.0);
        assert_eq!(alignment.x, 0.0);
    }

    #[test]
    fn test_zero_neighbors() {
        let boid = Boid::new();
        let neighbors: Vec<Boid> = vec![];
        
        assert_eq!(boid.cohesion(&neighbors, 1.0), Vec2::ZERO);
        assert_eq!(boid.alignment(&neighbors, 1.0), Vec2::ZERO);
        assert_eq!(boid.separation(&neighbors, 1.0), Vec2::ZERO);
    }
}
