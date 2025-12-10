//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | ARCH/src/lib.rs
//! PURPOSE: Terminal-style tree view architecture explorer with drill-down navigation
//! MODIFIED: 2025-12-09
//! LAYER: ARCH (architecture explorer)
//! ═══════════════════════════════════════════════════════════════════════════════

#![allow(unexpected_cfgs)]

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, TouchEvent};

mod audit;
mod graph;
pub use audit::{CrateAudit, GitMetadata, ValidationStatus};
pub use graph::{CrateInfo, CrateLayer, DependencyGraph};

const WORKSPACE_DATA: &str = include_str!("workspace_data.json");

// Colors
struct Colors;
impl Colors {
    const BG: &'static str = "#0a0a0f";
    const TEXT: &'static str = "#ffffff";
    const DIM: &'static str = "#555566";
    const DNA: &'static str = "#3b82f6";
    const CORE: &'static str = "#14b8a6";
    const PROJECT: &'static str = "#a855f7";
    const TOOL: &'static str = "#f59e0b";
    const LEARN: &'static str = "#22c55e";
    const BACK: &'static str = "#888899";
}

#[derive(Clone)]
enum LineAction {
    None,
    Back,
    Enter(String),      // Enter a folder (category or subfolder)
    Select(String),     // Select a crate (leaf node)
}

#[derive(Clone)]
struct TreeLine {
    name: String,
    suffix: String,
    color: &'static str,
    action: LineAction,
    // For info panel when selected
    path: Option<String>,
    deps: Vec<String>,
    layer: Option<&'static str>,
}

struct AppState {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    width: f64,
    height: f64,
    dpr: f64,
    lines: Vec<TreeLine>,
    selected: Option<String>,   // Selected crate name
    current_path: Vec<String>,  // Navigation path (e.g., ["TOOLS", "CORE"])
    scroll_y: f64,
    max_scroll: f64,
    line_height: f64,
    font_size: f64,
    graph: DependencyGraph,
}

impl AppState {
    fn new(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d) -> Self {
        let window = window().unwrap();
        let dpr = window.device_pixel_ratio();
        let rect = canvas.get_bounding_client_rect();

        let width = rect.width();
        let height = rect.height();

        canvas.set_width((width * dpr) as u32);
        canvas.set_height((height * dpr) as u32);
        ctx.scale(dpr, dpr).ok();

        let graph: DependencyGraph = serde_json::from_str(WORKSPACE_DATA).unwrap_or_default();

        let is_mobile = width < 500.0;
        let font_size = if is_mobile { 12.0 } else { 14.0 };
        let line_height = font_size * 1.6;

        let mut state = Self {
            canvas,
            ctx,
            width,
            height,
            dpr,
            lines: Vec::new(),
            selected: None,
            current_path: Vec::new(),
            scroll_y: 0.0,
            max_scroll: 0.0,
            line_height,
            font_size,
            graph,
        };

        state.build_tree();
        state
    }

    fn build_tree(&mut self) {
        self.lines.clear();
        self.scroll_y = 0.0;

        // Title with current path
        let title = if self.current_path.is_empty() {
            "ARCH".to_string()
        } else {
            format!("ARCH/{}", self.current_path.join("/"))
        };

        self.lines.push(TreeLine {
            name: title,
            suffix: " Architecture Explorer".into(),
            color: Colors::TEXT,
            action: LineAction::None,
            path: None,
            deps: vec![],
            layer: None,
        });

        // Separator
        self.lines.push(TreeLine {
            name: "────────────────────────────────".into(),
            suffix: String::new(),
            color: Colors::DIM,
            action: LineAction::None,
            path: None,
            deps: vec![],
            layer: None,
        });

        // Add back navigation if not at root
        if !self.current_path.is_empty() {
            self.lines.push(TreeLine {
                name: "../".into(),
                suffix: "  [back]".into(),
                color: Colors::BACK,
                action: LineAction::Back,
                path: None,
                deps: vec![],
                layer: None,
            });
            self.lines.push(TreeLine {
                name: String::new(),
                suffix: String::new(),
                color: Colors::DIM,
                action: LineAction::None,
                path: None,
                deps: vec![],
                layer: None,
            });
        }

        // Build content based on current path
        let path_strs: Vec<&str> = self.current_path.iter().map(|s| s.as_str()).collect();
        match path_strs.as_slice() {
            [] => self.build_root(),
            ["DNA"] => self.build_dna(),
            ["TOOLS"] => self.build_tools(),
            ["TOOLS", "CORE"] => self.build_tools_core(),
            ["SIMULATION"] => self.build_simulation(),
            ["SIMULATION", "CORE"] => self.build_simulation_core(),
            ["LEARN"] => self.build_learn(),
            ["STANDALONE"] => self.build_standalone(),
            _ => {}
        }

        // Calculate max scroll
        let content_height = self.lines.len() as f64 * self.line_height + 40.0;
        let panel_height = if self.selected.is_some() { 100.0 } else { 0.0 };
        self.max_scroll = (content_height - self.height + panel_height + 20.0).max(0.0);
    }

