use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, Document, HtmlElement, Performance};
use antimony_core::{Boid, Obstacle};
use glam::Vec2;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Simulation state tracking
struct SimulationStats {
    max_speed_record: f32,
    initial_population: usize,
    extinction_triggered: bool,
}

impl SimulationStats {
    fn new(initial_pop: usize) -> Self {
        Self {
            max_speed_record: 0.0,
            initial_population: initial_pop,
            extinction_triggered: false,
        }
    }
}

/// Append a log event to the console-log div
fn log_event(document: &Document, msg: &str, event_class: &str) {
    if let Some(console_log) = document.get_element_by_id("console-log") {
        // Create a new paragraph element
        if let Ok(p) = document.create_element("p") {
            p.set_text_content(Some(msg));
            let _ = p.set_attribute("class", event_class);
            let _ = console_log.append_child(&p);
            
            // Auto-scroll to bottom by setting scrollTop to scrollHeight
            if let Ok(html_el) = console_log.dyn_into::<HtmlElement>() {
                html_el.set_scroll_top(html_el.scroll_height());
            }
        }
    }
}

struct World {
    boids: Vec<Boid>,
    obstacles: Vec<Obstacle>,
    width: f32,
    height: f32,
}

const BOID_SIZE: f32 = 5.0;

fn scan_dom_obstacles(document: &Document) -> Vec<Obstacle> {
    let mut obstacles = Vec::new();
    let elements = document.get_elements_by_class_name("monolith");
    
    for i in 0..elements.length() {
        if let Some(element) = elements.item(i) {
            let rect = element.get_bounding_client_rect();
            
            // Convert to canvas coordinates (already in pixels from bounding rect)
            let center_x = rect.left() as f32 + rect.width() as f32 / 2.0;
            let center_y = rect.top() as f32 + rect.height() as f32 / 2.0;
            
            // Calculate radius as max of half width/height
            let radius = (rect.width().max(rect.height()) as f32) / 2.0;
            
            obstacles.push(Obstacle {
                center: Vec2::new(center_x, center_y),
                radius,
            });
        }
    }
    obstacles
}

