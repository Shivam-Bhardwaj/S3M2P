// AutoCrate - ASTM Standard Shipping Crate Generator
// Rust/WASM port of the original TypeScript application
#![allow(unexpected_cfgs)]

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub mod calculator;
pub mod constants;
pub mod geometry;
pub mod render;
pub mod assembly;
pub mod generator;

pub use constants::LumberSize;
pub use geometry::*;
pub use render::{WebGLRenderer, Canvas2DRenderer, ViewMode, Camera};
pub use assembly::{CrateAssembly, ComponentType};
pub use generator::generate_crate;

/// Product dimensions input (in inches)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductDimensions {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub weight: f32,
}

impl Default for ProductDimensions {
    fn default() -> Self {
        Self {
            length: 120.0,
            width: 120.0,
            height: 120.0,
            weight: 10000.0,
        }
    }
}

/// Clearances around product (in inches)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clearances {
    pub side: f32,
    pub end: f32,
    pub top: f32,
}

/// Crate construction style per ASTM D6039
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrateStyle {
    /// Style A - Open frame crate (cleated frame only, no sheathing)
    /// Heavy-duty, suitable for severe handling
    A,
    /// Style B - Sheathed/covered crate (cleated frame with plywood panels)
    /// Light-duty, suitable for light-to-moderate handling
    B,
}

impl Default for Clearances {
    fn default() -> Self {
        Self {
            side: 2.0,
            end: 2.0,
            top: 3.0,
        }
    }
}

/// Complete crate specification
#[derive(Clone, Debug)]
pub struct CrateSpec {
    pub product: ProductDimensions,
    pub clearances: Clearances,
    pub style: CrateStyle,
    pub skid_count: u8,
    pub skid_size: LumberSize,
    pub floorboard_size: LumberSize,
    pub cleat_size: LumberSize,
}

impl Default for CrateSpec {
    fn default() -> Self {
        Self {
            product: ProductDimensions::default(),
            clearances: Clearances::default(),
            style: CrateStyle::B,
            skid_count: 3,
            skid_size: LumberSize::L4x4,
            floorboard_size: LumberSize::L2x6,
            cleat_size: LumberSize::L1x4,
        }
    }
}

impl Default for CrateStyle {
    fn default() -> Self {
        CrateStyle::B
    }
}

/// Generated crate geometry
#[derive(Clone, Debug)]
pub struct CrateGeometry {
    pub overall_length: f32,
    pub overall_width: f32,
    pub overall_height: f32,
    pub base_height: f32,
    pub skids: Vec<SkidGeometry>,
    pub floorboards: Vec<BoardGeometry>,
    pub panels: PanelSet,
    pub cleats: Vec<CleatGeometry>,
}

use web_sys::{window, HtmlCanvasElement, MouseEvent, WheelEvent};
use wasm_bindgen::closure::Closure;
use std::rc::Rc;
use std::cell::RefCell;

/// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"AutoCrate 3D Visualization Demo".into());

    // Get canvas element
    let document = window()
        .ok_or("No window")?
        .document()
        .ok_or("No document")?;

    let canvas = document
        .get_element_by_id("canvas")
        .ok_or("No canvas element")?
        .dyn_into::<HtmlCanvasElement>()?;

    // Set canvas size
    let width = window().unwrap().inner_width()?.as_f64().unwrap_or(800.0) as u32;
    let height = window().unwrap().inner_height()?.as_f64().unwrap_or(600.0) as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    // Initialize WebGL renderer
    let mut renderer = render::WebGLRenderer::new(canvas.clone())
        .map_err(|e| JsValue::from_str(&e))?;

    renderer.init_shaders()
        .map_err(|e| JsValue::from_str(&e))?;

    web_sys::console::log_1(&"WebGL renderer initialized!".into());

    // Set up camera (looking at a crate)
    {
        let camera = renderer.camera_mut();
        camera.distance = 150.0;
        camera.azimuth = std::f32::consts::PI / 4.0;  // 45 degrees
        camera.elevation = std::f32::consts::PI / 6.0; // 30 degrees
        camera.target = glam::Vec3::new(0.0, 0.0, 20.0);
    }

    // ASTM D6039 Style B Compliant Crate (Sheathed)
    // Dimensions: 48" wide x 60" long x 40" tall

    // BASE ASSEMBLY
    // 3 Skids (4x4 lumber, 60" long, running along Y axis)
    let skid1 = render::Mesh::create_box(
        glam::Vec3::new(-18.0, -30.0, 0.0),
        glam::Vec3::new(-14.5, 30.0, 3.5),
    );
    let skid2 = render::Mesh::create_box(
        glam::Vec3::new(-1.75, -30.0, 0.0),
        glam::Vec3::new(1.75, 30.0, 3.5),
    );
    let skid3 = render::Mesh::create_box(
        glam::Vec3::new(14.5, -30.0, 0.0),
        glam::Vec3::new(18.0, 30.0, 3.5),
    );

    // Floor boards (2x6 lumber, 48" wide, across skids along X axis)
    let mut floor_boards = Vec::new();
    for i in 0..11 {
        let y_pos = -27.5 + i as f32 * 5.5;
        floor_boards.push(render::Mesh::create_box(
            glam::Vec3::new(-24.0, y_pos, 3.5),
            glam::Vec3::new(24.0, y_pos + 1.5, 5.0),
        ));
    }

    // FRAME STRUCTURE (1x4 cleats forming box frame)
    // 4 vertical corner posts (full height 35")
    let corner_fl = render::Mesh::create_box( // Front-left
        glam::Vec3::new(-24.0, -30.0, 5.0),
        glam::Vec3::new(-23.25, -26.5, 40.0),
    );
    let corner_fr = render::Mesh::create_box( // Front-right
        glam::Vec3::new(23.25, -30.0, 5.0),
        glam::Vec3::new(24.0, -26.5, 40.0),
    );
    let corner_bl = render::Mesh::create_box( // Back-left
        glam::Vec3::new(-24.0, 26.5, 5.0),
        glam::Vec3::new(-23.25, 30.0, 40.0),
    );
    let corner_br = render::Mesh::create_box( // Back-right
        glam::Vec3::new(23.25, 26.5, 5.0),
        glam::Vec3::new(24.0, 30.0, 40.0),
    );

    // Intermediate vertical posts (max 24" spacing rule)
    // Front wall: 48" wide needs 1 intermediate post at center
    let mid_front = render::Mesh::create_box(
        glam::Vec3::new(-0.375, -30.0, 5.0),
        glam::Vec3::new(0.375, -26.5, 40.0),
    );
    // Back wall
    let mid_back = render::Mesh::create_box(
        glam::Vec3::new(-0.375, 26.5, 5.0),
        glam::Vec3::new(0.375, 30.0, 40.0),
    );

    // Bottom rail frame (1x4 horizontal around base perimeter)
    let bot_front = render::Mesh::create_box(
        glam::Vec3::new(-24.0, -30.0, 5.0),
        glam::Vec3::new(24.0, -29.25, 5.75),
    );
    let bot_back = render::Mesh::create_box(
        glam::Vec3::new(-24.0, 29.25, 5.0),
        glam::Vec3::new(24.0, 30.0, 5.75),
    );

    // Top rail frame (1x4 horizontal around top perimeter)
    let top_front = render::Mesh::create_box(
        glam::Vec3::new(-24.0, -30.0, 39.25),
        glam::Vec3::new(24.0, -29.25, 40.0),
    );
    let top_back = render::Mesh::create_box(
        glam::Vec3::new(-24.0, 29.25, 39.25),
        glam::Vec3::new(24.0, 30.0, 40.0),
    );
    let top_left = render::Mesh::create_box(
        glam::Vec3::new(-24.0, -30.0, 39.25),
        glam::Vec3::new(-23.25, 30.0, 40.0),
    );
    let top_right = render::Mesh::create_box(
        glam::Vec3::new(23.25, -30.0, 39.25),
        glam::Vec3::new(24.0, 30.0, 40.0),
    );

    // Nail heads (small cylinders approximated as tiny boxes)
    let mut nail_heads = Vec::new();
    // Nails on floor boards (spaced ~6" apart on each board)
    for i in 0..11 {
        let y = -27.5 + i as f32 * 5.5 + 0.75; // On top of floor board
        for x_pos in [-18.0, -16.0, 0.0, 16.0, 18.0] {
            nail_heads.push(render::Mesh::create_box(
                glam::Vec3::new(x_pos - 0.075, y - 0.075, 5.0),
                glam::Vec3::new(x_pos + 0.075, y + 0.075, 5.15),
            ));
        }
    }

    // SHEATHING (Plywood panels on all 4 sides + top)
    let panel_front = render::Mesh::create_box(
        glam::Vec3::new(-24.0, -30.5, 5.0),
        glam::Vec3::new(24.0, -30.0, 40.0),
    );
    let panel_back = render::Mesh::create_box(
        glam::Vec3::new(-24.0, 30.0, 5.0),
        glam::Vec3::new(24.0, 30.5, 40.0),
    );
    let panel_left = render::Mesh::create_box(
        glam::Vec3::new(-24.5, -30.0, 5.0),
        glam::Vec3::new(-24.0, 30.0, 40.0),
    );
    let panel_right = render::Mesh::create_box(
        glam::Vec3::new(24.0, -30.0, 5.0),
        glam::Vec3::new(24.5, 30.0, 40.0),
    );
    let panel_top = render::Mesh::create_box(
        glam::Vec3::new(-24.0, -30.0, 40.0),
        glam::Vec3::new(24.0, 30.0, 40.5),
    );

    // Upload to GPU
    let gl = renderer.gl.clone();

    // Skids
    let skid_bufs_vec = vec![
        render::MeshBuffer::from_mesh(&gl, &skid1).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &skid2).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &skid3).map_err(|e| JsValue::from_str(&e))?,
    ];

    // Floor boards
    let mut floor_bufs = Vec::new();
    for floor in &floor_boards {
        floor_bufs.push(render::MeshBuffer::from_mesh(&gl, floor)
            .map_err(|e| JsValue::from_str(&e))?);
    }

    // Corner posts + intermediate posts
    let corner_bufs = vec![
        render::MeshBuffer::from_mesh(&gl, &corner_fl).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &corner_fr).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &corner_bl).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &corner_br).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &mid_front).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &mid_back).map_err(|e| JsValue::from_str(&e))?,
    ];

    // Rails (top + bottom)
    let rail_bufs = vec![
        render::MeshBuffer::from_mesh(&gl, &top_front).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &top_back).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &top_left).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &top_right).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &bot_front).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &bot_back).map_err(|e| JsValue::from_str(&e))?,
    ];

    // Nail heads
    let mut nail_bufs = Vec::new();
    for nail in &nail_heads {
        nail_bufs.push(render::MeshBuffer::from_mesh(&gl, nail)
            .map_err(|e| JsValue::from_str(&e))?);
    }

    // Panels
    let panel_bufs_vec = vec![
        render::MeshBuffer::from_mesh(&gl, &panel_front).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &panel_back).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &panel_left).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &panel_right).map_err(|e| JsValue::from_str(&e))?,
        render::MeshBuffer::from_mesh(&gl, &panel_top).map_err(|e| JsValue::from_str(&e))?,
    ];

    web_sys::console::log_1(&"Meshes created and uploaded to GPU!".into());

    // Render frame with the complete crate
    renderer.begin_frame();

    // Draw skids (4x4 lumber - darker brown)
    let skid_color = glam::Vec3::new(0.65, 0.45, 0.30);
    for buf in &skid_bufs_vec {
        renderer.draw_mesh(buf, skid_color);
    }

    // Draw floor boards (2x6 lumber - lighter tan)
    let floor_color = glam::Vec3::new(0.85, 0.75, 0.60);
    for floor_buf in &floor_bufs {
        renderer.draw_mesh(floor_buf, floor_color);
    }

    // Draw corner posts and rails (1x4 lumber - medium brown)
    let frame_color = glam::Vec3::new(0.75, 0.60, 0.45);
    for buf in &corner_bufs {
        renderer.draw_mesh(buf, frame_color);
    }
    for buf in &rail_bufs {
        renderer.draw_mesh(buf, frame_color);
    }

    // Draw plywood panels (light wood tone)
    let panel_color = glam::Vec3::new(0.80, 0.70, 0.55);
    for buf in &panel_bufs_vec {
        renderer.draw_mesh(buf, panel_color);
    }

    // Draw nail heads (galvanized steel)
    let nail_color = glam::Vec3::new(0.60, 0.65, 0.70);
    for buf in &nail_bufs {
        renderer.draw_mesh(buf, nail_color);
    }

    renderer.end_frame();

    web_sys::console::log_1(&"ASTM D6039 Style B compliant crate rendered!".into());
    web_sys::console::log_1(&"Components: 3 skids, 11 boards, 6 posts, 6 rails, 5 panels, 55 nails".into());

    // Wrap renderer and buffers in Rc<RefCell<>> for event handlers
    let renderer = Rc::new(RefCell::new(renderer));
    let skid_bufs = Rc::new(skid_bufs_vec);
    let floor_bufs = Rc::new(floor_bufs);
    let corner_bufs = Rc::new(corner_bufs);
    let rail_bufs = Rc::new(rail_bufs);
    let panel_bufs = Rc::new(panel_bufs_vec);
    let nail_bufs = Rc::new(nail_bufs);

    // Mouse down handler
    {
        let renderer = renderer.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let mut r = renderer.borrow_mut();
            r.is_dragging = true;
            r.last_mouse_x = event.client_x() as f32;
            r.last_mouse_y = event.client_y() as f32;
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Mouse move handler (orbit camera)
    {
        let renderer = renderer.clone();
        let skid_bufs = skid_bufs.clone();
        let floor_bufs = floor_bufs.clone();
        let corner_bufs = corner_bufs.clone();
        let rail_bufs = rail_bufs.clone();
        let panel_bufs = panel_bufs.clone();
        let nail_bufs = nail_bufs.clone();

        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let mut r = renderer.borrow_mut();
            if r.is_dragging {
                let dx = event.client_x() as f32 - r.last_mouse_x;
                let dy = event.client_y() as f32 - r.last_mouse_y;

                // Orbit camera
                let sensitivity = 0.005;
                r.camera_mut().orbit(dx * sensitivity, -dy * sensitivity);

                r.last_mouse_x = event.client_x() as f32;
                r.last_mouse_y = event.client_y() as f32;

                // Re-render
                r.begin_frame();

                let skid_color = glam::Vec3::new(0.65, 0.45, 0.30);
                for buf in skid_bufs.iter() {
                    r.draw_mesh(buf, skid_color);
                }

                let floor_color = glam::Vec3::new(0.85, 0.75, 0.60);
                for buf in floor_bufs.iter() {
                    r.draw_mesh(buf, floor_color);
                }

                let frame_color = glam::Vec3::new(0.75, 0.60, 0.45);
                for buf in corner_bufs.iter() {
                    r.draw_mesh(buf, frame_color);
                }
                for buf in rail_bufs.iter() {
                    r.draw_mesh(buf, frame_color);
                }

                let panel_color = glam::Vec3::new(0.80, 0.70, 0.55);
                for buf in panel_bufs.iter() {
                    r.draw_mesh(buf, panel_color);
                }

                let nail_color = glam::Vec3::new(0.60, 0.65, 0.70);
                for buf in nail_bufs.iter() {
                    r.draw_mesh(buf, nail_color);
                }

                r.end_frame();
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Mouse up handler
    {
        let renderer = renderer.clone();
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            renderer.borrow_mut().is_dragging = false;
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Wheel handler (zoom)
    {
        let renderer = renderer.clone();
        let skid_bufs = skid_bufs.clone();
        let floor_bufs = floor_bufs.clone();
        let corner_bufs = corner_bufs.clone();
        let rail_bufs = rail_bufs.clone();
        let panel_bufs = panel_bufs.clone();
        let nail_bufs = nail_bufs.clone();

        let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
            event.prevent_default();
            let mut r = renderer.borrow_mut();

            // Zoom camera
            let delta = event.delta_y() as f32 * 0.1;
            r.camera_mut().zoom(delta);

            // Re-render
            r.begin_frame();

            let skid_color = glam::Vec3::new(0.65, 0.45, 0.30);
            for buf in skid_bufs.iter() {
                r.draw_mesh(buf, skid_color);
            }

            let floor_color = glam::Vec3::new(0.85, 0.75, 0.60);
            for buf in floor_bufs.iter() {
                r.draw_mesh(buf, floor_color);
            }

            let frame_color = glam::Vec3::new(0.75, 0.60, 0.45);
            for buf in corner_bufs.iter() {
                r.draw_mesh(buf, frame_color);
            }
            for buf in rail_bufs.iter() {
                r.draw_mesh(buf, frame_color);
            }

            let panel_color = glam::Vec3::new(0.80, 0.70, 0.55);
            for buf in panel_bufs.iter() {
                r.draw_mesh(buf, panel_color);
            }

            let nail_color = glam::Vec3::new(0.60, 0.65, 0.70);
            for buf in nail_bufs.iter() {
                r.draw_mesh(buf, nail_color);
            }

            r.end_frame();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    web_sys::console::log_1(&"Interactive controls enabled! Drag to orbit, scroll to zoom".into());

    Ok(())
}