    fn build_root(&mut self) {
        // Root level: show all top-level categories
        self.add_folder("DNA/", "[Foundation]", Colors::DNA, "DNA");
        self.add_folder("TOOLS/", "[Utilities & Engines]", Colors::TOOL, "TOOLS");
        self.add_folder("SIMULATION/", "[Data & Phenomena]", Colors::CORE, "SIMULATION");
        self.add_folder("LEARN/", "[Tutorials]", Colors::LEARN, "LEARN");
        self.add_folder("STANDALONE/", "[Applications]", Colors::PROJECT, "STANDALONE");
    }

    fn build_dna(&mut self) {
        let crates: Vec<_> = self.graph.crates.iter()
            .filter(|c| c.path.starts_with("DNA/") && c.path != "DNA")
            .cloned()
            .collect();

        for c in crates {
            self.add_crate(&c, Colors::DNA, "Foundation");
        }
    }

    fn build_tools(&mut self) {
        // CORE subfolder
        self.add_folder("CORE/", "[Engines]", Colors::CORE, "CORE");
        self.lines.push(TreeLine {
            name: String::new(),
            suffix: String::new(),
            color: Colors::DIM,
            action: LineAction::None,
            path: None,
            deps: vec![],
            layer: None,
        });

        // Tool projects (not in CORE)
        let crates: Vec<_> = self.graph.crates.iter()
            .filter(|c| c.path.starts_with("TOOLS/") && !c.path.contains("/CORE/"))
            .cloned()
            .collect();

        for c in crates {
            self.add_crate(&c, Colors::TOOL, "Tool");
        }
    }

    fn build_tools_core(&mut self) {
        // All CORE engines under TOOLS + SPICE_ENGINE (visually grouped here)
        let mut crates: Vec<_> = self.graph.crates.iter()
            .filter(|c| c.path.starts_with("TOOLS/CORE/") || c.path == "SIMULATION/CORE/SPICE_ENGINE")
            .cloned()
            .collect();
        crates.sort_by(|a, b| a.name.cmp(&b.name));

        for c in crates {
            self.add_crate(&c, Colors::CORE, "Engine");
        }
    }

    fn build_simulation(&mut self) {
        // CORE subfolder
        self.add_folder("CORE/", "[Engines]", Colors::CORE, "CORE");
        self.lines.push(TreeLine {
            name: String::new(),
            suffix: String::new(),
            color: Colors::DIM,
            action: LineAction::None,
            path: None,
            deps: vec![],
            layer: None,
        });

        // Simulation projects (not in CORE)
        let crates: Vec<_> = self.graph.crates.iter()
            .filter(|c| c.path.starts_with("SIMULATION/") && !c.path.contains("/CORE/"))
            .cloned()
            .collect();

        for c in crates {
            self.add_crate(&c, Colors::CORE, "Simulation");
        }
    }

    fn build_simulation_core(&mut self) {
        // SIMULATION CORE engines (excludes SPICE which is visually under TOOLS)
        let crates: Vec<_> = self.graph.crates.iter()
            .filter(|c| c.path.starts_with("SIMULATION/CORE/") && c.path != "SIMULATION/CORE/SPICE_ENGINE")
            .cloned()
            .collect();

        for c in crates {
            self.add_crate(&c, Colors::CORE, "Engine");
        }
    }

    fn build_learn(&mut self) {
        let crates: Vec<_> = self.graph.crates.iter()
            .filter(|c| c.path.starts_with("LEARN/"))
            .cloned()
            .collect();

        for c in crates {
            self.add_crate(&c, Colors::LEARN, "Tutorial");
        }
    }

