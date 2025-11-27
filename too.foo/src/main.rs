use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen::Clamped;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, Document, HtmlElement, Performance, ImageData};
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

// --- Fungal Growth System (Reaction-Diffusion) ---
const SIM_WIDTH: usize = 256;
const SIM_HEIGHT: usize = 144; 

struct FungalGrid {
    cells: Vec<f32>,
    next_cells: Vec<f32>,
    width: usize,
    height: usize,
    image_data: Vec<u8>,
}

impl FungalGrid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![0.0; width * height],
            next_cells: vec![0.0; width * height],
            width,
            height,
            image_data: vec![0; width * height * 4],
        }
    }

    fn seed(&mut self, x_pct: f32, y_pct: f32, amount: f32) {
        let cx = (x_pct * self.width as f32) as usize;
        let cy = (y_pct * self.height as f32) as usize;
        
        if cx < self.width && cy < self.height {
            let idx = cy * self.width + cx;
            self.cells[idx] = (self.cells[idx] + amount).min(100.0);
        }
    }

    fn update(&mut self) {
        let w = self.width as i32;
        let h = self.height as i32;
        let diffusion_rate = 0.2;
        let decay_rate = 0.015;
        let growth_rate = 1.01;

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let val = self.cells[idx];

                let mut sum = 0.0;
                let mut neighbors: f32 = 0.0;

                if x > 0 { sum += self.cells[idx - 1]; neighbors += 1.0; }
                if x < w - 1 { sum += self.cells[idx + 1]; neighbors += 1.0; }
                if y > 0 { sum += self.cells[idx - self.width]; neighbors += 1.0; }
                if y < h - 1 { sum += self.cells[idx + self.width]; neighbors += 1.0; }

                let avg = sum / neighbors.max(1.0);
                let mut new_val = val + (avg - val) * diffusion_rate;
                
                if new_val > 5.0 && new_val < 80.0 {
                    new_val *= growth_rate;
                }
                new_val -= decay_rate;
                self.next_cells[idx] = new_val.clamp(0.0, 100.0);
            }
        }
        std::mem::swap(&mut self.cells, &mut self.next_cells);
    }

    fn cut(&mut self, x_pct: f32, y_pct: f32, radius_pct: f32) {
        let radius_cells = (radius_pct * self.width as f32) as i32;
        let cx = (x_pct * self.width as f32) as i32;
        let cy = (y_pct * self.height as f32) as i32;

        let start_x = (cx - radius_cells).max(0);
        let end_x = (cx + radius_cells).min(self.width as i32);
        let start_y = (cy - radius_cells).max(0);
        let end_y = (cy + radius_cells).min(self.height as i32);

        for y in start_y..end_y {
            for x in start_x..end_x {
                let dx = x - cx;
                let dy = y - cy;
                if dx*dx + dy*dy < radius_cells*radius_cells {
                    let idx = (y * self.width as i32 + x) as usize;
                    self.cells[idx] = 0.0; 
                }
            }
        }
    }

    fn render_to_buffer(&mut self) {
        for i in 0..self.cells.len() {
            let val = self.cells[i];
            let pixel_idx = i * 4;
            
            if val < 1.0 {
                self.image_data[pixel_idx + 3] = 0;
            } else {
                let intensity = (val / 100.0).clamp(0.0, 1.0);
                self.image_data[pixel_idx] = (20.0 + intensity * 80.0) as u8;
                self.image_data[pixel_idx + 1] = (50.0 + intensity * 205.0) as u8;
                self.image_data[pixel_idx + 2] = (40.0 + intensity * 110.0) as u8;
                self.image_data[pixel_idx + 3] = (intensity * 255.0) as u8;
            }
        }
    }
    
    fn draw(&mut self, _ctx: &CanvasRenderingContext2d, _canvas_w: f32, _canvas_h: f32) -> ImageData {
        self.render_to_buffer();
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.image_data), 
            self.width as u32, 
            self.height as u32
        ).unwrap()
    }
}

struct SimulationStats {
    max_speed_record: f32,
    max_generation: u16,
}

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
    fungal_grid: FungalGrid, 
    predators: Vec<PredatorZone>,
    season: SeasonCycle,
    config: SimConfig,
    width: f32,
    height: f32,
    event_cooldown: f32,
    last_season: &'static str,
    offscreen_canvas: HtmlCanvasElement, 
    offscreen_ctx: CanvasRenderingContext2d,
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
            obstacles.push(Obstacle { center: Vec2::new(center_x, center_y), radius });
        }
    }
    obstacles
}