/// Check if the URL contains ?paused=true query parameter
fn is_paused() -> bool {
    let window = window().unwrap();
    let location = window.location();
    if let Ok(search) = location.search() {
        search.contains("paused=true")
    } else {
        false
    }
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

    // Check if we should run in paused mode (for E2E testing)
    let paused = is_paused();

    // Resize canvas to fill window
    let w = window.inner_width().unwrap().as_f64().unwrap();
    let h = window.inner_height().unwrap().as_f64().unwrap();
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);

    // Attach resize event listener
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

    // Initialize world with 100 boids
    let mut boids = Vec::new();
    let width = w as f32;
    let height = h as f32;
    
    for _ in 0..100 {
        let mut boid = Boid::new();
        // Scale positions from 0.0-1.0 range to canvas coordinates
        boid.position.x *= width;
        boid.position.y *= height;
        boids.push(boid);
    }
    
    let obstacles = scan_dom_obstacles(&document);

    let initial_pop = boids.len();
    
    let state = Rc::new(RefCell::new(World {
        boids,
        obstacles,
        width,
        height,
    }));

    // Cache DOM element references for the dashboard
    let stat_pop = document.get_element_by_id("stat-pop");
    let stat_gen = document.get_element_by_id("stat-gen");
    let stat_fps = document.get_element_by_id("stat-fps");

    // Get performance API for FPS calculation
    let performance: Performance = window.performance().unwrap();

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let state_clone = state.clone();
    let document_clone = document.clone();
    let mut frame_count: u32 = 0;
    let mut last_time = performance.now();
    let mut fps_accumulator = 0.0;
    let mut fps_frame_count = 0;
    let mut stats = SimulationStats::new(initial_pop);
    
    *g.borrow_mut() = Some(Closure::new(move || {
        let mut s = state_clone.borrow_mut();
        frame_count += 1;
        
        // FPS calculation
        let current_time = performance.now();
        let delta = current_time - last_time;
        last_time = current_time;
        fps_accumulator += delta;
        fps_frame_count += 1;

        if frame_count % 60 == 0 {
            s.obstacles = scan_dom_obstacles(&document_clone);
        }
        
        // Update dashboard every 30 frames
        if frame_count % 30 == 0 {
            // Update population
            if let Some(ref el) = stat_pop {
                el.set_text_content(Some(&format!("POP: {}", s.boids.len())));
            }
            
            // Calculate and update max generation
            let max_gen = s.boids.iter().map(|b| b.generation).max().unwrap_or(0);
            if let Some(ref el) = stat_gen {
                el.set_text_content(Some(&format!("GEN: {}", max_gen)));
            }
            
            // Update FPS (average over last period)
            if fps_frame_count > 0 && fps_accumulator > 0.0 {
                let avg_fps = (fps_frame_count as f64 / fps_accumulator) * 1000.0;
                if let Some(ref el) = stat_fps {
                    el.set_text_content(Some(&format!("FPS: {:.0}", avg_fps)));
                }
                fps_accumulator = 0.0;
                fps_frame_count = 0;
            }
            
            // Check for new speed record
            let current_max_speed = s.boids.iter()
                .map(|b| b.genes.max_speed)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
            
            if current_max_speed > stats.max_speed_record + 0.1 {
                stats.max_speed_record = current_max_speed;
                log_event(&document_clone, &format!("NEW SPEED RECORD: {:.2}", current_max_speed), "event-record");
            }
            
            // Check for mass extinction event (population drops below 50%)
            let current_pop = s.boids.len();
            if current_pop < stats.initial_population / 2 && !stats.extinction_triggered {
                stats.extinction_triggered = true;
                log_event(&document_clone, &format!("⚠ MASS EXTINCTION! Pop: {}", current_pop), "event-death");
            } else if current_pop >= stats.initial_population / 2 {
                stats.extinction_triggered = false; // Reset for next potential extinction
            }
        }

        // Update dimensions from canvas (handled by resize listener)
        let canvas_w = ctx.canvas().unwrap().width() as f32;
        let canvas_h = ctx.canvas().unwrap().height() as f32;
        
        // Update world state
        s.width = canvas_w;
        s.height = canvas_h;

        // Background: Dark Grey
        ctx.set_fill_style(&JsValue::from_str("#222222"));
        ctx.fill_rect(0.0, 0.0, canvas_w as f64, canvas_h as f64);

        // 1. Snapshot state to calculate forces
        let boids_snapshot = s.boids.clone();
        let obstacles_snapshot = s.obstacles.clone();
        let mut accelerations = Vec::with_capacity(s.boids.len());
        let vision_radius = 50.0;

        for boid in &s.boids {
            let cohesion = boid.cohesion(&boids_snapshot, vision_radius) * 1.0;
            let alignment = boid.alignment(&boids_snapshot, vision_radius) * 1.0;
            let separation = boid.separation(&boids_snapshot, vision_radius) * 1.5;
            
            // Obstacle Avoidance using the avoid_obstacles method
            let avoidance = boid.avoid_obstacles(&obstacles_snapshot) * 100.0;

            accelerations.push(cohesion + alignment + separation + avoidance);
        }

        // 2. Apply forces and update
        let mut new_babies: Vec<Boid> = Vec::new();
        
        for (i, boid) in s.boids.iter_mut().enumerate() {
            // Add flocking acceleration
            boid.velocity += accelerations[i] * 0.05; // Small factor to prevent explosion
            
            // Limit speed using genome's max_speed
            let max_speed = boid.genes.max_speed;
            if boid.velocity.length() > max_speed {
                boid.velocity = boid.velocity.normalize() * max_speed;
            }

            boid.update(1.0, canvas_w, canvas_h); // using 1.0 as relative time step
            
            // Feeding zone: check distance to obstacles (monoliths)
            for obs in &obstacles_snapshot {
                let distance = boid.position.distance(obs.center);
                if distance < 150.0 {
                    boid.feed(0.5);
                }
            }
            
            // Reproduction: if boid has enough energy, spawn a child
            if let Some(mut child) = boid.reproduce() {
                // Scale child position to canvas coordinates (it inherits parent position)
                child.position.x = child.position.x.clamp(0.0, canvas_w);
                child.position.y = child.position.y.clamp(0.0, canvas_h);
                new_babies.push(child);
            }
        }
        
        // Add new babies to the population
        s.boids.append(&mut new_babies);
        
        // Death: remove boids with no energy
        s.boids.retain(|b| b.energy > 0.0);

        // Draw all boids as triangles
        for boid in s.boids.iter() {
            // Positions are already in canvas coordinates
            let x = boid.position.x;
            let y = boid.position.y;
            
            // Calculate angle from velocity
            let angle = boid.velocity.y.atan2(boid.velocity.x);
            
            // Save context state
            ctx.save();
            
            // Translate to boid position
            ctx.translate(x as f64, y as f64).unwrap();
            
            // Rotate to face velocity direction
            ctx.rotate(angle as f64).unwrap();
            
            // Draw triangle pointing forward
            ctx.begin_path();
            ctx.move_to(BOID_SIZE as f64 * 1.5, 0.0); // Tip of triangle
            ctx.line_to(-BOID_SIZE as f64, -BOID_SIZE as f64); // Bottom left
            ctx.line_to(-BOID_SIZE as f64, BOID_SIZE as f64); // Bottom right
            ctx.close_path();
            
            // Use the HSL color from the boid's genetic traits and health
            let color = boid.get_color_string();
            ctx.set_fill_style(&JsValue::from_str(&color));
            ctx.fill();
            
            // Restore context state
            ctx.restore();
        }

        // Only request next frame if not paused (allows E2E tests to take stable screenshots)
        if !paused {
            web_sys::window()
                .unwrap()
                .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .unwrap();
        }
    }));

    // Start animation loop (render at least once)
    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
