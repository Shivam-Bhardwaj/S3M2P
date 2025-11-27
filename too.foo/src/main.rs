use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, Document, Performance};
use antimony_core::{
    BoidArena, SpatialGrid, Obstacle, FoodSource, Genome, SimConfig,
    SeasonCycle, PredatorZone, BoidRole, BoidState,
    compute_flocking_forces, simulation_step, feed_from_sources, get_boid_color,
    apply_predator_zones, compute_diversity, trigger_mass_extinction,
};
use glam::Vec2;

mod fungal;
use fungal::{FungalNetwork, InteractionResult};

mod shader;
use shader::BackgroundEffect;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Fixed capacity - no runtime allocations (increased for evolution)
const ARENA_CAPACITY: usize = 4096;
const CELL_CAPACITY: usize = 64;
const BOID_SIZE: f32 = 6.0;
const VISION_RADIUS: f32 = 60.0;

/// Simulation state tracking
struct SimulationStats {
    max_speed_record: f32,
    max_generation: u16,
    low_diversity_frames: u32,
}

/// Chakravyu zone - the deadly center where boids can enter but not escape
#[derive(Clone, Copy, Debug)]
struct ChakravyuZone {
    center: Vec2,
    radius: f32,
    energy_drain: f32,
    inward_force: f32,
}

/// Update the single-line console log (replaces content)
fn log_event(document: &Document, msg: &str, event_class: &str) {
    if let Some(console_log) = document.get_element_by_id("console-log") {
        // Create a span with the message and class
        let styled_msg = format!("<span class=\"{}\">{}</span>", event_class, msg);
        console_log.set_inner_html(&styled_msg);
    }
}

/// Exclusion zone around UI elements where nothing should spawn/grow
#[derive(Clone, Copy, Debug)]
struct ExclusionZone {
    center: Vec2,
    radius: f32,
}

/// Scan DOM for monolith elements and create exclusion zones
/// Updated for circular layout: creates a large exclusion circle in the center
fn scan_exclusion_zones(document: &Document) -> (Vec<ExclusionZone>, Option<ChakravyuZone>) {
    let mut zones = Vec::new();
    let mut chakravyu = None;
    
    // Center constellation exclusion
    if let Some(constellation) = document.get_element_by_id("constellation") {
        let rect = constellation.get_bounding_client_rect();
        let center_x = rect.left() as f32 + rect.width() as f32 / 2.0;
        let center_y = rect.top() as f32 + rect.height() as f32 / 2.0;
        // Radius covers the whole ring + padding (for fungus exclusion)
        let outer_radius = (rect.width().max(rect.height()) as f32) / 2.0 + 20.0;
        
        zones.push(ExclusionZone {
            center: Vec2::new(center_x, center_y),
            radius: outer_radius,
        });
        
        // Chakravyu zone - smaller inner circle where boids get trapped and die
        // Boids CAN enter but will be pulled inward and drained
        let chakravyu_radius = outer_radius * 0.7; // Inner deadly zone
        chakravyu = Some(ChakravyuZone {
            center: Vec2::new(center_x, center_y),
            radius: chakravyu_radius,
            energy_drain: 0.5, // Energy loss per frame inside
            inward_force: 2.0, // Pull toward center
        });
    } else {
        // Fallback to scanning individual monoliths if constellation not found
        let elements = document.get_elements_by_class_name("monolith");
        for i in 0..elements.length() {
            if let Some(element) = elements.item(i) {
                let rect = element.get_bounding_client_rect();
                let center_x = rect.left() as f32 + rect.width() as f32 / 2.0;
                let center_y = rect.top() as f32 + rect.height() as f32 / 2.0;
                let radius = (rect.width().max(rect.height()) as f32) / 2.0 + 40.0;
                
                zones.push(ExclusionZone {
                    center: Vec2::new(center_x, center_y),
                    radius,
                });
            }
        }
    }
    
    (zones, chakravyu)
}

/// Check if a position is inside any exclusion zone
fn is_in_exclusion_zone(pos: Vec2, zones: &[ExclusionZone]) -> bool {
    for zone in zones {
        if pos.distance(zone.center) < zone.radius {
            return true;
        }
    }
    false
}

