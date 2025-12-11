//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | TOOLS/AUTOCRATE/src/lib.rs
//! PURPOSE: ASTM standard shipping crate generator WASM application entry point
//! MODIFIED: 2025-12-09
//! LAYER: TOOLS → AUTOCRATE
//! ═══════════════════════════════════════════════════════════════════════════════

// AutoCrate - ASTM Standard Shipping Crate Generator
// Rust/WASM port of the original TypeScript application
#![allow(unexpected_cfgs)]

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Blob, CanvasRenderingContext2d, Document, Element, HtmlAnchorElement, HtmlCanvasElement,
    HtmlElement, HtmlInputElement, HtmlSelectElement, MouseEvent, Url, WheelEvent,
};

pub use autocrate_engine::*;

thread_local! {
    static STATE: RefCell<AppState> = RefCell::new(AppState::default());
}

#[derive(Default)]
struct AppState {
    spec: CrateSpec,
    design: Option<CrateDesign>,
    rotation_x: f32,
    rotation_y: f32,
    zoom: f32,
    dragging: bool,
    last_mouse_x: i32,
    last_mouse_y: i32,
    selected_part_id: Option<String>,
}

/// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"AutoCrate initialized (viewer + outputs)".into());

    init_ui()?;
    Ok(())
}

fn init_ui() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    setup_viewport_events(&document)?;

    // Export setViewAngle(view) to JS
    let set_view_closure = Closure::wrap(Box::new(|view: String| {
        set_view(&view);
    }) as Box<dyn Fn(String)>);
    js_sys::Reflect::set(
        &window,
        &JsValue::from_str("setViewAngle"),
        set_view_closure.as_ref(),
    )?;
    set_view_closure.forget();

    // Generate/update
    if let Some(btn) = document.get_element_by_id("generate-btn") {
        let btn: HtmlElement = btn.dyn_into()?;
        let closure = Closure::wrap(Box::new(move || {
            if let Err(e) = generate_and_render() {
                web_sys::console::error_1(&format!("Generate failed: {:?}", e).into());
            }
        }) as Box<dyn FnMut()>);
        btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Downloads
    hook_download_button(&document, "download-step", DownloadKind::Step)?;
    hook_download_button(&document, "download-bom", DownloadKind::Bom)?;
    hook_download_button(&document, "download-cut", DownloadKind::CutList)?;

    // Part select
    if let Some(select) = document.get_element_by_id("part-select") {
        let select: HtmlSelectElement = select.dyn_into()?;
        let closure = Closure::wrap(Box::new(move || {
            if let Err(e) = on_part_selected() {
                web_sys::console::error_1(&format!("Part selection failed: {:?}", e).into());
            }
        }) as Box<dyn FnMut()>);
        select.set_onchange(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Initial design
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.rotation_x = 0.5;
        s.rotation_y = 0.75;
        s.zoom = 1.0;
    });
    generate_and_render()?;

    Ok(())
}

#[derive(Clone, Copy)]
enum DownloadKind {
    Step,
    Bom,
    CutList,
}

fn hook_download_button(document: &Document, id: &str, kind: DownloadKind) -> Result<(), JsValue> {
    let btn = document
        .get_element_by_id(id)
        .ok_or_else(|| format!("Button {id} not found"))?;
    let btn: HtmlElement = btn.dyn_into()?;
    let closure = Closure::wrap(Box::new(move || {
        if let Err(e) = download_current(kind) {
            web_sys::console::error_1(&format!("Download failed: {:?}", e).into());
        }
    }) as Box<dyn FnMut()>);
    btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(())
}

fn read_f32(document: &Document, id: &str) -> Result<f32, JsValue> {
    let input = document
        .get_element_by_id(id)
        .ok_or_else(|| format!("Input {id} not found"))?;
    let input: HtmlInputElement = input.dyn_into()?;
    let v = input
        .value()
        .parse::<f32>()
        .map_err(|_| JsValue::from_str("Invalid number"))?;
    Ok(v)
}

fn set_text(document: &Document, id: &str, text: &str) -> Result<(), JsValue> {
    if let Some(elem) = document.get_element_by_id(id) {
        let elem: HtmlElement = elem.dyn_into()?;
        elem.set_inner_text(text);
    }
    Ok(())
}

