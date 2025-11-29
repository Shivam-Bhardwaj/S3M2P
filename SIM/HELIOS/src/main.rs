// Helios - Heliosphere Visualization
// GPU-free Canvas 2D rendering following too.foo patterns

#![allow(unexpected_cfgs)]

mod render;
mod simulation;

#[cfg(target_arch = "wasm32")]
use simulation::{DragMode, SimulationState};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{
    window, CanvasRenderingContext2d, HtmlButtonElement, HtmlCanvasElement, HtmlInputElement,
    InputEvent, KeyboardEvent, MouseEvent, TouchEvent, WheelEvent,
};

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
        wasm_bindgen_futures::spawn_local(async {
            run();
        });
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
        None => {
            log("No window found");
            return;
        }
    };

    let document = match window.document() {
        Some(d) => d,
        None => {
            log("No document found");
            return;
        }
    };

    log("Helios - Heliosphere Visualization (Canvas 2D)");
    log("Controls: Scroll=zoom, Drag=pan, 1-8=planets, Space=pause, +/-=time");

    // Get canvas
    let canvas = match document.get_element_by_id("helios-canvas") {
        Some(el) => match el.dyn_into::<HtmlCanvasElement>() {
            Ok(c) => c,
            Err(_) => {
                log("Element is not a canvas");
                return;
            }
        },
        None => {
            log("Canvas not found");
            return;
        }
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
            Err(_) => {
                log("Failed to get 2D context");
                return;
            }
        },
        _ => {
            log("Failed to get 2D context");
            return;
        }
    };

    // Initialize simulation state
    let state = Rc::new(RefCell::new(SimulationState::new()));
    state
        .borrow_mut()
        .set_viewport(window_width as f64, window_height as f64);
    state.borrow_mut().view_heliosphere(); // Start with heliosphere view - shows pulsating solar cycle

    // Time tracking
    let start_time = Rc::new(RefCell::new(
        window.performance().map(|p| p.now()).unwrap_or(0.0) / 1000.0,
    ));
    let last_frame_time = Rc::new(RefCell::new(*start_time.borrow()));
    let frame_times = Rc::new(RefCell::new([0.0f64; 60]));
    let frame_idx = Rc::new(RefCell::new(0usize));

    // === INPUT HANDLERS - CAD-like 3D controls ===
    // Left-click drag: Pan
    // Right-click drag OR Middle-click drag: Orbit camera around Sun
    // Double-click: Reset to heliosphere view
    // Scroll: Zoom

    // Double-click - reset view to heliosphere
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            event.prevent_default();
            let mut s = state.borrow_mut();
            s.view_heliosphere();
            s.view.tilt = 0.4; // Reset tilt
            s.view.rotation = 0.0; // Reset rotation
            s.orbit_dirty = true;
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("dblclick", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Mouse down - determine drag mode
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            event.prevent_default();
            let mut s = state.borrow_mut();

            // Determine drag mode based on button
            // Button 0 = left (pan), 1 = middle (orbit), 2 = right (orbit)
            let is_orbit = event.button() == 2 || event.button() == 1;

            s.view.drag_start_x = event.client_x() as f64;
            s.view.drag_start_y = event.client_y() as f64;

            if is_orbit {
                // Orbit mode - rotate camera around Sun
                s.view.drag_mode = DragMode::Orbit;
                s.view.last_tilt = s.view.tilt;
                s.view.last_rotation = s.view.rotation;
            } else {
                // Pan mode - move camera position
                s.view.drag_mode = DragMode::Pan;
                s.view.last_center_x = s.view.center_x;
                s.view.last_center_y = s.view.center_y;
            }
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Prevent context menu on right-click
    {
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            event.prevent_default();
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("contextmenu", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Mouse up - end drag
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().view.drag_mode = DragMode::None;
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Mouse leave - end drag
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().view.drag_mode = DragMode::None;
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Mouse move - handle pan or orbit
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let mut s = state.borrow_mut();
            let dx = event.client_x() as f64 - s.view.drag_start_x;
            let dy = event.client_y() as f64 - s.view.drag_start_y;

            match s.view.drag_mode {
                DragMode::Pan => {
                    // Pan: move the view center
                    s.view.center_x = s.view.last_center_x - dx * s.view.zoom;
                    s.view.center_y = s.view.last_center_y - dy * s.view.zoom;
                }
                DragMode::Orbit => {
                    // Orbit: rotate camera around Sun (CAD-like)
                    // Horizontal drag = azimuth rotation
                    // Vertical drag = tilt (elevation)
                    let rotation_sensitivity = 0.005;
                    let tilt_sensitivity = 0.005;

                    let new_rotation = s.view.last_rotation + dx * rotation_sensitivity;
                    let new_tilt = (s.view.last_tilt + dy * tilt_sensitivity)
                        .clamp(0.0, std::f64::consts::PI * 0.45); // 0 to ~80 degrees

                    s.view.rotation = new_rotation;
                    s.view.tilt = new_tilt;
                    s.orbit_dirty = true;
                }
                DragMode::None => {}
            }
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
            .unwrap();
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
        canvas
            .add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Touch start (1 finger = pan, 2 fingers = pinch zoom OR orbit)
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            event.prevent_default();
            let touches = event.touches();
            let mut s = state.borrow_mut();

            if touches.length() == 2 {
                // Two fingers: pinch-to-zoom
                if let (Some(t0), Some(t1)) = (touches.get(0), touches.get(1)) {
                    let dx = t1.client_x() as f64 - t0.client_x() as f64;
                    let dy = t1.client_y() as f64 - t0.client_y() as f64;
                    s.view.pinching = true;
                    s.view.drag_mode = DragMode::None;
                    s.view.pinch_start_dist = (dx * dx + dy * dy).sqrt();
                    s.view.pinch_start_zoom = s.view.zoom;
                    s.view.pinch_center_x = (t0.client_x() + t1.client_x()) as f64 / 2.0;
                    s.view.pinch_center_y = (t0.client_y() + t1.client_y()) as f64 / 2.0;
                    // Also save rotation state for two-finger rotate
                    s.view.last_tilt = s.view.tilt;
                    s.view.last_rotation = s.view.rotation;
                }
            } else if touches.length() == 1 {
                // One finger: pan
                if let Some(touch) = touches.get(0) {
                    s.view.drag_mode = DragMode::Pan;
                    s.view.pinching = false;
                    s.view.drag_start_x = touch.client_x() as f64;
                    s.view.drag_start_y = touch.client_y() as f64;
                    s.view.last_center_x = s.view.center_x;
                    s.view.last_center_y = s.view.center_y;
                }
            }
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Touch end
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: TouchEvent| {
            let mut s = state.borrow_mut();
            s.view.drag_mode = DragMode::None;
            s.view.pinching = false;
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Touch move (pan, pinch-zoom, or two-finger rotate)
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            event.prevent_default();
            let touches = event.touches();
            let mut s = state.borrow_mut();

            if s.view.pinching && touches.length() == 2 {
                // Two-finger gesture
                if let (Some(t0), Some(t1)) = (touches.get(0), touches.get(1)) {
                    let dx = t1.client_x() as f64 - t0.client_x() as f64;
                    let dy = t1.client_y() as f64 - t0.client_y() as f64;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if s.view.pinch_start_dist > 10.0 {
                        // Pinch zoom
                        let scale = s.view.pinch_start_dist / dist;
                        let new_zoom = (s.view.pinch_start_zoom * scale).max(0.0001).min(10.0);

                        let (au_x, au_y) = s
                            .view
                            .screen_to_au(s.view.pinch_center_x, s.view.pinch_center_y);
                        s.view.zoom = new_zoom;
                        let (new_au_x, new_au_y) = s
                            .view
                            .screen_to_au(s.view.pinch_center_x, s.view.pinch_center_y);
                        s.view.center_x += au_x - new_au_x;
                        s.view.center_y += au_y - new_au_y;
                    }

                    // Also track center movement for orbit
                    let new_center_x = (t0.client_x() + t1.client_x()) as f64 / 2.0;
                    let new_center_y = (t0.client_y() + t1.client_y()) as f64 / 2.0;
                    let center_dx = new_center_x - s.view.pinch_center_x;
                    let center_dy = new_center_y - s.view.pinch_center_y;

                    // Two-finger drag rotates view
                    let rotation_sensitivity = 0.003;
                    let tilt_sensitivity = 0.003;
                    s.view.rotation = s.view.last_rotation + center_dx * rotation_sensitivity;
                    s.view.tilt = (s.view.last_tilt + center_dy * tilt_sensitivity)
                        .clamp(0.0, std::f64::consts::PI * 0.45);
                    s.orbit_dirty = true;
                }
            } else if s.view.is_panning() && touches.length() == 1 {
                // Single finger pan
                if let Some(touch) = touches.get(0) {
                    let dx = touch.client_x() as f64 - s.view.drag_start_x;
                    let dy = touch.client_y() as f64 - s.view.drag_start_y;
                    s.view.center_x = s.view.last_center_x - dx * s.view.zoom;
                    s.view.center_y = s.view.last_center_y - dy * s.view.zoom;
                }
            }
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())
            .unwrap();
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
                "+" | "=" => {
                    let ts = s.time_scale * 2.0;
                    s.set_time_scale(ts);
                }
                "-" | "_" => {
                    let ts = s.time_scale / 2.0;
                    s.set_time_scale(ts);
                }
                "ArrowLeft" => s.julian_date -= 30.0, // Month back
                "ArrowRight" => s.julian_date += 30.0, // Month forward
                "ArrowUp" => s.julian_date += 365.25, // Year forward
                "ArrowDown" => s.julian_date -= 365.25, // Year back
                "Home" => {
                    s.view_inner_system();
                    s.julian_date = simulation::J2000_EPOCH + 8766.0; // 2024
                    s.time_scale = 1.0;
                }
                // 3D view controls
                "t" | "T" => {
                    // Tilt up
                    let new_tilt = s.view.tilt + 0.15;
                    s.view.set_tilt(new_tilt);
                    s.mark_orbits_dirty();
                }
                "g" | "G" => {
                    // Tilt down
                    let new_tilt = s.view.tilt - 0.15;
                    s.view.set_tilt(new_tilt);
                    s.mark_orbits_dirty();
                }
                "r" | "R" => {
                    // Reset 3D view
                    s.view.tilt = 0.5;
                    s.view.rotation = 0.0;
                    s.mark_orbits_dirty();
                }
                _ => {}
            }
        }) as Box<dyn FnMut(_)>);
        document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // === UI CONTROLS ===

    // Get UI elements
    let time_slider = document
        .get_element_by_id("time-slider")
        .and_then(|el| el.dyn_into::<HtmlInputElement>().ok());
    let date_display = document.get_element_by_id("date-display");
    let speed_display = document.get_element_by_id("speed-display");
    let cycle_display = document.get_element_by_id("cycle-display");
    let play_pause_btn = document.get_element_by_id("play-pause");
    let solar_icon = document.get_element_by_id("solar-icon");

    // Time slider handler
    if let Some(slider) = time_slider.clone() {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: InputEvent| {
            let mut s = state.borrow_mut();
            let days_offset: f64 = slider.value().parse().unwrap_or(0.0);
            s.julian_date = simulation::J2000_EPOCH + 8766.0 + days_offset;
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("time-slider") {
            el.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Play/Pause button (large, central)
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().toggle_pause();
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("play-pause") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Slower button
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            let mut s = state.borrow_mut();
            let ts = s.time_scale / 2.0;
            s.set_time_scale(ts);
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("btn-slower") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Faster button
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            let mut s = state.borrow_mut();
            let ts = s.time_scale * 2.0;
            s.set_time_scale(ts);
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("btn-faster") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // === PLANET NAVIGATION ===
    // Clickable planet icons for quick navigation

    // Sun
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().focus_on_sun();
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("nav-sun") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Planet navigation helper macro - planets 0-7
    let planet_ids = [
        "nav-mercury",
        "nav-venus",
        "nav-earth",
        "nav-mars",
        "nav-jupiter",
        "nav-saturn",
        "nav-uranus",
        "nav-neptune",
    ];

    for (idx, id) in planet_ids.iter().enumerate() {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().focus_on_planet(idx);
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id(id) {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Heliosphere view
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().view_heliosphere();
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("nav-heliosphere") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // === VIEW PRESETS ===

    // Inner solar system view
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().view_inner_system();
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("view-inner") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Outer solar system view
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().view_outer_system();
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("view-outer") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Zoom in button
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().zoom_by(0.7);
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("view-zoom-in") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Zoom out button
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            state.borrow_mut().zoom_by(1.4);
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("view-zoom-out") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // === 3D VIEW CONTROLS ===

    // Tilt up (towards edge-on view)
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            let mut s = state.borrow_mut();
            let new_tilt = s.view.tilt + 0.15;
            s.view.set_tilt(new_tilt);
            s.mark_orbits_dirty();
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("view-tilt-up") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Tilt down (towards top-down view)
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            let mut s = state.borrow_mut();
            let new_tilt = s.view.tilt - 0.15;
            s.view.set_tilt(new_tilt);
            s.mark_orbits_dirty();
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("view-tilt-down") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Reset 3D view
    {
        let state = state.clone();
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            let mut s = state.borrow_mut();
            s.view.tilt = 0.5; // Default ~30 degrees
            s.view.rotation = 0.0;
            s.mark_orbits_dirty();
        }) as Box<dyn FnMut(_)>);
        if let Some(el) = document.get_element_by_id("view-3d-reset") {
            el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        closure.forget();
    }

    // Wrap UI elements for animation loop
    let time_slider = Rc::new(time_slider);
    let date_display = Rc::new(date_display);
    let speed_display = Rc::new(speed_display);
    let cycle_display = Rc::new(cycle_display);
    let play_pause_btn = Rc::new(play_pause_btn);
    let solar_icon = Rc::new(solar_icon);

    // === ANIMATION LOOP ===

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let ctx = Rc::new(ctx);
    let canvas = Rc::new(canvas);

    // Clone UI elements for animation loop
    let time_slider_loop = time_slider.clone();
    let date_display_loop = date_display.clone();
    let speed_display_loop = speed_display.clone();
    let cycle_display_loop = cycle_display.clone();
    let play_pause_loop = play_pause_btn.clone();
    let solar_icon_loop = solar_icon.clone();

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

        // Update UI elements (every 10 frames to reduce DOM updates)
        if s.frame_count % 10 == 0 {
            // Update time slider position
            if let Some(ref slider) = *time_slider_loop {
                let days_offset = s.julian_date - (simulation::J2000_EPOCH + 8766.0);
                slider.set_value(&format!("{:.0}", days_offset.clamp(-36525.0, 36525.0)));
            }

            // Update date display
            if let Some(ref el) = *date_display_loop {
                let year = s.current_year();
                el.set_text_content(Some(&format!("{:.1}", year)));
            }

            // Update speed display with human-readable units
            if let Some(ref el) = *speed_display_loop {
                el.set_text_content(Some(&s.time_scale_str()));
            }

            // Update cycle display
            if let Some(ref el) = *cycle_display_loop {
                el.set_text_content(Some(s.solar_cycle_name()));
            }

            // Update play/pause button appearance
            if let Some(ref el) = *play_pause_loop {
                if s.paused {
                    el.set_inner_html("&#9654;"); // Play icon
                    el.class_list().add_1("paused").ok();
                } else {
                    el.set_inner_html("&#9208;"); // Pause icon
                    el.class_list().remove_1("paused").ok();
                }
            }

            // Update solar icon based on activity
            if let Some(ref el) = *solar_icon_loop {
                let activity = (s.solar_cycle_phase * 2.0 * std::f64::consts::PI).sin() * 0.5 + 0.5;
                if activity > 0.5 {
                    el.class_list().add_1("active").ok();
                } else {
                    el.class_list().remove_1("active").ok();
                }
            }
        }

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