struct PopUp {
    text: String,
    pos: Vec2,
    life: f32, // 0.0 to 1.0
    color: String,
}

struct World {
    arena: BoidArena<ARENA_CAPACITY>,
    grid: SpatialGrid<CELL_CAPACITY>,
    obstacles: Vec<Obstacle>,
    exclusion_zones: Vec<ExclusionZone>,
    chakravyu: Option<ChakravyuZone>,
    food_sources: Vec<FoodSource>,
    fungal_network: FungalNetwork,
    background: BackgroundEffect,
    predators: Vec<PredatorZone>,
    season: SeasonCycle,
    config: SimConfig,
    width: f32,
    height: f32,
    last_season: &'static str,
    popups: Vec<PopUp>,
}

fn scan_dom_obstacles(document: &Document) -> Vec<Obstacle> {
    let mut obstacles = Vec::new();
    let elements = document.get_elements_by_class_name("monolith");
    
    for i in 0..elements.length() {
        if let Some(element) = elements.item(i) {
            let rect = element.get_bounding_client_rect();
            let center_x = rect.left() as f32 + rect.width() as f32 / 2.0;
            let center_y = rect.top() as f32 + rect.height() as f32 / 2.0;
            let radius = (rect.width().max(rect.height()) as f32) / 2.0;
            
            obstacles.push(Obstacle {
                center: Vec2::new(center_x, center_y),
                radius,
            });
        }
    }
    obstacles
}

fn is_paused() -> bool {
    let window = window().unwrap();
    let location = window.location();
    if let Ok(search) = location.search() {
        search.contains("paused=true")
    } else {
        false
    }
}

fn draw_organism(
    ctx: &CanvasRenderingContext2d, 
    x: f64, 
    y: f64, 
    angle: f64, 
    color: &str, 
    base_size: f64,
    role: BoidRole,
    state: BoidState,
    size_mult: f32,
) {
    ctx.save();
    ctx.translate(x, y).unwrap();
    ctx.rotate(angle).unwrap();
    
    let size = base_size * size_mult as f64;
    
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(1.5);
    
    match role {
        BoidRole::Herbivore => {
            // Round/Smooth shape (Circle/Oval)
            ctx.begin_path();
            ctx.ellipse(0.0, 0.0, size, size * 0.8, 0.0, 0.0, std::f64::consts::TAU).unwrap();
            ctx.stroke();
            
            // Fill for herbivores
            ctx.set_fill_style(&JsValue::from_str(&format!("{}80", color)));
            ctx.begin_path();
            ctx.ellipse(0.0, 0.0, size, size * 0.8, 0.0, 0.0, std::f64::consts::TAU).unwrap();
            ctx.fill();
        }
        BoidRole::Carnivore => {
            // Spiky/Angular shape (Triangle/Chevron)
            ctx.begin_path();
            ctx.move_to(-size, -size * 0.8);
            ctx.line_to(size, 0.0);
            ctx.line_to(-size, size * 0.8);
            ctx.line_to(-size * 0.5, 0.0);
            ctx.close_path();
            ctx.stroke();
            
            // Add spikes if hunting
            if state == BoidState::Hunt {
                ctx.begin_path();
                ctx.move_to(size * 0.3, -size * 0.4);
                ctx.line_to(size * 0.6, -size * 0.6);
                ctx.line_to(size * 0.3, -size * 0.2);
                ctx.close_path();
                ctx.stroke();
                
                ctx.begin_path();
                ctx.move_to(size * 0.3, size * 0.4);
                ctx.line_to(size * 0.6, size * 0.6);
                ctx.line_to(size * 0.3, size * 0.2);
                ctx.close_path();
                ctx.stroke();
            }
        }
        BoidRole::Scavenger => {
            // Smaller irregular shape
            ctx.begin_path();
            ctx.move_to(-size * 0.6, -size * 0.5);
            ctx.line_to(size * 0.4, -size * 0.3);
            ctx.line_to(size * 0.6, size * 0.3);
            ctx.line_to(-size * 0.4, size * 0.5);
            ctx.line_to(-size * 0.6, 0.0);
            ctx.close_path();
            ctx.stroke();
        }
    }
    
    // State indicators
    if state == BoidState::Flee {
        // White ring for fleeing
        ctx.set_stroke_style(&JsValue::from_str("rgba(255, 255, 255, 0.8)"));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        ctx.arc(0.0, 0.0, size * 1.5, 0.0, std::f64::consts::TAU).unwrap();
        ctx.stroke();
    }
    
    // Energy glow
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.6)"));
    ctx.begin_path();
    ctx.arc(-size * 0.3, 0.0, size * 0.2, 0.0, std::f64::consts::TAU).unwrap();
    ctx.fill();

    ctx.restore();
}

