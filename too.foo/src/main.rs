use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, Document, HtmlElement, Performance};
use antimony_core::{
    BoidArena, SpatialGrid, Obstacle, FoodSource, Genome, SimConfig,
    SeasonCycle, PredatorZone,
    compute_flocking_forces, simulation_step, feed_from_sources, get_boid_color,
    apply_predator_zones, trigger_migration, trigger_earthquake,
};
use glam::Vec2;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Fixed capacity - no runtime allocations
const ARENA_CAPACITY: usize = 1024;
const CELL_CAPACITY: usize = 32;

// --- Fungal Growth System ---
// Simple grid-based growth simulation (Cellular Automata)
const FUNGAL_GRID_SIZE: usize = 100; // 100x100 grid overlay
const FUNGAL_UPDATE_INTERVAL: u32 = 5; // Update every N frames

struct FungalGrid {
    // 0 = empty, >0 = biomass (0-255)
    cells: Vec<u8>,
    width: usize,
    height: usize,
    cell_size_x: f32,
    cell_size_y: f32,
    update_timer: u32,
}

impl FungalGrid {
    fn new(width: usize, height: usize, screen_w: f32, screen_h: f32) -> Self {
        Self {
            cells: vec![0; width * height],
            width,
            height,
            cell_size_x: screen_w / width as f32,
            cell_size_y: screen_h / height as f32,
            update_timer: 0,
        }
    }

    fn resize(&mut self, screen_w: f32, screen_h: f32) {
        self.cell_size_x = screen_w / self.width as f32;
        self.cell_size_y = screen_h / self.height as f32;
    }

    fn seed(&mut self, x: f32, y: f32, amount: u8) {
        let cx = (x / self.cell_size_x) as usize;
        let cy = (y / self.cell_size_y) as usize;
        if cx < self.width && cy < self.height {
            let idx = cy * self.width + cx;
            self.cells[idx] = self.cells[idx].saturating_add(amount);
        }
    }

    fn update(&mut self) {
        self.update_timer += 1;
        if self.update_timer < FUNGAL_UPDATE_INTERVAL {
            return;
        }
        self.update_timer = 0;

        let mut next_cells = self.cells.clone();
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let val = self.cells[idx];
                
                if val > 10 {
                    // Spread to neighbors if healthy
                    // Random spread logic
                    let spread_chance = if val > 200 { 0.2 } else { 0.05 };
                    if js_sys::Math::random() < spread_chance {
                        // Pick random neighbor
                        let dx = (js_sys::Math::random() * 3.0) as i32 - 1; // -1, 0, 1
                        let dy = (js_sys::Math::random() * 3.0) as i32 - 1;
                        
                        let nx = (x as i32 + dx).clamp(0, self.width as i32 - 1) as usize;
                        let ny = (y as i32 + dy).clamp(0, self.height as i32 - 1) as usize;
                        let nidx = ny * self.width + nx;
                        
                        // Grow into empty space or reinforce
                        if next_cells[nidx] < 200 {
                            next_cells[nidx] = next_cells[nidx].saturating_add(15);
                            // Cost to parent
                            next_cells[idx] = next_cells[idx].saturating_sub(2);
                        }
                    }
                } else if val > 0 {
                    // Decay if weak
                    next_cells[idx] = next_cells[idx].saturating_sub(1);
                }
            }
        }
        self.cells = next_cells;
    }

    // Cut the fungus at a position (Robot cutting)
    fn cut(&mut self, x: f32, y: f32, radius: f32) {
        let cx_start = ((x - radius) / self.cell_size_x).max(0.0) as usize;
        let cx_end = ((x + radius) / self.cell_size_x).min(self.width as f32) as usize;
        let cy_start = ((y - radius) / self.cell_size_y).max(0.0) as usize;
        let cy_end = ((y + radius) / self.cell_size_y).min(self.height as f32) as usize;

        for cy in cy_start..cy_end {
            for cx in cx_start..cx_end {
                let idx = cy * self.width + cx;
                self.cells[idx] = 0; // Kill fungus instantly
            }
        }
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        // Draw as a texture or simple rects for now
        // Optimization: only fill rects, don't stroke
        ctx.set_fill_style(&JsValue::from_str("rgba(50, 200, 100, 0.15)"));
        
        for y in 0..self.height {
            for x in 0..self.width {
                let val = self.cells[y * self.width + x];
                if val > 20 {
                    let alpha = (val as f32 / 255.0) * 0.3;
                    ctx.set_fill_style(&JsValue::from_str(&format!("rgba(50, 255, 100, {})", alpha)));
                    ctx.fill_rect(
                        x as f64 * self.cell_size_x as f64, 
                        y as f64 * self.cell_size_y as f64, 
                        self.cell_size_x as f64 + 0.5, // +0.5 to avoid gaps
                        self.cell_size_y as f64 + 0.5
                    );
                }
            }
        }
    }
}

