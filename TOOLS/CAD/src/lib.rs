//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | TOOLS/CAD/src/lib.rs
//! PURPOSE: CAD solid modeler WASM application with 3D wireframe visualization
//! MODIFIED: 2025-12-09
//! LAYER: TOOLS → CAD
//! ═══════════════════════════════════════════════════════════════════════════════

#![allow(unexpected_cfgs)]
use std::cell::RefCell;
use std::f32::consts::PI;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, Document, Element, HtmlCanvasElement, HtmlElement, HtmlInputElement,
    HtmlSelectElement, MouseEvent, WheelEvent,
};

use cad_engine::{
    is_manifold, make_box, make_cone, make_cylinder, make_sphere, surface_area, volume, Point3,
    Solid,
};

// Global state for the current solid and view
thread_local! {
    static STATE: RefCell<AppState> = RefCell::new(AppState::default());
}

struct AppState {
    solid: Option<Solid>,
    rotation_x: f32,
    rotation_y: f32,
    zoom: f32,
    dragging: bool,
    last_mouse_x: i32,
    last_mouse_y: i32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            solid: None,
            rotation_x: 0.5,  // ~30 degrees
            rotation_y: 0.75, // ~45 degrees
            zoom: 2.0,
            dragging: false,
            last_mouse_x: 0,
            last_mouse_y: 0,
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"CAD Modeler initialized".into());

    if let Err(e) = init_ui() {
        web_sys::console::error_1(&format!("Failed to initialize UI: {:?}", e).into());
    }

    Ok(())
}

fn init_ui() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Set up create button
    if let Some(btn) = document.get_element_by_id("create-btn") {
        let btn: HtmlElement = btn.dyn_into()?;
        let closure = Closure::wrap(Box::new(move || {
            if let Err(e) = create_solid() {
                web_sys::console::error_1(&format!("Create failed: {:?}", e).into());
            }
        }) as Box<dyn FnMut()>);
        btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Set up viewport mouse events
    setup_viewport_events(&document)?;

    // Export view function to JS
    let set_view_closure = Closure::wrap(Box::new(|view: String| {
        set_view(&view);
    }) as Box<dyn Fn(String)>);
    js_sys::Reflect::set(
        &window,
        &JsValue::from_str("setViewAngle"),
        set_view_closure.as_ref(),
    )?;
    set_view_closure.forget();

    // Create initial solid
    create_solid()?;

    Ok(())
}

fn setup_viewport_events(document: &Document) -> Result<(), JsValue> {
    let canvas = document
        .get_element_by_id("viewport-canvas")
        .ok_or("Canvas not found")?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    // Mouse down
    {
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.dragging = true;
                state.last_mouse_x = event.client_x();
                state.last_mouse_y = event.client_y();
            });
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas.set_onmousedown(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Mouse up
    {
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            STATE.with(|state| {
                state.borrow_mut().dragging = false;
            });
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas.set_onmouseup(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Mouse leave
    {
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            STATE.with(|state| {
                state.borrow_mut().dragging = false;
            });
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas.set_onmouseleave(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Mouse move
    {
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                if state.dragging {
                    let dx = event.client_x() - state.last_mouse_x;
                    let dy = event.client_y() - state.last_mouse_y;
                    state.rotation_y += dx as f32 * 0.01;
                    state.rotation_x += dy as f32 * 0.01;
                    state.last_mouse_x = event.client_x();
                    state.last_mouse_y = event.client_y();
                    drop(state);
                    let _ = render();
                }
            });
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas.set_onmousemove(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Mouse wheel for zoom
    {
        let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
            event.prevent_default();
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                let delta = event.delta_y() as f32 * 0.001;
                state.zoom = (state.zoom - delta).clamp(0.5, 10.0);
                drop(state);
                let _ = render();
            });
        }) as Box<dyn FnMut(WheelEvent)>);
        canvas.set_onwheel(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    Ok(())
}

fn set_view(view: &str) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match view {
            "front" => {
                state.rotation_x = 0.0;
                state.rotation_y = 0.0;
            }
            "top" => {
                state.rotation_x = PI / 2.0;
                state.rotation_y = 0.0;
            }
            "right" => {
                state.rotation_x = 0.0;
                state.rotation_y = PI / 2.0;
            }
            _ => {
                // "iso" or any other view defaults to isometric
                state.rotation_x = 0.5;
                state.rotation_y = 0.75;
            }
        }
    });
    let _ = render();
}

