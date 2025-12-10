//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | ARCH/src/lib.rs
//! PURPOSE: Terminal-style file-level architecture explorer with complete drill-down
//! MODIFIED: 2025-12-09
//! LAYER: ARCH (architecture explorer)
//! ═══════════════════════════════════════════════════════════════════════════════

#![allow(unexpected_cfgs)]

use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, TouchEvent};
use serde::{Deserialize, Serialize};

mod audit;
mod graph;
pub use audit::{CrateAudit, GitMetadata, ValidationStatus};
pub use graph::{CrateInfo, CrateLayer, DependencyGraph};

const WORKSPACE_DATA: &str = include_str!("workspace_data.json");
const FILE_DB: &str = include_str!("db.json");

// File metadata from db.json
#[derive(Debug, Clone, Deserialize, Serialize)]
struct FileInfo {
    path: String,
    name: String,
    purpose: String,
    main_function: String,
    #[serde(rename = "type")]
    file_type: String,
}

type FileDatabase = HashMap<String, FileInfo>;

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
    const FILE: &'static str = "#ccccdd";
}

#[derive(Clone)]
enum LineAction {
    None,
    Back,
    EnterFolder(String),     // Enter a folder/category
    SelectFile(String),      // Select a file (leaf node)
}

#[derive(Clone)]
struct TreeLine {
    name: String,
    suffix: String,
    color: &'static str,
    action: LineAction,
    // Metadata
    file_info: Option<FileInfo>,
}

struct AppState {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    width: f64,
    height: f64,
    dpr: f64,
    lines: Vec<TreeLine>,
    selected_file: Option<String>,  // Selected file path
    current_path: Vec<String>,      // Navigation path (e.g., ["TOOLS", "CAD", "src"])
    scroll_y: f64,
    max_scroll: f64,
    line_height: f64,
    font_size: f64,
    graph: DependencyGraph,
    file_db: FileDatabase,
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
        let file_db: FileDatabase = serde_json::from_str(FILE_DB).unwrap_or_default();

        let is_mobile = width < 500.0;
        let font_size = if is_mobile { 11.0 } else { 13.0 };
        let line_height = font_size * 1.5;

        let mut state = Self {
            canvas,
            ctx,
            width,
            height,
            dpr,
            lines: Vec::new(),
            selected_file: None,
            current_path: Vec::new(),
            scroll_y: 0.0,
            max_scroll: 0.0,
            line_height,
            font_size,
            graph,
            file_db,
        };

