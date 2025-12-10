//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: render.rs | ARCH/src/render.rs
//! PURPOSE: DOM-based HTML rendering for ARCH file explorer and file viewer
//! MODIFIED: 2025-12-09
//! LAYER: ARCH (architecture explorer)
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::{AppState, LineAction, TreeLine, ViewMode};
use wasm_bindgen::JsValue;
use web_sys::{Document, Element};

const Colors_BG: &str = "#0a0a0f";
const Colors_TEXT: &str = "#ffffff";
const Colors_DIM: &str = "#555566";
const Colors_FILE: &str = "#ccccdd";

pub struct ArchRenderer {
    document: Document,
    root: Element,
}

impl ArchRenderer {
    pub fn new(root_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        let root = document
            .get_element_by_id(root_id)
            .ok_or("Root element not found")?;

        Ok(Self { document, root })
    }

    pub fn render(&self, state: &AppState) -> Result<(), JsValue> {
        let html = match &state.view_mode {
            ViewMode::Tree => self.build_tree_html(state),
            ViewMode::FileViewer { path } => self.build_file_viewer_html(state, path),
        };

        self.root.set_inner_html(&html);
        Ok(())
    }

    fn build_tree_html(&self, state: &AppState) -> String {
        let mut html = String::new();

        html.push_str(r#"<div class="arch-container">"#);

        // Header with breadcrumb
        html.push_str(&self.render_header(state));

        // Tree list
        html.push_str(r#"<main class="arch-tree" id="tree-list">"#);
        for line in &state.lines {
            html.push_str(&self.render_tree_line(line, state));
        }
        html.push_str("</main>");

        html.push_str("</div>");
        html
    }

    fn render_header(&self, state: &AppState) -> String {
        let title = if state.current_path.is_empty() {
            "ARCH".to_string()
        } else {
            format!("ARCH/{}", state.current_path.join("/"))
        };

        format!(
            r#"<header class="arch-header"><span class="arch-header__title">{}</span><span class="arch-header__subtitle"> File Explorer</span></header>"#,
            escape_html(&title)
        )
    }

    fn render_tree_line(&self, line: &TreeLine, state: &AppState) -> String {
        let is_selected = match &line.action {
            LineAction::SelectFile(path) => state.selected_file.as_ref() == Some(path),
            _ => false,
        };

        let (action_type, action_data) = match &line.action {
            LineAction::Back => ("back", String::new()),
            LineAction::EnterFolder(folder) => {
                ("folder", format!(r#" data-target="{}""#, escape_html(folder)))
            }
            LineAction::SelectFile(path) => {
                ("file", format!(r#" data-path="{}""#, escape_html(path)))
            }
            LineAction::None => ("none", String::new()),
        };

        let selected_class = if is_selected {
            " tree-line--selected"
        } else {
            ""
        };
        let type_class = format!("tree-line--{}", action_type);

        format!(
            r#"<div class="tree-line {}{}" data-action="{}"{}><span class="tree-line__name" style="color: {}">{}</span><span class="tree-line__suffix">{}</span></div>"#,
            type_class,
            selected_class,
            action_type,
            action_data,
            line.color,
            escape_html(&line.name),
            escape_html(&line.suffix)
        )
    }

    fn build_file_viewer_html(&self, state: &AppState, path: &str) -> String {
        let file_info = state.file_db.get(path);
        let content = state.file_content_cache.get(path);

        if file_info.is_none() || content.is_none() {
            return format!(
                r#"<div class="file-viewer file-viewer--active"><div class="file-viewer__header"><button class="file-viewer__close" data-action="close-file">← Back to files</button></div><div class="file-viewer__content"><p style="color: {}; padding: 20px;">File not found or content not available</p></div></div>"#,
                Colors_TEXT
            );
        }

        let file_info = file_info.unwrap();
        let content = content.unwrap();

        let mut html = String::new();

        // Container
        html.push_str(r#"<div class="file-viewer file-viewer--active">"#);

        // Header
        html.push_str(r#"<header class="file-viewer__header">"#);
        html.push_str(
            r#"<button class="file-viewer__close" data-action="close-file">← Back to files</button>"#,
        );
        html.push_str(r#"<div class="file-viewer__title">"#);
        html.push_str(&format!(
            r#"<span class="file-viewer__filename">{}</span>"#,
            escape_html(&file_info.name)
        ));
        html.push_str(&format!(
            r#"<span class="file-viewer__path">{}</span>"#,
            escape_html(&file_info.path)
        ));
        html.push_str("</div></header>");

        // Content with line numbers
        html.push_str(r#"<div class="file-viewer__content">"#);
        html.push_str(r#"<pre class="code-display"><code>"#);

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            html.push_str(&format!(
                r#"<span class="line-number">{:4}</span>"#,
                line_num
            ));
            html.push_str(&format!(
                r#"<span class="line-content">{}</span>
"#,
                escape_html(line)
            ));
        }

        html.push_str("</code></pre></div>");
        html.push_str("</div>");

        html
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