fn generate_and_render() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Update spec from inputs (minimal v1: product dims + weight)
    let (l, w, h, wt) = (
        read_f32(&document, "prod-length")?,
        read_f32(&document, "prod-width")?,
        read_f32(&document, "prod-height")?,
        read_f32(&document, "prod-weight")?,
    );

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.spec.product.length = l;
        state.spec.product.width = w;
        state.spec.product.height = h;
        state.spec.product.weight = wt;
        state.design = Some(design_from_spec(&state.spec));

        if state.selected_part_id.is_none() {
            if let Some(design) = state.design.as_ref() {
                if let Some(first) = design.parts.first() {
                    state.selected_part_id = Some(first.id.clone());
                }
            }
        }
    });

    update_ui(&document)?;
    render()?;
    Ok(())
}

fn update_ui(document: &Document) -> Result<(), JsValue> {
    STATE.with(|state| {
        let state = state.borrow();
        if let Some(design) = state.design.as_ref() {
            let overall = format!(
                "{:.2} x {:.2} x {:.2}",
                design.geometry.overall_length, design.geometry.overall_width, design.geometry.overall_height
            );
            set_text(document, "overall-dims", &overall).ok();
            set_text(document, "parts-count", &design.parts.len().to_string()).ok();

            // Populate part select
            if let Some(elem) = document.get_element_by_id("part-select") {
                if let Ok(select) = elem.dyn_into::<HtmlSelectElement>() {
                    // Clear options
                    select.set_inner_html("");

                    let mut parts: Vec<&CratePart> = design.parts.iter().collect();
                    parts.sort_by(|a, b| a.id.cmp(&b.id));

                    for part in parts {
                        let opt = document.create_element("option").ok();
                        if let Some(opt) = opt {
                            opt.set_attribute("value", &part.id).ok();
                            opt.set_inner_html(&format!("{} — {}", part.id, part.name));
                            select.append_child(&opt).ok();
                        }
                    }

                    if let Some(sel) = state.selected_part_id.as_ref() {
                        select.set_value(sel);
                    }
                }
            }

            update_selected_part_details(document).ok();
        }
    });

    Ok(())
}

fn on_part_selected() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let select = document
        .get_element_by_id("part-select")
        .ok_or("part-select not found")?;
    let select: HtmlSelectElement = select.dyn_into()?;
    let value = select.value();

    STATE.with(|state| state.borrow_mut().selected_part_id = Some(value));
    update_selected_part_details(&document)?;
    render()?;
    Ok(())
}

fn update_selected_part_details(document: &Document) -> Result<(), JsValue> {
    STATE.with(|state| {
        let state = state.borrow();
        let Some(design) = state.design.as_ref() else { return; };
        let Some(sel) = state.selected_part_id.as_ref() else { return; };
        let Some(part) = design.parts.iter().find(|p| &p.id == sel) else { return; };

        let size = part.bounds.size();
        let details = format!(
            "id: {id}\nname: {name}\ncategory: {cat:?}\n\nbounds (in):\n  min: ({minx:.2}, {miny:.2}, {minz:.2})\n  max: ({maxx:.2}, {maxy:.2}, {maxz:.2})\n  size: ({sx:.2}, {sy:.2}, {sz:.2})\n\nmetadata:\n  {meta}\n",
            id = part.id,
            name = part.name,
            cat = part.category,
            minx = part.bounds.min.x,
            miny = part.bounds.min.y,
            minz = part.bounds.min.z,
            maxx = part.bounds.max.x,
            maxy = part.bounds.max.y,
            maxz = part.bounds.max.z,
            sx = size.x,
            sy = size.y,
            sz = size.z,
            meta = part.metadata.clone().unwrap_or_else(|| "-".to_string()),
        );

        if let Some(elem) = document.get_element_by_id("part-details") {
            if let Ok(elem) = elem.dyn_into::<HtmlElement>() {
                elem.set_inner_text(&details);
            }
        }
    });
    Ok(())
}

fn set_view(view: &str) {
    use std::f32::consts::PI;
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
                state.rotation_x = 0.5;
                state.rotation_y = 0.75;
            }
        }
    });
    let _ = render();
}

