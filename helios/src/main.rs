// Helios - Heliosphere Visualization
// GPU-free Canvas 2D rendering following too.foo patterns

#![allow(unexpected_cfgs)]

mod simulation;
mod render;

#[cfg(target_arch = "wasm32")]
use simulation::SimulationState;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{
    window, HtmlCanvasElement, CanvasRenderingContext2d,
    KeyboardEvent, MouseEvent, WheelEvent, TouchEvent,
};
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        wasm_bindgen_futures::spawn_local(async { run(); });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("Helios is a WASM-only crate. Build with: trunk serve");
    }
}

#[cfg(target_arch = "wasm32")]
fn run() {
    let window = match window() {
        Some(w) => w,
        None => { log("No window found"); return; }
    };

    let document = match window.document() {
        Some(d) => d,
        None => { log("No document found"); return; }
    };

    log("Helios - Heliosphere Visualization (Canvas 2D)");
    log("Controls: Scroll=zoom, Drag=pan, 1-8=planets, Space=pause, +/-=time");

    // Get canvas
    let canvas = match document.get_element_by_id("helios-canvas") {
        Some(el) => match el.dyn_into::<HtmlCanvasElement>() {
            Ok(c) => c,
            Err(_) => { log("Element is not a canvas"); return; }
        },
        None => { log("Canvas not found"); return; }
    };

    // Set canvas size
    let window_width = window.inner_width().unwrap().as_f64().unwrap() as u32;
    let window_height = window.inner_height().unwrap().as_f64().unwrap() as u32;
    canvas.set_width(window_width);
    canvas.set_height(window_height);

    log(&format!("Canvas: {}x{}", window_width, window_height));

    // Get 2D context
    let ctx = match canvas.get_context("2d") {
        Ok(Some(ctx)) => match ctx.dyn_into::<CanvasRenderingContext2d>() {
            Ok(c) => c,
            Err(_) => { log("Failed to get 2D context"); return; }
        },
        _ => { log("Failed to get 2D context"); return; }
    };

    // Initialize simulation state
    let state = Rc::new(RefCell::new(SimulationState::new()));
    state.borrow_mut().set_viewport(window_width as f64, window_height as f64);
    state.borrow_mut().view_inner_system(); // Start with inner solar system view

    // Time tracking
    let start_time = Rc::new(RefCell::new(
        window.performance().map(|p| p.now()).unwrap_or(0.0) / 1000.0
    ));
    let last_frame_time = Rc::new(RefCell::new(*start_time.borrow()));
    let frame_times = Rc::new(RefCell::new([0.0f64; 60]));
    let frame_idx = Rc::new(RefCell::new(0usize));

    // === INPUT HANDLERS ===

    // Mouse down
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let mut s = state.borrow_mut();
            s.view.dragging = true;
            s.view.drag_start_x = event.client_x() as f64;
            s.view.drag_start_y = event.client_y() as f64;
            s.view.last_center_x = s.view.center_x;
            s.view.last_center_y = s.view.center_y;
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Mouse up
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().view.dragging = false;
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Mouse move
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let mut s = state.borrow_mut();
            if s.view.dragging {
                let dx = event.client_x() as f64 - s.view.drag_start_x;
                let dy = event.client_y() as f64 - s.view.drag_start_y;
                s.view.center_x = s.view.last_center_x - dx * s.view.zoom;
                s.view.center_y = s.view.last_center_y - dy * s.view.zoom;
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Mouse wheel (zoom)
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
            event.prevent_default();
            let mut s = state.borrow_mut();

            // Zoom towards mouse position
            let mouse_x = event.client_x() as f64;
            let mouse_y = event.client_y() as f64;
            let (au_x, au_y) = s.view.screen_to_au(mouse_x, mouse_y);

            let factor = if event.delta_y() > 0.0 { 1.15 } else { 0.87 };
            s.zoom_by(factor);

            // Adjust center to zoom towards mouse
            let (new_au_x, new_au_y) = s.view.screen_to_au(mouse_x, mouse_y);
            s.view.center_x += au_x - new_au_x;
            s.view.center_y += au_y - new_au_y;
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Touch start
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            event.prevent_default();
            if let Some(touch) = event.touches().get(0) {
                let mut s = state.borrow_mut();
                s.view.dragging = true;
                s.view.drag_start_x = touch.client_x() as f64;
                s.view.drag_start_y = touch.client_y() as f64;
                s.view.last_center_x = s.view.center_x;
                s.view.last_center_y = s.view.center_y;
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Touch end
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: TouchEvent| {
            state.borrow_mut().view.dragging = false;
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Touch move
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            event.prevent_default();
            if let Some(touch) = event.touches().get(0) {
                let mut s = state.borrow_mut();
                if s.view.dragging {
                    let dx = touch.client_x() as f64 - s.view.drag_start_x;
                    let dy = touch.client_y() as f64 - s.view.drag_start_y;
                    s.view.center_x = s.view.last_center_x - dx * s.view.zoom;
                    s.view.center_y = s.view.last_center_y - dy * s.view.zoom;
                }
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Keyboard
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let mut s = state.borrow_mut();
            match event.key().as_str() {
                " " => s.toggle_pause(),
                "1" => s.focus_on_planet(0), // Mercury
                "2" => s.focus_on_planet(1), // Venus
                "3" => s.focus_on_planet(2), // Earth
                "4" => s.focus_on_planet(3), // Mars
                "5" => s.focus_on_planet(4), // Jupiter
                "6" => s.focus_on_planet(5), // Saturn
                "7" => s.focus_on_planet(6), // Uranus
                "8" => s.focus_on_planet(7), // Neptune
                "0" | "s" | "S" => s.focus_on_sun(),
                "i" | "I" => s.view_inner_system(),
                "o" | "O" => s.view_outer_system(),
                "h" | "H" => s.view_heliosphere(),
                "+" | "=" => { let ts = s.time_scale * 2.0; s.set_time_scale(ts); }
                "-" | "_" => { let ts = s.time_scale / 2.0; s.set_time_scale(ts); }
                "ArrowLeft" => s.julian_date -= 30.0, // Month back
                "ArrowRight" => s.julian_date += 30.0, // Month forward
                "ArrowUp" => s.julian_date += 365.25, // Year forward
                "ArrowDown" => s.julian_date -= 365.25, // Year back
                "Home" => {
                    s.view_inner_system();
                    s.julian_date = simulation::J2000_EPOCH + 8766.0; // 2024
                    s.time_scale = 1.0;
                }
                _ => {}
            }
        }) as Box<dyn FnMut(_)>);
        document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // === ANIMATION LOOP ===

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let ctx = Rc::new(ctx);
    let canvas = Rc::new(canvas);

    let window_clone = window.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        // Time
        let now = window_clone.performance().map(|p| p.now()).unwrap_or(0.0) / 1000.0;
        let time = now - *start_time.borrow();
        let dt = (now - *last_frame_time.borrow()).min(0.1); // Cap dt to avoid spiral
        *last_frame_time.borrow_mut() = now;

        // FPS calculation (rolling average)
        {
            let mut times = frame_times.borrow_mut();
            let mut idx = frame_idx.borrow_mut();
            times[*idx] = dt;
            *idx = (*idx + 1) % 60;
        }

        // Update FPS every 30 frames
        let mut s = state.borrow_mut();
        if s.frame_count % 30 == 0 {
            let times = frame_times.borrow();
            let avg_dt: f64 = times.iter().sum::<f64>() / 60.0;
            s.fps = if avg_dt > 0.0 { 1.0 / avg_dt } else { 60.0 };
        }

        // Handle resize
        let current_width = canvas.client_width() as u32;
        let current_height = canvas.client_height() as u32;
        if current_width != canvas.width() || current_height != canvas.height() {
            if current_width > 0 && current_height > 0 {
                canvas.set_width(current_width);
                canvas.set_height(current_height);
                s.set_viewport(current_width as f64, current_height as f64);
            }
        }

        // Update simulation
        s.update(dt);

        // Render
        render::render(&ctx, &s, time);

        drop(s); // Release borrow before next frame

        // Request next frame
        window_clone
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("requestAnimationFrame failed");
    }));

    // Start animation
    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .expect("requestAnimationFrame failed");

    log("Animation loop started");
}