/// Simulation state tracking
struct SimulationStats {
    max_speed_record: f32,
    max_generation: u16,
    total_births: u64,
    total_deaths: u64,
}

/// Append a log event to the console-log div
fn log_event(document: &Document, msg: &str, event_class: &str) {
    if let Some(console_log) = document.get_element_by_id("console-log") {
        if let Ok(p) = document.create_element("p") {
            p.set_text_content(Some(msg));
            let _ = p.set_attribute("class", event_class);
            let _ = console_log.append_child(&p);
            
            if let Ok(html_el) = console_log.dyn_into::<HtmlElement>() {
                html_el.set_scroll_top(html_el.scroll_height());
            }
        }
    }
}

struct World {
    arena: BoidArena<ARENA_CAPACITY>,
    grid: SpatialGrid<CELL_CAPACITY>,
    obstacles: Vec<Obstacle>,
    food_sources: Vec<FoodSource>,
    fungal_grid: FungalGrid, // NEW: Full screen fungal growth
    predators: Vec<PredatorZone>,
    season: SeasonCycle,
    config: SimConfig,
    width: f32,
    height: f32,
    event_cooldown: f32,
    last_season: &'static str,
}

const BOID_SIZE: f32 = 6.0;
const VISION_RADIUS: f32 = 60.0;

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

// --- Rendering Functions ---

fn draw_fungal_colony(ctx: &CanvasRenderingContext2d, x: f64, y: f64, radius: f64, hue: u16, fullness: f32, time: f64) {
    // Draw the "Source" node of the fungus (High detail)
    ctx.save();
    ctx.translate(x, y).unwrap();
    
    let breath = 1.0 + 0.05 * (time * 1.5).sin();
    ctx.scale(breath, breath).unwrap();

    let seed = (x * y).abs() as u32; 
    let strands = 12 + (fullness * 10.0) as i32;
    ctx.set_stroke_style(&JsValue::from_str(&format!("hsla({}, 70%, 60%, 0.4)", hue)));
    ctx.set_line_width(1.0);
    
    for i in 0..strands {
        let angle = (i as f64 / strands as f64) * std::f64::consts::TAU + (seed as f64 % 10.0);
        let len = radius * (fullness as f64) * (0.8 + 0.4 * ((i as f64 * 1.32).sin())); 
        
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        let cp_len = len * 0.5;
        let cp_angle = angle + 0.5 * ((time * 0.2 + i as f64).sin());
        let end_x = angle.cos() * len;
        let end_y = angle.sin() * len;
        let cp_x = cp_angle.cos() * cp_len;
        let cp_y = cp_angle.sin() * cp_len;
        ctx.quadratic_curve_to(cp_x, cp_y, end_x, end_y);
        ctx.stroke();
        
        ctx.begin_path();
        ctx.arc(end_x, end_y, (2.0 + 2.0 * fullness).into(), 0.0, std::f64::consts::TAU).unwrap();
        ctx.set_fill_style(&JsValue::from_str(&format!("hsla({}, 90%, 80%, 0.8)", hue)));
        ctx.fill();
    }
    
    let core_radius = radius * 0.3 * (fullness as f64);
    if core_radius > 0.0 {
        let gradient = ctx.create_radial_gradient(0.0, 0.0, 0.0, 0.0, 0.0, core_radius * 2.0).unwrap();
        gradient.add_color_stop(0.0, &format!("hsla({}, 90%, 60%, 0.8)", hue)).unwrap();
        gradient.add_color_stop(1.0, &format!("hsla({}, 90%, 60%, 0.0)", hue)).unwrap();
        
        ctx.set_fill_style(&gradient);
        ctx.begin_path();
        ctx.arc(0.0, 0.0, core_radius * 2.0, 0.0, std::f64::consts::TAU).unwrap();
        ctx.fill();
    }

    ctx.restore();
}