fn is_paused() -> bool {
    let window = window().unwrap();
    if let Ok(search) = window.location().search() {
        search.contains("paused=true")
    } else {
        false
    }
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
    let canvas = document.get_element_by_id("simulation").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
    let paused = is_paused();

    let w = window.inner_width().unwrap().as_f64().unwrap();
    let h = window.inner_height().unwrap().as_f64().unwrap();
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);

    let offscreen_canvas = document.create_element("canvas").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
    offscreen_canvas.set_width(SIM_WIDTH as u32);
    offscreen_canvas.set_height(SIM_HEIGHT as u32);
    let offscreen_ctx = offscreen_canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();

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

    let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
    ctx.set_image_smoothing_enabled(true);

    let width = w as f32;
    let height = h as f32;

    let mut arena: BoidArena<ARENA_CAPACITY> = BoidArena::new();
    let mut rng = rand::thread_rng();
    use rand::Rng;
    
    for _ in 0..150 {
        let pos = Vec2::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height));
        let vel = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
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

    let mut fungal_grid = FungalGrid::new(SIM_WIDTH, SIM_HEIGHT);
    fungal_grid.seed(0.5, 0.5, 100.0);
    fungal_grid.seed(0.2, 0.2, 80.0);
    fungal_grid.seed(0.8, 0.8, 80.0);

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
        offscreen_canvas,
        offscreen_ctx,
    }));

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
    };
    
    *g.borrow_mut() = Some(Closure::new(move || {
        let mut s = state_clone.borrow_mut();
        frame_count += 1;
        
        let current_time = performance.now();
        let delta = current_time - last_time;
        last_time = current_time;
        fps_accumulator += delta;
        fps_frame_count += 1;
        
        if frame_count % 60 == 0 {
            s.obstacles = scan_dom_obstacles(&document_clone);
        }
        
        if frame_count % 30 == 0 {
            let alive_count = s.arena.alive_count;
            if let Some(ref el) = stat_pop { el.set_text_content(Some(&format!("POP: {}", alive_count))); }
            
            let mut max_gen: u16 = 0;
            let mut max_speed: f32 = 0.0;
            for idx in s.arena.iter_alive() {
                max_gen = max_gen.max(s.arena.generation[idx]);
                max_speed = max_speed.max(s.arena.genes[idx].max_speed);
            }
            
            if let Some(ref el) = stat_gen { el.set_text_content(Some(&format!("GEN: {}", max_gen))); }
            
            if fps_frame_count > 0 && fps_accumulator > 0.0 {
                let avg_fps = (fps_frame_count as f64 / fps_accumulator) * 1000.0;
                if let Some(ref el) = stat_fps { el.set_text_content(Some(&format!("FPS: {:.0}", avg_fps))); }
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

        let canvas_w = ctx.canvas().unwrap().width() as f32;
        let canvas_h = ctx.canvas().unwrap().height() as f32;
        
        if s.width != canvas_w || s.height != canvas_h {
            s.width = canvas_w;
            s.height = canvas_h;
            s.grid.resize(canvas_w, canvas_h);
        }

        // === SIMULATION STEP ===
        
        s.season.update(1.0);
        let width = s.width;
        let height = s.height;
        
            // Continuous seeding from food sources
            if frame_count % 2 == 0 {
                // Collect data first to avoid borrow conflicts
                let seeds: Vec<(f32, f32, f32)> = s.food_sources.iter()
                    .filter(|src| src.energy > 0.0)
                    .map(|src| (src.position.x, src.position.y, 5.0))
                    .collect();
                    
                for (x, y, amt) in seeds {
                    let x_pct = x / width;
                    let y_pct = y / height;
                    s.fungal_grid.seed(x_pct, y_pct, amt);
                }
            }
            
            s.fungal_grid.update();

            for pred in s.predators.iter_mut() {
                pred.update(1.0);
            }
            s.predators.retain(|p| p.active);
            
            // Collect necessary data for robot cutting
            let cuts: Vec<(f32, f32)> = s.arena.iter_alive()
                .map(|idx| (s.arena.positions[idx].x, s.arena.positions[idx].y))
                .collect();
                
            let boid_radius_pct = (BOID_SIZE * 2.5) / width;
            for (x, y) in cuts {
                let x_pct = x / width;
                let y_pct = y / height;
                s.fungal_grid.cut(x_pct, y_pct, boid_radius_pct);
            }

            // Scope for simulation logic to satisfy borrow checker
            // We can't borrow individual fields mutably from `s` because it's a `RefMut<World>`.
            // We must destructure `s` to get mutable references to its fields.
            // Since `s` is a `RefMut`, we can destructure `*s`.
            {
                let World {
                    arena,
                    grid,
                    obstacles,
                    food_sources,
                    predators,
                    season,
                    config,
                    event_cooldown,
                    last_season,
                    ..
                } = &mut *s;

                grid.build(arena);
                compute_flocking_forces(arena, grid, VISION_RADIUS, obstacles);
                feed_from_sources(arena, food_sources, season);
                
                // Random events
                *event_cooldown -= 1.0;
                if *event_cooldown <= 0.0 {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    
                    let event_chance = 0.002;
                    
                    if rng.gen::<f32>() < event_chance {
                        *event_cooldown = 200.0;
                    }
                }
                
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
                
                apply_predator_zones(arena, predators);
                
                simulation_step(
                    arena,
                    grid,
                    config,
                    width,
                    height,
                    1.0,
                );
            }

        // === RENDERING ===
        
        // Background
        ctx.set_fill_style(&JsValue::from_str("#050508"));
        ctx.fill_rect(0.0, 0.0, canvas_w as f64, canvas_h as f64);
        
        // 1. Draw Fungal Layer
        let img_data = s.fungal_grid.draw(&ctx, canvas_w, canvas_h);
        s.offscreen_ctx.put_image_data(&img_data, 0.0, 0.0).unwrap();
        ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
            &s.offscreen_canvas, 
            0.0, 0.0, canvas_w as f64, canvas_h as f64
        ).unwrap();
        
        // 2. Robots
        for idx in s.arena.iter_alive() {
            let pos = s.arena.positions[idx];
            let vel = s.arena.velocities[idx];
            let angle = vel.y.atan2(vel.x);
            let (hue, sat, light) = get_boid_color(&s.arena, idx);
            let color = format!("hsl({}, {}%, {}%)", hue, sat, light);
            draw_robot_boid(&ctx, pos.x as f64, pos.y as f64, angle as f64, &color, BOID_SIZE as f64);
        }
        
        // 3. Predators
        for pred in &s.predators {
            if !pred.active { continue; }
            let alpha = 0.3 * (1.0 + (pred.lifetime * 5.0).sin());
            ctx.set_stroke_style(&JsValue::from_str(&format!("rgba(255, 0, 50, {})", alpha)));
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.arc(pred.position.x as f64, pred.position.y as f64, pred.radius as f64, 0.0, std::f64::consts::TAU).unwrap();
            ctx.stroke();
        }

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