    fn build_standalone(&mut self) {
        let crates: Vec<_> = self.graph.crates.iter()
            .filter(|c| c.layer == CrateLayer::Project)
            .cloned()
            .collect();

        for c in crates {
            self.add_crate(&c, Colors::PROJECT, "Application");
        }
    }

    fn add_folder(&mut self, name: &str, desc: &str, color: &'static str, target: &str) {
        self.lines.push(TreeLine {
            name: name.into(),
            suffix: format!("  {}", desc),
            color,
            action: LineAction::Enter(target.to_string()),
            path: None,
            deps: vec![],
            layer: None,
        });
    }

    fn add_crate(&mut self, c: &CrateInfo, color: &'static str, layer: &'static str) {
        let suffix = if c.dependencies.is_empty() {
            String::new()
        } else {
            format!("  -> {}", c.dependencies.join(", "))
        };

        self.lines.push(TreeLine {
            name: format!("{}/", c.name.to_uppercase().replace("-", "_")),
            suffix,
            color,
            action: LineAction::Select(c.name.clone()),
            path: Some(c.path.clone()),
            deps: c.dependencies.clone(),
            layer: Some(layer),
        });
    }

    fn navigate(&mut self, action: &LineAction) {
        match action {
            LineAction::Back => {
                self.current_path.pop();
                self.selected = None;
                self.build_tree();
            }
            LineAction::Enter(target) => {
                self.current_path.push(target.clone());
                self.selected = None;
                self.build_tree();
            }
            LineAction::Select(name) => {
                if self.selected.as_ref() == Some(name) {
                    self.selected = None;
                } else {
                    self.selected = Some(name.clone());
                }
            }
            LineAction::None => {}
        }
    }

    fn line_at(&self, y: f64) -> Option<&TreeLine> {
        let scroll_y = y + self.scroll_y;
        let start_y = 20.0;
        let idx = ((scroll_y - start_y) / self.line_height) as usize;
        self.lines.get(idx).filter(|l| !matches!(l.action, LineAction::None))
    }

    fn render(&self) {
        let ctx = &self.ctx;

        // Clear
        ctx.set_fill_style(&JsValue::from_str(Colors::BG));
        ctx.fill_rect(0.0, 0.0, self.width, self.height);

        // Calculate panel height for scroll area
        let panel_h = if self.selected.is_some() {
            if self.width < 400.0 { 90.0 } else { 80.0 }
        } else {
            0.0
        };

        ctx.save();
        ctx.translate(0.0, -self.scroll_y).ok();

        let x = 16.0;
        let mut y = 20.0 + self.font_size;

        ctx.set_font(&format!("{}px monospace", self.font_size));
        ctx.set_text_baseline("top");

        for line in &self.lines {
            let is_selected = match &line.action {
                LineAction::Select(name) => self.selected.as_ref() == Some(name),
                _ => false,
            };

            // Draw highlight background for selected
            if is_selected {
                ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.1)"));
                ctx.fill_rect(0.0, y - 2.0, self.width, self.line_height);
            }

            // Draw name
            ctx.set_fill_style(&JsValue::from_str(line.color));
            ctx.fill_text(&line.name, x, y).ok();

            // Draw suffix in dim color
            if !line.suffix.is_empty() {
                let suffix_x = x + line.name.chars().count() as f64 * self.font_size * 0.6;
                ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
                ctx.fill_text(&line.suffix, suffix_x, y).ok();
            }

            y += self.line_height;
        }

        ctx.restore();