        state.build_tree();
        state
    }

    fn build_tree(&mut self) {
        self.lines.clear();
        self.scroll_y = 0.0;

        // Title with breadcrumb
        let title = if self.current_path.is_empty() {
            "ARCH".to_string()
        } else {
            format!("ARCH/{}", self.current_path.join("/"))
        };

        self.lines.push(TreeLine {
            name: title,
            suffix: " File Explorer".into(),
            color: Colors::TEXT,
            action: LineAction::None,
            file_info: None,
        });

        // Separator
        self.lines.push(TreeLine {
            name: "────────────────────────────────".into(),
            suffix: String::new(),
            color: Colors::DIM,
            action: LineAction::None,
            file_info: None,
        });

        // Back navigation
        if !self.current_path.is_empty() {
            self.lines.push(TreeLine {
                name: "../".into(),
                suffix: "  [back]".into(),
                color: Colors::BACK,
                action: LineAction::Back,
                file_info: None,
            });
            self.lines.push(TreeLine {
                name: String::new(),
                suffix: String::new(),
                color: Colors::DIM,
                action: LineAction::None,
                file_info: None,
            });
        }

        // Build content based on current path
        if self.current_path.is_empty() {
            // Root: Show top-level categories
            self.build_root_categories();
        } else {
            // Navigate into directory
            self.build_directory_contents();
        }

        // Calculate max scroll
        let content_height = self.lines.len() as f64 * self.line_height + 40.0;
        let panel_height = if self.selected_file.is_some() { 110.0 } else { 0.0 };
        self.max_scroll = (content_height - self.height + panel_height + 20.0).max(0.0);
    }

    fn build_root_categories(&mut self) {
        // Show main directory categories
        let categories = vec![
            ("DNA/", "[Foundation]", Colors::DNA),
            ("TOOLS/", "[Utilities]", Colors::TOOL),
            ("SIMULATION/", "[Simulations]", Colors::CORE),
            ("LEARN/", "[Tutorials]", Colors::LEARN),
            ("HELIOS/", "[Solar System]", Colors::PROJECT),
            ("WELCOME/", "[Landing Page]", Colors::PROJECT),
            ("BLOG/", "[Blog Platform]", Colors::PROJECT),
            ("ARCH/", "[Architecture Explorer]", Colors::PROJECT),
            ("SCRIPTS/", "[Build Scripts]", Colors::TOOL),
        ];

        for (name, desc, color) in categories {
            self.add_folder(name, desc, color);
        }
    }

    fn build_directory_contents(&mut self) {
        let prefix = self.current_path.join("/");

        // Collect all items in current directory
        let mut folders = std::collections::BTreeSet::new();
        let mut files = Vec::new();

        for (path, file_info) in &self.file_db {
            if path.starts_with(&prefix) {
                let relative = &path[prefix.len()..];
                if relative.starts_with('/') {
                    let relative = &relative[1..];

                    if let Some(idx) = relative.find('/') {
                        // It's a subfolder
                        let folder = &relative[..idx];
                        folders.insert(folder.to_string());
                    } else {
                        // It's a file in current directory
                        files.push(file_info.clone());
                    }
                }
            }
        }

        // Sort folders and files
        let mut sorted_folders: Vec<_> = folders.into_iter().collect();
        sorted_folders.sort();
        files.sort_by(|a, b| a.name.cmp(&b.name));

        let has_folders = !sorted_folders.is_empty();

        // Add folders first
        for folder in sorted_folders {
            let color = self.get_folder_color(&folder);
            self.add_folder(&format!("{}/", folder), "", color);
        }

        if !files.is_empty() && has_folders {
            self.lines.push(TreeLine {
                name: String::new(),
                suffix: String::new(),
                color: Colors::DIM,
                action: LineAction::None,
                file_info: None,
            });
        }

        // Add files
        for file in files {
            self.add_file(file);
        }
    }

    fn get_folder_color(&self, folder: &str) -> &'static str {
        match folder {
            "src" | "tests" | "examples" => Colors::CORE,
            "CORE" => Colors::CORE,
            _ => Colors::TOOL,
        }
    }

    fn add_folder(&mut self, name: &str, desc: &str, color: &'static str) {
        let folder_name = name.trim_end_matches('/');
        self.lines.push(TreeLine {
            name: name.into(),
            suffix: if desc.is_empty() { String::new() } else { format!("  {}", desc) },
            color,
            action: LineAction::EnterFolder(folder_name.to_string()),
            file_info: None,
        });
    }

    fn add_file(&mut self, file: FileInfo) {
        let suffix = if file.purpose.is_empty() {
            String::new()
        } else {
            let short_purpose = if file.purpose.len() > 120 {
                format!("  {}...", &file.purpose[..117])
            } else {
                format!("  {}", file.purpose)
            };
            short_purpose
        };

        self.lines.push(TreeLine {
            name: file.name.clone(),
            suffix,
            color: Colors::FILE,
            action: LineAction::SelectFile(file.path.clone()),
            file_info: Some(file),
        });
    }

    fn navigate(&mut self, action: &LineAction) {
        match action {
            LineAction::Back => {
                self.current_path.pop();
                self.selected_file = None;
                self.build_tree();
            }
            LineAction::EnterFolder(folder) => {
                self.current_path.push(folder.clone());
                self.selected_file = None;
                self.build_tree();
            }
            LineAction::SelectFile(path) => {
                if self.selected_file.as_ref() == Some(path) {
                    self.selected_file = None;
                } else {
                    self.selected_file = Some(path.clone());
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

        let panel_h = if self.selected_file.is_some() {
            if self.width < 400.0 { 110.0 } else { 100.0 }
        } else {
            0.0
        };

        ctx.save();
        ctx.translate(0.0, -self.scroll_y).ok();

        let x = 14.0;
        let mut y = 18.0 + self.font_size;

        ctx.set_font(&format!("{}px monospace", self.font_size));
        ctx.set_text_baseline("top");

        for line in &self.lines {
            let is_selected = match &line.action {
                LineAction::SelectFile(path) => self.selected_file.as_ref() == Some(path),
                _ => false,
            };

            // Highlight selected
            if is_selected {
                ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.08)"));
                ctx.fill_rect(0.0, y - 2.0, self.width, self.line_height);
            }

            // Draw name
            ctx.set_fill_style(&JsValue::from_str(line.color));
            ctx.fill_text(&line.name, x, y).ok();

            // Draw suffix (purpose/description)
            if !line.suffix.is_empty() {
                let suffix_x = x + line.name.chars().count() as f64 * self.font_size * 0.6;
                ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
                ctx.fill_text(&line.suffix, suffix_x, y).ok();
            }

            y += self.line_height;
        }

        ctx.restore();

        // Draw info panel for selected file
        if let Some(ref path) = self.selected_file {
            if let Some(file_info) = self.file_db.get(path) {
                self.draw_file_info_panel(file_info, panel_h);
            }
        }
    }

    fn draw_file_info_panel(&self, file: &FileInfo, panel_h: f64) {
        let ctx = &self.ctx;
        let y = self.height - panel_h;
        let is_mobile = self.width < 400.0;
        let x = 14.0;
        let small_font = (self.font_size - 1.0).max(10.0);

        // Background
        ctx.set_fill_style(&JsValue::from_str("rgba(10, 10, 15, 0.98)"));
        ctx.fill_rect(0.0, y, self.width, panel_h);

        // Top border
        ctx.set_fill_style(&JsValue::from_str(Colors::FILE));
        ctx.fill_rect(0.0, y, self.width, 2.0);

        ctx.set_font(&format!("{}px monospace", small_font));
        ctx.set_text_baseline("top");

        let mut ty = y + 8.0;
        let row_h = small_font * 1.4;

        // File name
        ctx.set_fill_style(&JsValue::from_str(Colors::FILE));
        ctx.fill_text(&file.name, x, ty).ok();
        ty += row_h;

        // Path
        ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
        ctx.fill_text("Path:", x, ty).ok();
        ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
        let path_text = if is_mobile && file.path.len() > 35 {
            format!("...{}", &file.path[file.path.len()-32..])
        } else {
            file.path.clone()
        };
        ctx.fill_text(&path_text, x + 40.0, ty).ok();
        ty += row_h;

        // Purpose
        ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
        ctx.fill_text("Info:", x, ty).ok();
        ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
        let purpose_text = if is_mobile && file.purpose.len() > 80 {
            format!("{}...", &file.purpose[..77])
        } else if file.purpose.len() > 150 {
            format!("{}...", &file.purpose[..147])
        } else {
            file.purpose.clone()
        };
        ctx.fill_text(&purpose_text, x + 40.0, ty).ok();
        ty += row_h;

        // Type and main function
        ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
        ctx.fill_text("Type:", x, ty).ok();
        ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
        ctx.fill_text(&file.file_type, x + 40.0, ty).ok();

        if !file.main_function.is_empty() && file.main_function != "N/A" {
            ctx.set_fill_style(&JsValue::from_str(Colors::DIM));
            ctx.fill_text("Entry:", x + 140.0, ty).ok();
            ctx.set_fill_style(&JsValue::from_str(Colors::TEXT));
            ctx.fill_text(&file.main_function, x + 185.0, ty).ok();
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
        self.font_size = if is_mobile { 11.0 } else { 13.0 };
        self.line_height = self.font_size * 1.5;

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