fn setup_viewport_events(document: &Document) -> Result<(), JsValue> {
    let canvas = document
        .get_element_by_id("viewport-canvas")
        .ok_or("viewport-canvas not found")?;
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
            STATE.with(|state| state.borrow_mut().dragging = false);
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas.set_onmouseup(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Mouse leave
    {
        let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
            STATE.with(|state| state.borrow_mut().dragging = false);
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas.set_onmouseleave(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Mouse move (rotate)
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
                }
            });
            let _ = render();
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas.set_onmousemove(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Wheel (zoom)
    {
        let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
            event.prevent_default();
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                let delta = event.delta_y() as f32 * 0.001;
                state.zoom = (state.zoom - delta).clamp(0.35, 6.0);
            });
            let _ = render();
        }) as Box<dyn FnMut(WheelEvent)>);
        canvas.set_onwheel(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    Ok(())
}

fn render() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let canvas = document
        .get_element_by_id("viewport-canvas")
        .ok_or("viewport-canvas not found")?;
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

    // Draw
    STATE.with(|state| {
        let state = state.borrow();
        if let Some(design) = state.design.as_ref() {
            draw_design(
                &ctx,
                design,
                css_width,
                css_height,
                state.rotation_x,
                state.rotation_y,
                state.zoom,
                state.selected_part_id.as_deref(),
            )
            .ok();
        }
    });

    Ok(())
}

fn compute_bbox(parts: &[CratePart]) -> Option<BoundingBox> {
    if parts.is_empty() {
        return None;
    }
    let mut min = parts[0].bounds.min;
    let mut max = parts[0].bounds.max;
    for p in parts.iter().skip(1) {
        min.x = min.x.min(p.bounds.min.x);
        min.y = min.y.min(p.bounds.min.y);
        min.z = min.z.min(p.bounds.min.z);
        max.x = max.x.max(p.bounds.max.x);
        max.y = max.y.max(p.bounds.max.y);
        max.z = max.z.max(p.bounds.max.z);
    }
    Some(BoundingBox::new(min, max))
}

fn draw_design(
    ctx: &CanvasRenderingContext2d,
    design: &CrateDesign,
    width: f64,
    height: f64,
    rot_x: f32,
    rot_y: f32,
    zoom: f32,
    selected: Option<&str>,
) -> Result<(), JsValue> {
    let cx = width / 2.0;
    let cy = height / 2.0;

    let mut parts: Vec<&CratePart> = design.parts.iter().collect();
    parts.sort_by(|a, b| a.id.cmp(&b.id));

    let bbox = compute_bbox(&design.parts).ok_or("No parts")?;
    let center = Point3::new(
        (bbox.min.x + bbox.max.x) * 0.5,
        (bbox.min.y + bbox.max.y) * 0.5,
        (bbox.min.z + bbox.max.z) * 0.5,
    );
    let size = bbox.size();
    let max_dim = size.x.max(size.y).max(size.z).max(1.0);
    let base_scale = 0.42 * (height.min(width) / max_dim as f64);
    let scale = base_scale * zoom as f64;

    let (sin_x, cos_x) = rot_x.sin_cos();
    let (sin_y, cos_y) = rot_y.sin_cos();

    let project = |p: &Point3| -> (f64, f64) {
        let x = p.x - center.x;
        let y = p.y - center.y;
        let z = p.z - center.z;

        // Rotate around Y
        let x1 = x * cos_y - z * sin_y;
        let z1 = x * sin_y + z * cos_y;

        // Rotate around X
        let y1 = y * cos_x - z1 * sin_x;

        let px = cx + (x1 as f64) * scale;
        let py = cy - (y1 as f64) * scale;
        (px, py)
    };

    let edges: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    for part in parts {
        let b = &part.bounds;
        let corners = [
            Point3::new(b.min.x, b.min.y, b.min.z),
            Point3::new(b.max.x, b.min.y, b.min.z),
            Point3::new(b.max.x, b.max.y, b.min.z),
            Point3::new(b.min.x, b.max.y, b.min.z),
            Point3::new(b.min.x, b.min.y, b.max.z),
            Point3::new(b.max.x, b.min.y, b.max.z),
            Point3::new(b.max.x, b.max.y, b.max.z),
            Point3::new(b.min.x, b.max.y, b.max.z),
        ];

        let stroke = match part.category {
            PartCategory::Lumber => "#ff6b35",
            PartCategory::Plywood => "#4aa3ff",
            PartCategory::Hardware => "#44ff88",
            PartCategory::Decal => "#c77dff",
        };

        let is_selected = selected.map(|s| s == part.id).unwrap_or(false);
        ctx.set_stroke_style(&JsValue::from_str(if is_selected { "#ffffff" } else { stroke }));
        ctx.set_line_width(if is_selected { 2.25 } else { 1.25 });

        for (a, b) in edges {
            let (x1, y1) = project(&corners[a]);
            let (x2, y2) = project(&corners[b]);
            ctx.begin_path();
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
            ctx.stroke();
        }
    }

    // Axis indicator (reusing CAD idea)
    draw_axis_indicator(ctx, height, sin_x, cos_x, sin_y, cos_y)?;

    Ok(())
}