        // Draw info panel if something selected
        if let Some(ref name) = self.selected {
            if let Some(line) = self.lines.iter().find(|l| {
                matches!(&l.action, LineAction::Select(n) if n == name)
            }) {
                self.draw_info_panel(line, panel_h);
            }
        }
    }

    fn draw_info_panel(&self, line: &TreeLine, panel_h: f64) {
        let ctx = &self.ctx;
        let y = self.height - panel_h;
        let is_mobile = self.width < 400.0;
        let x = 16.0;
        let small_font = self.font_size - 1.0;

        // Background
        ctx.set_fill_style(&JsValue::from_str("rgba(10, 10, 15, 0.98)"));
        ctx.fill_rect(0.0, y, self.width, panel_h);

        // Top border
        ctx.set_fill_style(&JsValue::from_str(line.color));
        ctx.fill_rect(0.0, y, self.width, 2.0);

        ctx.set_font(&format!("{}px monospace", small_font));
        ctx.set_text_baseline("top");

        let mut ty = y + 10.0;
        let row_h = small_font * 1.4;

        // Name (always first)
        ctx.set_fill_style(&JsValue::from_str(line.color));
        ctx.fill_text(&line.name, x, ty).ok();

        if is_mobile {
            // Mobile: stack vertically
            ty += row_h;

            // Path
            ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
            ctx.fill_text("Path: ", x, ty).ok();
            ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
            ctx.fill_text(line.path.as_deref().unwrap_or("-"), x + 45.0, ty).ok();
            ty += row_h;

            // Layer
            ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
            ctx.fill_text("Layer: ", x, ty).ok();
            ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
            ctx.fill_text(line.layer.unwrap_or("-"), x + 50.0, ty).ok();
            ty += row_h;

            // Deps
            ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
            ctx.fill_text("Deps: ", x, ty).ok();
            ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
            let deps = if line.deps.is_empty() { "(none)".into() } else { line.deps.join(", ") };
            ctx.fill_text(&deps, x + 45.0, ty).ok();
        } else {
            // Desktop: Layer on same line as name
            ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
            ctx.fill_text(line.layer.unwrap_or("-"), x + 200.0, ty).ok();
            ty += row_h;

            // Path
            ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
            ctx.fill_text("Path:", x, ty).ok();
            ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
            ctx.fill_text(line.path.as_deref().unwrap_or("-"), x + 50.0, ty).ok();
            ty += row_h;

            // Deps
            ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
            ctx.fill_text("Deps:", x, ty).ok();
            ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
            let deps = if line.deps.is_empty() { "(none)".into() } else { line.deps.join(", ") };
            ctx.fill_text(&deps, x + 50.0, ty).ok();
        }
    }

    fn handle_resize(&mut self) {
        let window = window().unwrap();
        let dpr = window.device_pixel_ratio();
        let rect = self.canvas.get_bounding_client_rect();

        self.width = rect.width();
        self.height = rect.height();
        self.dpr = dpr;

        self.canvas.set_width((self.width * dpr) as u32);
        self.canvas.set_height((self.height * dpr) as u32);

        self.ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).ok();
        self.ctx.scale(dpr, dpr).ok();

        let is_mobile = self.width < 500.0;
        self.font_size = if is_mobile { 12.0 } else { 14.0 };
        self.line_height = self.font_size * 1.6;

        self.build_tree();
    }
}

thread_local! {
    static APP: RefCell<Option<AppState>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let canvas = document
        .get_element_by_id("canvas")
        .ok_or("No canvas")?
        .dyn_into::<HtmlCanvasElement>()?;

    let ctx = canvas
        .get_context("2d")?
        .ok_or("No 2d context")?
        .dyn_into::<CanvasRenderingContext2d>()?;

    let state = AppState::new(canvas.clone(), ctx);
    APP.with(|app| *app.borrow_mut() = Some(state));

    render();
    setup_events(&canvas)?;

    Ok(())
}

fn setup_events(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
    // Click
    {
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            APP.with(|app| {
                if let Some(ref mut state) = *app.borrow_mut() {
                    let rect = state.canvas.get_bounding_client_rect();
                    let y = event.client_y() as f64 - rect.top();

                    if let Some(line) = state.line_at(y) {
                        let action = line.action.clone();
                        state.navigate(&action);
                    }
                }
            });
            render();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Touch
    {
        let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
            if let Some(touch) = event.touches().get(0) {
                APP.with(|app| {
                    if let Some(ref mut state) = *app.borrow_mut() {
                        let rect = state.canvas.get_bounding_client_rect();
                        let y = touch.client_y() as f64 - rect.top();

                        if let Some(line) = state.line_at(y) {
                            let action = line.action.clone();
                            state.navigate(&action);
                        }
                    }
                });
                render();
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Wheel scroll
    {
        let closure = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
            event.prevent_default();
            APP.with(|app| {
                if let Some(ref mut state) = *app.borrow_mut() {
                    state.scroll_y = (state.scroll_y + event.delta_y() * 0.5)
                        .clamp(0.0, state.max_scroll);
                }
            });
            render();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Resize
    {
        let closure = Closure::wrap(Box::new(move || {
            APP.with(|app| {
                if let Some(ref mut state) = *app.borrow_mut() {
                    state.handle_resize();
                }
            });
            render();
        }) as Box<dyn FnMut()>);
        window()
            .unwrap()
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

fn render() {
    APP.with(|app| {
        if let Some(ref state) = *app.borrow() {
            state.render();
        }
    });
}