fn main() {
    console_error_panic_hook::set_once();

    let window = window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("simulation")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let paused = is_paused();

    // Get simulation area dimensions (canvas parent, excludes telemetry bar)
    let sim_area = document.get_element_by_id("simulation-area");
    let (w, h) = if let Some(area) = &sim_area {
        let rect = area.get_bounding_client_rect();
        (rect.width(), rect.height())
    } else {
        (window.inner_width().unwrap().as_f64().unwrap(),
         window.inner_height().unwrap().as_f64().unwrap())
    };
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);

    // Resize handler
    {
        let canvas = canvas.clone();
        let document_for_closure = document.clone();
        let window_for_closure = window.clone();
        let closure = Closure::wrap(Box::new(move || {
            let sim_area = document_for_closure.get_element_by_id("simulation-area");
            let (w, h) = if let Some(area) = &sim_area {
                let rect = area.get_bounding_client_rect();
                (rect.width(), rect.height())
            } else {
                (window_for_closure.inner_width().unwrap().as_f64().unwrap(),
                 window_for_closure.inner_height().unwrap().as_f64().unwrap())
            };
            canvas.set_width(w as u32);
            canvas.set_height(h as u32);
        }) as Box<dyn FnMut()>);
        window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    let width = w as f32;
    let height = h as f32;

    // Get initial exclusion zones and chakravyu around monoliths
    let (exclusion_zones, chakravyu) = scan_exclusion_zones(&document);

    // Initialize arena with starting population (avoid exclusion zones)
    let mut arena: BoidArena<ARENA_CAPACITY> = BoidArena::new();
    let mut rng = rand::thread_rng();
    use rand::Rng;
    
    for _ in 0..150 {
        // Try to spawn outside exclusion zones
        let mut attempts = 0;
        loop {
            let pos = Vec2::new(
                rng.gen_range(0.0..width),
                rng.gen_range(0.0..height),
            );
            if !is_in_exclusion_zone(pos, &exclusion_zones) || attempts > 10 {
                let vel = Vec2::new(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                );
                arena.spawn(pos, vel, Genome::random());
                break;
            }
            attempts += 1;
        }
    }

    let grid = SpatialGrid::new(width, height, VISION_RADIUS);
    let obstacles = scan_dom_obstacles(&document);
    
    // No fixed food sources anymore, boids forage the network
    let food_sources = vec![];

    // Initialize Fungal Network
    let mut fungal_network = FungalNetwork::new(width, height);
    
    // Initial seeding (avoid exclusion zones)
    for _ in 0..10 {
        fungal_network.spawn_root();
    }

    // Initialize Background Effect
    let background = BackgroundEffect::new(width as f64, height as f64);

    let mut config = SimConfig::default();
    config.reproduction_threshold = 140.0;
    config.base_mortality = 0.00005;

    let state = Rc::new(RefCell::new(World {
        arena,
        grid,
        obstacles,
        exclusion_zones,
        chakravyu,
        food_sources,
        fungal_network,
        background,
        predators: Vec::new(),
        season: SeasonCycle::new(),
        config,
        width,
        height,
        last_season: "SPRING",
        popups: Vec::new(),
    }));

    // Cache DOM element references
    let stat_pop = document.get_element_by_id("stat-pop");
    let stat_gen = document.get_element_by_id("stat-gen");
    let stat_fps = document.get_element_by_id("stat-fps");
    let stat_season = document.get_element_by_id("stat-season");

    let performance: Performance = window.performance().unwrap();

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let state_clone = state.clone();
    let document_clone = document.clone();
    let mut frame_count: u32 = 0;
    let mut last_time = performance.now();
    let mut fps_accumulator = 0.0;
    let mut fps_frame_count = 0;
    let mut stats = SimulationStats {
        max_speed_record: 0.0,
        max_generation: 0,
        low_diversity_frames: 0,
    };
    
    *g.borrow_mut() = Some(Closure::new(move || {
        let mut s = state_clone.borrow_mut();
        frame_count += 1;
        
        // FPS calculation
        let current_time = performance.now();
        let delta = current_time - last_time;
        last_time = current_time;
        fps_accumulator += delta;
        fps_frame_count += 1;
        
        // Rescan DOM obstacles and exclusion zones occasionally
        if frame_count % 60 == 0 {
            s.obstacles = scan_dom_obstacles(&document_clone);
            let (zones, chakravyu) = scan_exclusion_zones(&document_clone);
            s.exclusion_zones = zones;
            s.chakravyu = chakravyu;
        }
        
        // Update dashboard every 30 frames
        if frame_count % 30 == 0 {
            let alive_count = s.arena.alive_count;
            
            if let Some(ref el) = stat_pop {
                el.set_text_content(Some(&format!("POP: {}", alive_count)));
            }
            
            // Find max generation
            let mut max_gen: u16 = 0;
            let mut max_speed: f32 = 0.0;
            for idx in s.arena.iter_alive() {
                max_gen = max_gen.max(s.arena.generation[idx]);
                max_speed = max_speed.max(s.arena.genes[idx].max_speed);
            }
            
            if let Some(ref el) = stat_gen {
                el.set_text_content(Some(&format!("GEN: {}", max_gen)));
            }
            
            if fps_frame_count > 0 && fps_accumulator > 0.0 {
                let avg_fps = (fps_frame_count as f64 / fps_accumulator) * 1000.0;
                if let Some(ref el) = stat_fps {
                    el.set_text_content(Some(&format!("FPS: {:.0}", avg_fps)));
                }
                fps_accumulator = 0.0;
                fps_frame_count = 0;
            }
            
            // Update season display
            if let Some(ref el) = stat_season {
                el.set_text_content(Some(&format!("SEASON: {}", s.season.season_name())));
            }
            
            // Log events
            if max_speed > stats.max_speed_record + 0.1 {
                stats.max_speed_record = max_speed;
                log_event(&document_clone, &format!("⚡ SPEED RECORD: {:.2}", max_speed), "event-record");
            }
            
            if max_gen > stats.max_generation {
                stats.max_generation = max_gen;
                if max_gen % 5 == 0 {
                    log_event(&document_clone, &format!("🧬 GEN {} reached", max_gen), "event-birth");
                }
            }
        }

        // Update canvas dimensions
        let canvas_w = ctx.canvas().unwrap().width() as f32;
        let canvas_h = ctx.canvas().unwrap().height() as f32;
        
        if s.width != canvas_w || s.height != canvas_h {
            s.width = canvas_w;
            s.height = canvas_h;
            s.grid.resize(canvas_w, canvas_h);
            s.fungal_network.resize(canvas_w, canvas_h);
            s.background.resize(canvas_w as f64, canvas_h as f64);
        }

        // === SIMULATION STEP ===
        
        // Destructure to get separate borrows
        let World { 
            arena, 
            grid, 
            obstacles,
            exclusion_zones,
            chakravyu,
            food_sources,
            fungal_network,
            background,
            predators,
            season,
            config, 
            width: world_w, 
            height: world_h,
            last_season,
            popups,
            ..
        } = &mut *s;
        
        // Update season
        season.update(1.0);
        
        // Check for season change
        let current_season = season.season_name();
        if current_season != *last_season {
            *last_season = current_season;
            log_event(&document_clone, &format!("🌍 {} has arrived!", current_season), "event-record");
            
            // Winter is harsh
            if current_season == "WINTER" {
                log_event(&document_clone, "❄ Resources are scarce...", "event-death");
            } else if current_season == "SUMMER" {
                log_event(&document_clone, "☀ Abundance! Food plentiful!", "event-birth");
            }
        }
        
        // Update Fungal Network with exclusion zones
        fungal_network.update_with_exclusions(exclusion_zones);
        
        // Boids interactions with network
        // 1. Spore Trail: Chance to seed new root at boid pos
        // 2. Infect / Interact: Boids contacting nodes
        
        // Collect interaction results and push forces first to avoid borrow conflicts
        let mut interactions = Vec::new();
        let mut push_forces: Vec<(usize, Vec2)> = Vec::new();
        let mut chakravyu_victims: Vec<usize> = Vec::new();
        
        // Get chakravyu zone info
        let chakravyu_zone = *chakravyu;
        
        // Collect side effects to apply later
        let mut energy_adjustments: Vec<(usize, f32)> = Vec::new();
        let mut moksh_candidates: Vec<usize> = Vec::new();

        for idx in arena.iter_alive() {
            let pos = arena.positions[idx];
            let role = arena.roles[idx];
            
            // Seed (Spore) - only herbivores spread spores, not in exclusion zones
            if role == BoidRole::Herbivore && !is_in_exclusion_zone(pos, exclusion_zones) {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                if rng.gen::<f32>() < 0.005 {
                    fungal_network.seed_at_safe(pos, exclusion_zones);
                }
            }
            
        // CHAKRAVYU MECHANICS - Simplified and Intense
        // 1. When dying/low energy, rush to center.
        // 2. Center heals/refills energy but traps them until they die naturally?
        // User request: "One an organism starts dying, it rushing towards the center. If it survives it's good else it just dies outside. The moment it is inside it just gets stuck full health and dies a natural death again."
        
        if let Some(chakravyu) = chakravyu_zone {
            let dist_to_center = pos.distance(chakravyu.center);
            
            // Inside the Chakravyu: TRAP + FULL HEALTH + NATURAL DEATH
            if dist_to_center < chakravyu.radius {
                // Trap forces: Strong inward pull + Tangential spin
                let inward = (chakravyu.center - pos).normalize() * chakravyu.inward_force * 3.0;
                let tangent = Vec2::new(-(chakravyu.center.y - pos.y), chakravyu.center.x - pos.x).normalize();
                let spin = tangent * 2.0;
                push_forces.push((idx, inward + spin));
                
                // Heal to full health constantly
                if arena.energy[idx] < 200.0 {
                    // Side effect: heal
                    energy_adjustments.push((idx, 5.0));
                }
                
                // They will die naturally from age or population pressure (base sim logic), 
                // but energy won't run out here.
                
                // Visual: if they just entered, pop a message? Or maybe when they die?
                // Let's keep the Swaaha/Moksh on death.
                
                // Special logic: If they die INSIDE from age, give them a special popup
                // But we can't detect "about to die from age" easily here without checking age vs max_age
                // Let's just let them spin.
                
            } else {
                // Outside: If dying (low energy), RUSH to center
                if arena.energy[idx] < 50.0 {
                    // Desperate rush to the sanctuary
                    let rush = (chakravyu.center - pos).normalize() * 2.0; // Strong pull
                    push_forces.push((idx, rush));
                }
            }
        }
            
            // Only push from exclusion zones for FUNGUS protection (icons)
            // But NOT for the central area - that's the Chakravyu trap
            for zone in exclusion_zones.iter() {
                let dist = pos.distance(zone.center);
                // Only push near the outer edge (icon protection), not deep inside
                if dist < zone.radius && dist > zone.radius * 0.8 && dist > 0.001 {
                    let push = (pos - zone.center).normalize() * 1.5;
                    push_forces.push((idx, push));
                }
            }
            
            // Check for interaction - only Herbivores and Scavengers eat fungus
            if (role == BoidRole::Herbivore || role == BoidRole::Scavenger) && frame_count % 2 == 0 {
                let result = fungal_network.interact(pos, BOID_SIZE * 3.0);
                if result != InteractionResult::None {
                    interactions.push((idx, result));
                }
            }
        }
        
        // Apply push forces
        for (idx, push) in push_forces {
            arena.velocities[idx] += push;
        }
        
        // Apply energy adjustments from side effects (e.g. Moksh fade)
        for (idx, amount) in energy_adjustments {
            arena.energy[idx] += amount;
        }
        
        // Apply Moksh deaths
        for idx in moksh_candidates {
            arena.energy[idx] = -100.0;
        }
        
        // Apply Chakravyu energy drain - rapid death inside the circle
        if let Some(chakravyu) = chakravyu_zone {
            for idx in chakravyu_victims {
                arena.energy[idx] -= chakravyu.energy_drain;
                // Accelerated death for those deep inside
                let dist = arena.positions[idx].distance(chakravyu.center);
                if dist < chakravyu.radius * 0.3 {
                    // Very close to center - instant death
                    arena.energy[idx] -= 5.0;
                }
            }
        }
        
        // Apply interactions
        for (idx, result) in interactions {
            match result {
                InteractionResult::Nutrient(amt) => {
                    arena.energy[idx] = (arena.energy[idx] + amt).min(200.0);
                },
                InteractionResult::Damage(amt) => {
                    arena.energy[idx] -= amt;
                },
                InteractionResult::Death => {
                    arena.energy[idx] = -100.0; // Ensure death in next sim step
                },
                InteractionResult::None => {}
            }
        }

        // Update predators
        for pred in predators.iter_mut() {
            pred.update(1.0);
        }
        predators.retain(|p| p.active);
        
        // 1. Build spatial grid
        grid.build(arena);
        
        // 2. Compute flocking forces (writes to arena.scratch_accel)
        compute_flocking_forces(arena, grid, VISION_RADIUS, obstacles);
        
        // 3. Feed from food sources (season-affected)
        // Replaced by feeding from fungal network? 
        // For now, let's keep food_sources empty and maybe add logic later to feed from network nodes.
        feed_from_sources(arena, food_sources, season);
        
        // Obstacle feeding - still works near monoliths
        let obstacle_feeders: Vec<usize> = (0..ARENA_CAPACITY)
            .filter(|&idx| arena.alive[idx])
            .filter(|&idx| {
                obstacles.iter().any(|obs| {
                    arena.positions[idx].distance(obs.center) < 150.0
                })
            })
            .collect();
        
        for idx in obstacle_feeders {
            arena.energy[idx] = (arena.energy[idx] + 0.8 * season.food_multiplier()).min(200.0);
        }
        
        // Apply predator damage
        let predator_kills = apply_predator_zones(arena, predators);
        if predator_kills > 0 {
            log_event(&document_clone, &format!("🩸 Predator claimed {} victims!", predator_kills), "event-death");
        }
        
        // 4. Run simulation step (movement, reproduction, death)
        let (births, deaths) = simulation_step(
            arena,
            grid,
            config,
            *world_w,
            *world_h,
            1.0,
        );
        
        if deaths > 15 {
            log_event(&document_clone, &format!("☠ {} died", deaths), "event-death");
        }
        
        let _ = births; // Suppress unused warnings
        
        // === MASS EXTINCTION CHECK ===
        // When diversity collapses, trigger a reset event
        if frame_count % 60 == 0 && arena.alive_count > 50 {
            let diversity = compute_diversity(arena);
            
            if diversity < 0.25 {
                stats.low_diversity_frames += 1;
                
                // Sustained low diversity triggers extinction
                if stats.low_diversity_frames > 10 {
                    log_event(&document_clone, "☄ MASS EXTINCTION - Ecosystem collapsing!", "event-death");
                    trigger_mass_extinction(arena, 0.8, *world_w, *world_h); // Kill 80%
                    
                    // Also trim the fungal network
                    for node in fungal_network.nodes.iter_mut() {
                        if node.active {
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            if rng.gen::<f32>() < 0.5 {
                                node.active = false;
                            }
                        }
                    }
                    
                    stats.low_diversity_frames = 0;
                    log_event(&document_clone, "🌱 New founders seeded...", "event-birth");
                }
            } else {
                // Reset counter if diversity recovers
                stats.low_diversity_frames = 0;
            }
        }

        // === RENDERING ===
        
        // Update background effect
        background.update(0.016); // Approx 60fps dt
        background.draw(&ctx);
        
        // Draw Fungal Network
        fungal_network.draw(&ctx);
        
        // Draw predators
        for pred in &s.predators {
            if !pred.active { continue; }
            
            let pulse = 0.5 + 0.5 * (pred.lifetime * 5.0).sin();
            let alpha = 0.3 * pulse;
            
            // Tech Danger Zone
            ctx.set_stroke_style(&JsValue::from_str(&format!("rgba(255, 0, 50, {})", alpha)));
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.arc(pred.position.x as f64, pred.position.y as f64, pred.radius as f64, 0.0, std::f64::consts::TAU).unwrap();
            ctx.stroke();
        }

        // Draw Chakravyu Sanskrit Shield
        if let Some(chakravyu) = s.chakravyu {
            ctx.save();
            ctx.translate(chakravyu.center.x as f64, chakravyu.center.y as f64).unwrap();
            
            // Slow rotation for the text ring
            let time = performance.now() * 0.0002;
            ctx.rotate(time).unwrap();
            
            ctx.set_font("14px 'IBM Plex Mono', monospace");
            ctx.set_fill_style(&JsValue::from_str("rgba(0, 255, 170, 0.3)"));
            ctx.set_text_align("center");
            
            // "रागद्वेषवियुक्तैस्तु विषयानिन्द्रियैश्चरन्। आत्मवश्यैर्विधेयात्मा प्रसादमधिगच्छति॥"
            // Split into code-like segments
            let text = "::रागद्वेषवियुक्तैस्तु::void* // <आत्मवश्यैर्विधेयात्मा>; // fn(प्रसादमधिगच्छति) -> Peace";
            
            // Draw text in a circle
            let radius = chakravyu.radius as f64 - 10.0;
            let chars: Vec<char> = text.chars().collect();
            let angle_step = std::f64::consts::TAU / (chars.len() as f64);
            
            for (i, char) in chars.iter().enumerate() {
                ctx.save();
                let angle = i as f64 * angle_step;
                ctx.rotate(angle).unwrap();
                ctx.translate(0.0, -radius).unwrap();
                ctx.fill_text(&char.to_string(), 0.0, 0.0).unwrap();
                ctx.restore();
            }
            
            // Inner faint shield circle
            ctx.begin_path();
            ctx.arc(0.0, 0.0, radius - 15.0, 0.0, std::f64::consts::TAU).unwrap();
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 170, 0.1)"));
            ctx.set_line_width(1.0);
            ctx.stroke();
            
            ctx.restore();
        }

        // Update and draw popups
        s.popups.retain_mut(|p| {
            p.life -= 0.02;
            p.pos.y -= 0.5; // Float up
            p.life > 0.0
        });
        
        ctx.set_font("bold 12px 'IBM Plex Mono', monospace");
        ctx.set_text_align("center");
        for p in &s.popups {
            let alpha = p.life;
            // Replace the placeholder {} with alpha
            let color = p.color.replace("{}", &alpha.to_string());
            ctx.set_fill_style(&JsValue::from_str(&color));
            ctx.fill_text(&p.text, p.pos.x as f64, p.pos.y as f64).unwrap();
        }

        // Draw Organisms (Boids)
        for idx in s.arena.iter_alive() {
            let pos = s.arena.positions[idx];
            let vel = s.arena.velocities[idx];
            let angle = vel.y.atan2(vel.x);
            let (hue, sat, light) = get_boid_color(&s.arena, idx);
            let role = s.arena.roles[idx];
            let state = s.arena.states[idx];
            let size_mult = s.arena.genes[idx].size;
            
            let color = format!("hsl({}, {}%, {}%)", hue, sat, light);
            draw_organism(
                &ctx, 
                pos.x as f64, 
                pos.y as f64, 
                angle as f64, 
                &color, 
                BOID_SIZE as f64,
                role,
                state,
                size_mult,
            );
        }
        
        // Trails
        ctx.set_global_alpha(0.2);
        for idx in s.arena.iter_alive() {
            if s.arena.energy[idx] > 100.0 {
                let pos = s.arena.positions[idx];
                let vel = s.arena.velocities[idx];
                let speed = vel.length();
                if speed > 2.0 {
                    let trail_end = pos - vel.normalize() * speed * 8.0; // Longer trails
                    
                    ctx.begin_path();
                    ctx.move_to(pos.x as f64, pos.y as f64);
                    ctx.line_to(trail_end.x as f64, trail_end.y as f64);
                    
                    let (h, s_val, l) = get_boid_color(&s.arena, idx);
                    ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, {}%, {}%)", h, s_val, l)));
                    ctx.set_line_width(1.0);
                    ctx.stroke();
                }
            }
        }
        ctx.set_global_alpha(1.0);

        if !paused {
            web_sys::window()
                .unwrap()
                .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .unwrap();
        }
    }));

    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