fn draw_axis_indicator(
    ctx: &CanvasRenderingContext2d,
    height: f64,
    sin_x: f32,
    cos_x: f32,
    sin_y: f32,
    cos_y: f32,
) -> Result<(), JsValue> {
    let origin_x = 55.0;
    let origin_y = height - 55.0;
    let axis_len = 32.0;

    let project_axis = |ax: f32, ay: f32, az: f32| -> (f64, f64) {
        let x1 = ax * cos_y - az * sin_y;
        let y1 = ay * cos_x - (ax * sin_y + az * cos_y) * sin_x;
        (x1 as f64 * axis_len, -y1 as f64 * axis_len)
    };

    // X
    let (dx, dy) = project_axis(1.0, 0.0, 0.0);
    ctx.set_stroke_style(&JsValue::from_str("#ff4444"));
    ctx.set_line_width(2.0);
    ctx.begin_path();
    ctx.move_to(origin_x, origin_y);
    ctx.line_to(origin_x + dx, origin_y + dy);
    ctx.stroke();
    ctx.set_fill_style(&JsValue::from_str("#ff4444"));
    ctx.set_font("10px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace");
    ctx.fill_text("X", origin_x + dx + 6.0, origin_y + dy)?;

    // Y
    let (dx, dy) = project_axis(0.0, 1.0, 0.0);
    ctx.set_stroke_style(&JsValue::from_str("#44ff44"));
    ctx.begin_path();
    ctx.move_to(origin_x, origin_y);
    ctx.line_to(origin_x + dx, origin_y + dy);
    ctx.stroke();
    ctx.set_fill_style(&JsValue::from_str("#44ff44"));
    ctx.fill_text("Y", origin_x + dx + 6.0, origin_y + dy)?;

    // Z
    let (dx, dy) = project_axis(0.0, 0.0, 1.0);
    ctx.set_stroke_style(&JsValue::from_str("#4444ff"));
    ctx.begin_path();
    ctx.move_to(origin_x, origin_y);
    ctx.line_to(origin_x + dx, origin_y + dy);
    ctx.stroke();
    ctx.set_fill_style(&JsValue::from_str("#4444ff"));
    ctx.fill_text("Z", origin_x + dx + 6.0, origin_y + dy)?;

    Ok(())
}

fn download_current(kind: DownloadKind) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let (filename, contents, _mime) = STATE.with(
        |state| -> Result<(String, String, &'static str), JsValue> {
        let state = state.borrow();
        let design = state
            .design
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No design"))?;
        let l = state.spec.product.length;
        let w = state.spec.product.width;
        let h = state.spec.product.height;

        let suffix = format!("{:.0}x{:.0}x{:.0}", l, w, h);

        match kind {
            DownloadKind::Step => Ok((
                format!("autocrate_{suffix}.step"),
                export_step(design),
                "application/step",
            )),
            DownloadKind::Bom => Ok((
                format!("autocrate_{suffix}_bom.csv"),
                export_bom_csv(design),
                "text/csv",
            )),
            DownloadKind::CutList => Ok((
                format!("autocrate_{suffix}_cut_list.csv"),
                export_cut_list_csv(design),
                "text/csv",
            )),
        }
    },
    )?;

    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&contents));
    let blob = Blob::new_with_str_sequence(&array)?;

    let url = Url::create_object_url_with_blob(&blob)?;

    let a = document.create_element("a")?;
    let a: HtmlAnchorElement = a.dyn_into()?;
    a.set_href(&url);
    a.set_download(&filename);
    a.click();

    Url::revoke_object_url(&url)?;
    Ok(())
}