fn create_solid() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Get primitive type
    let prim_select = document
        .get_element_by_id("primitive-type")
        .ok_or("Select not found")?;
    let prim_select: HtmlSelectElement = prim_select.dyn_into()?;
    let prim_type = prim_select.value();

    let solid = match prim_type.as_str() {
        "box" => {
            let width = get_input_value(&document, "width")? as f32;
            let depth = get_input_value(&document, "depth")? as f32;
            let height = get_input_value(&document, "height")? as f32;
            make_box(width, depth, height)
        }
        "cylinder" => {
            let radius = get_input_value(&document, "cyl-radius")? as f32;
            let height = get_input_value(&document, "cyl-height")? as f32;
            let segments = get_input_value(&document, "cyl-segments")? as u32;
            make_cylinder(radius, height, segments)
        }
        "sphere" => {
            let radius = get_input_value(&document, "sph-radius")? as f32;
            let u_seg = get_input_value(&document, "sph-u-seg")? as u32;
            let v_seg = get_input_value(&document, "sph-v-seg")? as u32;
            make_sphere(radius, u_seg, v_seg)
        }
        "cone" => {
            let radius = get_input_value(&document, "cone-radius")? as f32;
            let height = get_input_value(&document, "cone-height")? as f32;
            let segments = get_input_value(&document, "cone-segments")? as u32;
            make_cone(radius, height, segments)
        }
        _ => make_box(100.0, 50.0, 25.0),
    };

    // Update state
    STATE.with(|state| {
        state.borrow_mut().solid = Some(solid);
    });

    // Update display
    display_properties(&document)?;
    render()?;

    Ok(())
}

fn display_properties(document: &Document) -> Result<(), JsValue> {
    STATE.with(|state| {
        let state = state.borrow();
        if let Some(ref solid) = state.solid {
            set_text(
                document,
                "result-vertices",
                &solid.vertices.len().to_string(),
            )
            .ok();
            set_text(document, "result-edges", &solid.edges.len().to_string()).ok();
            set_text(document, "result-faces", &solid.faces.len().to_string()).ok();

            let vol = volume(solid);
            let vol_text = if vol >= 1e6 {
                format!("{:.2} cm³", vol / 1e3)
            } else {
                format!("{:.1} mm³", vol)
            };
            set_text(document, "result-volume", &vol_text).ok();

            let area = surface_area(solid);
            let area_text = if area >= 1e4 {
                format!("{:.2} cm²", area / 100.0)
            } else {
                format!("{:.1} mm²", area)
            };
            set_text(document, "result-area", &area_text).ok();

            let manifold = if is_manifold(solid) { "Yes" } else { "No" };
            set_text(document, "result-manifold", manifold).ok();
        }
    });
    Ok(())
}

fn render() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let canvas = document
        .get_element_by_id("viewport-canvas")
        .ok_or("Canvas not found")?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    let dpr = window.device_pixel_ratio();

    let canvas_element: Element = canvas.clone().into();
    let rect = canvas_element.get_bounding_client_rect();
    let css_width = rect.width();
    let css_height = rect.height();

    let target_width = (css_width * dpr) as u32;
    let target_height = (css_height * dpr) as u32;

    if (canvas.width() as i32 - target_width as i32).abs() > 2
        || (canvas.height() as i32 - target_height as i32).abs() > 2
    {
        canvas.set_width(target_width);
        canvas.set_height(target_height);
    }

    let ctx = canvas
        .get_context("2d")?
        .ok_or("No 2D context")?
        .dyn_into::<CanvasRenderingContext2d>()?;

    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    ctx.scale(dpr, dpr)?;

    // Clear
    ctx.set_fill_style(&JsValue::from_str("#0a0a12"));
    ctx.fill_rect(0.0, 0.0, css_width, css_height);

    STATE.with(|state| {
        let state = state.borrow();
        if let Some(ref solid) = state.solid {
            draw_wireframe(
                &ctx,
                solid,
                css_width,
                css_height,
                state.rotation_x,
                state.rotation_y,
                state.zoom,
            )
            .ok();
        }
    });

    Ok(())
}

