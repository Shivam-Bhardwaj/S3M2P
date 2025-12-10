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
use serde::{Deserialize, Serialize};

mod audit;
mod graph;
mod render;
mod events;

pub use audit::{CrateAudit, GitMetadata, ValidationStatus};
pub use graph::{CrateInfo, CrateLayer, DependencyGraph};
use render::ArchRenderer;

const WORKSPACE_DATA: &str = include_str!("workspace_data.json");
const FILE_DB: &str = include_str!("db.json");

// File metadata from db.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub purpose: String,
    pub main_function: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub content: Option<String>,
}

type FileDatabase = HashMap<String, FileInfo>;

// View mode enum
#[derive(Clone, PartialEq)]
pub enum ViewMode {
    Tree,
    FileViewer { path: String },
}

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
pub enum LineAction {
    None,
    Back,
    EnterFolder(String),     // Enter a folder/category
    SelectFile(String),      // Select a file (leaf node)
}

#[derive(Clone)]
pub struct TreeLine {
    pub name: String,
    pub suffix: String,
    pub color: &'static str,
    pub action: LineAction,
    // Metadata
    pub file_info: Option<FileInfo>,
}

pub struct AppState {
    renderer: ArchRenderer,
    width: f64,
    height: f64,
    is_mobile: bool,
    lines: Vec<TreeLine>,
    selected_file: Option<String>,
    current_path: Vec<String>,
    view_mode: ViewMode,
    file_content_cache: HashMap<String, String>,
    line_height: f64,
    font_size: f64,
    graph: DependencyGraph,
    pub file_db: FileDatabase,
}

impl AppState {
    fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let width = window.inner_width()?.as_f64().unwrap_or(1024.0);
        let height = window.inner_height()?.as_f64().unwrap_or(768.0);

        let is_mobile = width < 500.0;
        let font_size = if is_mobile { 11.0 } else { 13.0 };
        let line_height = font_size * 1.5;

        let graph: DependencyGraph = serde_json::from_str(WORKSPACE_DATA).unwrap_or_default();
        let file_db: FileDatabase = serde_json::from_str(FILE_DB).unwrap_or_default();

        let renderer = ArchRenderer::new("arch-app")?;

        let mut state = Self {
            renderer,
            width,
            height,
            is_mobile,
            lines: Vec::new(),
            selected_file: None,
            current_path: Vec::new(),
            view_mode: ViewMode::Tree,
            file_content_cache: HashMap::new(),
            line_height,
            font_size,
            graph,
            file_db,
        };

        state.build_tree();
        Ok(state)
    }

    fn build_tree(&mut self) {
        self.lines.clear();

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
                self.selected_file = Some(path.clone());
                self.view_mode = ViewMode::FileViewer { path: path.clone() };
                // Load file content
                let _ = self.load_file_content(path);
            }
            LineAction::None => {}
        }
    }

    pub fn load_file_content(&mut self, path: &str) -> Result<String, JsValue> {
        // Check cache first
        if let Some(content) = self.file_content_cache.get(path) {
            return Ok(content.clone());
        }

        // Get from db.json (which now includes content)
        if let Some(file_info) = self.file_db.get(path) {
            if let Some(content) = &file_info.content {
                self.file_content_cache.insert(path.to_string(), content.clone());
                return Ok(content.clone());
            }
        }

        Err(JsValue::from_str("Content not available"))
    }

    pub fn close_file_viewer(&mut self) {
        self.view_mode = ViewMode::Tree;
        self.selected_file = None;
        self.build_tree();
    }

    fn render(&self) -> Result<(), JsValue> {
        self.renderer.render(self)
    }

    pub fn handle_resize(&mut self) {
        let window = web_sys::window().unwrap();
        self.width = window.inner_width().unwrap().as_f64().unwrap_or(1024.0);
        self.height = window.inner_height().unwrap().as_f64().unwrap_or(768.0);

        let new_is_mobile = self.width < 500.0;
        if new_is_mobile != self.is_mobile {
            self.is_mobile = new_is_mobile;
            self.font_size = if self.is_mobile { 11.0 } else { 13.0 };
            self.line_height = self.font_size * 1.5;
            self.build_tree();
        }
    }
}

thread_local! {
    static APP: RefCell<Option<AppState>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let state = AppState::new()?;
    APP.with(|app| *app.borrow_mut() = Some(state));

    render();
    events::setup_events("arch-app")?;

    Ok(())
}

pub fn render() {
    APP.with(|app| {
        if let Some(ref state) = *app.borrow() {
            let _ = state.render();
        }
    });
}