fn draw_robot_boid(ctx: &CanvasRenderingContext2d, x: f64, y: f64, angle: f64, color: &str, size: f64) {
    ctx.save();
    ctx.translate(x, y).unwrap();
    ctx.rotate(angle).unwrap();
    
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(1.5);
    
    ctx.begin_path();
    ctx.move_to(-size, -size * 0.8);
    ctx.line_to(size, 0.0);
    ctx.line_to(-size, size * 0.8);
    ctx.line_to(-size * 0.5, 0.0);
    ctx.close_path();
    
    ctx.stroke();
    
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.8)"));
    ctx.begin_path();
    ctx.arc(-size * 0.5, 0.0, size * 0.3, 0.0, std::f64::consts::TAU).unwrap();
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

    let w = window.inner_width().unwrap().as_f64().unwrap();
    let h = window.inner_height().unwrap().as_f64().unwrap();
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);

    // Resize handler
    {
        let canvas = canvas.clone();
        let window_for_closure = window.clone();
        let closure = Closure::wrap(Box::new(move || {
            let w = window_for_closure.inner_width().unwrap().as_f64().unwrap();
            let h = window_for_closure.inner_height().unwrap().as_f64().unwrap();
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

    // Initialize arena with starting population
    let mut arena: BoidArena<ARENA_CAPACITY> = BoidArena::new();
    let mut rng = rand::thread_rng();
    use rand::Rng;
    
    for _ in 0..150 {
        let pos = Vec2::new(
            rng.gen_range(0.0..width),
            rng.gen_range(0.0..height),
        );
        let vel = Vec2::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );
        arena.spawn(pos, vel, Genome::random());
    }

    let grid = SpatialGrid::new(width, height, VISION_RADIUS);
    let obstacles = scan_dom_obstacles(&document);
    
    let food_sources = vec![
        FoodSource::new(width * 0.25, height * 0.25),
        FoodSource::new(width * 0.75, height * 0.25),
        FoodSource::new(width * 0.25, height * 0.75),
        FoodSource::new(width * 0.75, height * 0.75),
        FoodSource::new(width * 0.5, height * 0.5),
    ];

    // Initialize Fungal Grid
    let mut fungal_grid = FungalGrid::new(FUNGAL_GRID_SIZE, FUNGAL_GRID_SIZE, width, height);
    // Seed fungal growth at food sources
    for src in &food_sources {
        fungal_grid.seed(src.position.x, src.position.y, 200);
    }

    let mut config = SimConfig::default();
    config.reproduction_threshold = 140.0;
    config.base_mortality = 0.00005;

    let state = Rc::new(RefCell::new(World {
        arena,
        grid,
        obstacles,
        food_sources,
        fungal_grid,
        predators: Vec::new(),
        season: SeasonCycle::new(),
        config,
        width,
        height,
        event_cooldown: 0.0,
        last_season: "SPRING",
    }));

    // Cache DOM element references
    let stat_pop = document.get_element_by_id("stat-pop");
    let stat_gen = document.get_element_by_id("stat-gen");
    let stat_fps = document.get_element_by_id("stat-fps");

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
        total_births: 0,
        total_deaths: 0,
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
        
        let time_sec = current_time / 1000.0;

        // Rescan DOM obstacles
        if frame_count % 60 == 0 {
            s.obstacles = scan_dom_obstacles(&document_clone);
        }
        
        // Update dashboard
        if frame_count % 30 == 0 {
            let alive_count = s.arena.alive_count;
            if let Some(ref el) = stat_pop {
                el.set_text_content(Some(&format!("POP: {}", alive_count)));
            }
            
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
            s.fungal_grid.resize(canvas_w, canvas_h); // Resize fungal grid too
        }

        // === SIMULATION STEP ===
        
        let World { 
            arena, 
            grid, 
            obstacles, 
            food_sources,
            fungal_grid,
            predators,
            season,
            config, 
            width: world_w, 
            height: world_h,
            event_cooldown,
            last_season,
        } = &mut *s;
        
        season.update(1.0);
        
        // Seed fungus from active food sources occasionally
        if frame_count % 10 == 0 {
            for src in food_sources.iter() {
                if src.energy > 0.0 {
                    fungal_grid.seed(src.position.x, src.position.y, 10);
                }
            }
        }
        
        // Update Fungal Grid
        fungal_grid.update();

        // Check for season change
        let current_season = season.season_name();
        if current_season != *last_season {
            *last_season = current_season;
            log_event(&document_clone, &format!("🌍 {} has arrived!", current_season), "event-record");
            
            if current_season == "WINTER" {
                log_event(&document_clone, "❄ Resources are scarce...", "event-death");
            } else if current_season == "SUMMER" {
                log_event(&document_clone, "☀ Abundance! Food plentiful!", "event-birth");
            }
        }
        
        // Random events (Code omitted for brevity, same as before)
        *event_cooldown -= 1.0;
        if *event_cooldown <= 0.0 {
            // ... (Keep existing event logic)
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let event_chance = 0.002;
            if rng.gen::<f32>() < event_chance {
                 *event_cooldown = 200.0; // Basic reset
            }
        }
        
        // Update predators
        for pred in predators.iter_mut() {
            pred.update(1.0);
        }
        predators.retain(|p| p.active);
        
        // 1. Build spatial grid
        grid.build(arena);
        
        // 2. Compute flocking forces
        compute_flocking_forces(arena, grid, VISION_RADIUS, obstacles);
        
        // 3. Feed from food sources
        feed_from_sources(arena, food_sources, season);
        
        // 4. Robot Interaction with Fungus (Cutting)
        // Iterate over all alive boids and cut the fungus at their position
        for idx in arena.iter_alive() {
            let pos = arena.positions[idx];
            fungal_grid.cut(pos.x, pos.y, BOID_SIZE * 2.0);
        }
        
        // Feed near obstacles
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
        
        // 5. Run simulation step
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
        let _ = births;

        // === RENDERING ===
        
        // Background
        ctx.set_fill_style(&JsValue::from_str("#0a0a12"));
        ctx.fill_rect(0.0, 0.0, canvas_w as f64, canvas_h as f64);
        
        // Draw Fungal Grid Overlay (Background Layer)
        fungal_grid.draw(&ctx);
        
        // Draw food sources (Fungal Cores)
        let season_hue = match s.season.season_name() {
            "SPRING" => 140,
            "SUMMER" => 60,
            "AUTUMN" => 30,
            "WINTER" => 200,
            _ => 140,
        };
        
        for food in &s.food_sources {
            if food.energy > 0.0 {
                draw_fungal_colony(&ctx, food.position.x as f64, food.position.y as f64, 
                    food.radius as f64, season_hue, food.fullness(), time_sec);
            }
        }
        
        // Draw predators
        for pred in &s.predators {
            if !pred.active { continue; }
            let alpha = 0.3 * (1.0 + (pred.lifetime * 5.0).sin());
            ctx.set_stroke_style(&JsValue::from_str(&format!("rgba(255, 0, 50, {})", alpha)));
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.arc(pred.position.x as f64, pred.position.y as f64, pred.radius as f64, 0.0, std::f64::consts::TAU).unwrap();
            ctx.stroke();
        }

        // Draw Robots
        for idx in s.arena.iter_alive() {
            let pos = s.arena.positions[idx];
            let vel = s.arena.velocities[idx];
            let angle = vel.y.atan2(vel.x);
            let (hue, sat, light) = get_boid_color(&s.arena, idx);
            let color = format!("hsl({}, {}%, {}%)", hue, sat, light);
            draw_robot_boid(&ctx, pos.x as f64, pos.y as f64, angle as f64, &color, BOID_SIZE as f64);
        }
        
        // Trails
        ctx.set_global_alpha(0.2);
        for idx in s.arena.iter_alive() {
            if s.arena.energy[idx] > 100.0 {
                let pos = s.arena.positions[idx];
                let vel = s.arena.velocities[idx];
                let speed = vel.length();
                if speed > 2.0 {
                    let trail_end = pos - vel.normalize() * speed * 8.0;
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