fn draw_wireframe(
    ctx: &CanvasRenderingContext2d,
    solid: &Solid,
    width: f64,
    height: f64,
    rot_x: f32,
    rot_y: f32,
    zoom: f32,
) -> Result<(), JsValue> {
    let cx = width / 2.0;
    let cy = height / 2.0;
    let scale = (zoom as f64) * (height.min(width) / 300.0);

    // Calculate sin/cos for rotation
    let (sin_x, cos_x) = rot_x.sin_cos();
    let (sin_y, cos_y) = rot_y.sin_cos();

    // Project 3D point to 2D
    let project = |p: &Point3| -> (f64, f64) {
        // Center the model
        let x = p.x;
        let y = p.y;
        let z = p.z;

        // Rotate around Y axis
        let x1 = x * cos_y - z * sin_y;
        let z1 = x * sin_y + z * cos_y;

        // Rotate around X axis
        let y1 = y * cos_x - z1 * sin_x;
        let _z2 = y * sin_x + z1 * cos_x;

        // Simple orthographic projection
        let px = cx + (x1 as f64) * scale;
        let py = cy - (y1 as f64) * scale;

        (px, py)
    };

    // Draw edges
    ctx.set_stroke_style(&JsValue::from_str("#ff6b35"));
    ctx.set_line_width(1.5);

    for edge in &solid.edges {
        if let (Some(start_vertex), Some(end_vertex)) =
            (solid.vertex(edge.start), solid.vertex(edge.end))
        {
            let (x1, y1) = project(&start_vertex.point);
            let (x2, y2) = project(&end_vertex.point);

            ctx.begin_path();
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
            ctx.stroke();
        }
    }

    // Draw vertices
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    for vertex in &solid.vertices {
        let (px, py) = project(&vertex.point);
        ctx.begin_path();
        ctx.arc(px, py, 2.0, 0.0, 2.0 * std::f64::consts::PI)?;
        ctx.fill();
    }

    // Draw axis indicator
    draw_axis_indicator(ctx, width, height, sin_x, cos_x, sin_y, cos_y)?;

    Ok(())
}

fn draw_axis_indicator(
    ctx: &CanvasRenderingContext2d,
    _width: f64,
    height: f64,
    sin_x: f32,
    cos_x: f32,
    sin_y: f32,
    cos_y: f32,
) -> Result<(), JsValue> {
    let origin_x = 50.0;
    let origin_y = height - 50.0;
    let axis_len = 30.0;

    // Project axis vectors
    let project_axis = |ax: f32, ay: f32, az: f32| -> (f64, f64) {
        let x1 = ax * cos_y - az * sin_y;
        let y1 = ay * cos_x - (ax * sin_y + az * cos_y) * sin_x;
        (x1 as f64 * axis_len, -y1 as f64 * axis_len)
    };

    // X axis (red)
    let (dx, dy) = project_axis(1.0, 0.0, 0.0);
    ctx.set_stroke_style(&JsValue::from_str("#ff4444"));
    ctx.set_line_width(2.0);
    ctx.begin_path();
    ctx.move_to(origin_x, origin_y);
    ctx.line_to(origin_x + dx, origin_y + dy);
    ctx.stroke();
    ctx.set_fill_style(&JsValue::from_str("#ff4444"));
    ctx.set_font("10px Monaco");
    ctx.fill_text("X", origin_x + dx + 5.0, origin_y + dy)?;

    // Y axis (green)
    let (dx, dy) = project_axis(0.0, 1.0, 0.0);
    ctx.set_stroke_style(&JsValue::from_str("#44ff44"));
    ctx.begin_path();
    ctx.move_to(origin_x, origin_y);
    ctx.line_to(origin_x + dx, origin_y + dy);
    ctx.stroke();
    ctx.set_fill_style(&JsValue::from_str("#44ff44"));
    ctx.fill_text("Y", origin_x + dx + 5.0, origin_y + dy)?;

    // Z axis (blue)
    let (dx, dy) = project_axis(0.0, 0.0, 1.0);
    ctx.set_stroke_style(&JsValue::from_str("#4444ff"));
    ctx.begin_path();
    ctx.move_to(origin_x, origin_y);
    ctx.line_to(origin_x + dx, origin_y + dy);
    ctx.stroke();
    ctx.set_fill_style(&JsValue::from_str("#4444ff"));
    ctx.fill_text("Z", origin_x + dx + 5.0, origin_y + dy)?;

    Ok(())
}

fn get_input_value(document: &Document, id: &str) -> Result<f64, JsValue> {
    let input = document
        .get_element_by_id(id)
        .ok_or(format!("Input {} not found", id))?;
    let input: HtmlInputElement = input.dyn_into()?;
    input
        .value()
        .parse::<f64>()
        .map_err(|_| JsValue::from_str("Invalid number"))
}

fn set_text(document: &Document, id: &str, text: &str) -> Result<(), JsValue> {
    if let Some(elem) = document.get_element_by_id(id) {
        let elem: HtmlElement = elem.dyn_into()?;
        elem.set_inner_text(text);
    }
    Ok(())
}
