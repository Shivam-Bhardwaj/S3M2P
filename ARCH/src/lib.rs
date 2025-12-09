//! ARCH - Architecture Visualization
//!
//! Hierarchical card-based view of the antimony-labs monorepo.

#![allow(unexpected_cfgs)]

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, WheelEvent};

mod audit;
mod graph;
pub use audit::{CrateAudit, GitMetadata, ValidationStatus};
pub use graph::{CrateInfo, CrateLayer, DependencyGraph};

const WORKSPACE_DATA: &str = include_str!("workspace_data.json");

// ============================================================================
// COLORS
// ============================================================================

struct Colors;
#[allow(dead_code)]
impl Colors {
    const BG: &'static str = "#0a0a0f";
    const CARD_BG: &'static str = "#14141f";
    const CARD_BORDER: &'static str = "#2a2a3a"; // Future: non-hover border
    const CARD_HOVER: &'static str = "#3b82f6";
    const TEXT_PRIMARY: &'static str = "#ffffff";
    const TEXT_SECONDARY: &'static str = "#888899";
    const TEXT_MUTED: &'static str = "#555566";

    const DNA: &'static str = "#3b82f6"; // Blue
    const CORE: &'static str = "#14b8a6"; // Teal
    const PROJECT: &'static str = "#a855f7"; // Purple
    const TOOL: &'static str = "#f59e0b"; // Amber
    const LEARN: &'static str = "#22c55e"; // Green
}

// ============================================================================
// CARD LAYOUT
// ============================================================================

#[derive(Clone)]
#[allow(dead_code)]
struct Card {
    name: String,
    description: String,
    color: &'static str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    children: Vec<String>,
    expanded: bool, // Future: click to expand/collapse
    audit: Option<CrateAudit>,
}

struct AppState {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    width: f64,
    height: f64,
    scroll_y: f64,
    hovered_card: Option<String>,
    cards: Vec<Card>,
}

impl AppState {
    fn new(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d) -> Self {
        // Handle high DPI displays
        let window = window().unwrap();
        let dpr = window.device_pixel_ratio();
        let rect = canvas.get_bounding_client_rect();

        let width = rect.width() * dpr;
        let height = rect.height() * dpr;

        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        ctx.scale(dpr, dpr).ok();

        let graph: DependencyGraph = serde_json::from_str(WORKSPACE_DATA).unwrap_or_default();

        let mut state = Self {
            canvas,
            ctx,
            width: rect.width(),   // Logical width
            height: rect.height(), // Logical height
            scroll_y: 0.0,
            hovered_card: None,
            cards: Vec::new(),
        };

        state.build_cards(&graph);
        state
    }

    fn build_cards(&mut self, graph: &DependencyGraph) {
        let card_width = 280.0;
        let card_height = 80.0;
        let small_card_height = 50.0;
        let padding = 20.0;
        let section_gap = 40.0;

        // Start below header
        let mut y = 70.0;
        let center_x = self.width / 2.0 - card_width / 2.0;

        // DNA - Foundation (Top)
        self.cards.push(Card {
            name: "DNA".to_string(),
            description: "Foundation layer: physics, CAD, simulation algorithms".to_string(),
            color: Colors::DNA,
            x: center_x,
            y,
            width: card_width,
            height: card_height,
            children: vec![],
            expanded: true,
            audit: Some(CrateAudit::new("DNA".to_string())),
        });
        y += card_height + section_gap;

        // CORE Engines - horizontal row
        let core_engines: Vec<_> = graph
            .crates
            .iter()
            .filter(|c| c.layer == CrateLayer::Core)
            .collect();

        let core_card_width = 160.0;
        let core_total_width = core_engines.len() as f64 * (core_card_width + padding) - padding;
        let mut core_x = (self.width - core_total_width) / 2.0;

        self.cards.push(Card {
            name: "CORE".to_string(),
            description: "Domain-specific engines".to_string(),
            color: Colors::CORE,
            x: center_x,
            y,
            width: card_width,
            height: 50.0,
            children: core_engines.iter().map(|c| c.name.clone()).collect(),
            expanded: true,
            audit: None,
        });
        y += 60.0;

        for crate_info in &core_engines {
            let short_name = crate_info.name.replace("-engine", "").to_uppercase();
            self.cards.push(Card {
                name: crate_info.name.clone(),
                description: short_name,
                color: Colors::CORE,
                x: core_x,
                y,
                width: core_card_width,
                height: small_card_height,
                children: vec![],
                expanded: false,
                audit: Some(CrateAudit::new(crate_info.name.clone())),
            });
            core_x += core_card_width + padding;
        }
        y += small_card_height + section_gap;

        // Projects row
        let projects: Vec<_> = graph
            .crates
            .iter()
            .filter(|c| c.layer == CrateLayer::Project)
            .collect();

        self.cards.push(Card {
            name: "PROJECTS".to_string(),
            description: "Web applications".to_string(),
            color: Colors::PROJECT,
            x: center_x,
            y,
            width: card_width,
            height: 50.0,
            children: projects.iter().map(|c| c.name.clone()).collect(),
            expanded: true,
            audit: None,
        });
        y += 60.0;

        let proj_card_width = 140.0;
        let proj_total_width = projects.len() as f64 * (proj_card_width + padding) - padding;
        let mut proj_x = (self.width - proj_total_width) / 2.0;

        for crate_info in &projects {
            self.cards.push(Card {
                name: crate_info.name.clone(),
                description: crate_info.path.clone(),
                color: Colors::PROJECT,
                x: proj_x,
                y,
                width: proj_card_width,
                height: small_card_height,
                children: vec![],
                expanded: false,
                audit: Some(CrateAudit::new(crate_info.name.clone())),
            });
            proj_x += proj_card_width + padding;
        }
        y += small_card_height + section_gap;

        // TOOLS section
        let tools: Vec<_> = graph
            .crates
            .iter()
            .filter(|c| c.layer == CrateLayer::Tool && c.path.starts_with("TOOLS/"))
            .collect();

        self.cards.push(Card {
            name: "TOOLS".to_string(),
            description: "Engineering utilities".to_string(),
            color: Colors::TOOL,
            x: padding,
            y,
            width: card_width,
            height: 50.0,
            children: tools.iter().map(|c| c.name.clone()).collect(),
            expanded: true,
            audit: None,
        });

        // LEARN section (same row)
        let learns: Vec<_> = graph
            .crates
            .iter()
            .filter(|c| c.layer == CrateLayer::Tool && c.path.starts_with("LEARN/"))
            .collect();

        self.cards.push(Card {
            name: "LEARN".to_string(),
            description: "Interactive tutorials".to_string(),
            color: Colors::LEARN,
            x: self.width - card_width - padding,
            y,
            width: card_width,
            height: 50.0,
            children: learns.iter().map(|c| c.name.clone()).collect(),
            expanded: true,
            audit: None,
        });
        y += 60.0;

        // Tools items (left column)
        let tool_x = padding + 20.0;
        let mut tool_y = y;
        let item_width = 200.0;

        for crate_info in &tools {
            let display_name = crate_info.name.replace("-", " ").to_uppercase();
            self.cards.push(Card {
                name: crate_info.name.clone(),
                description: display_name,
                color: Colors::TOOL,
                x: tool_x,
                y: tool_y,
                width: item_width,
                height: 40.0,
                children: vec![],
                expanded: false,
                audit: Some(CrateAudit::new(crate_info.name.clone())),
            });
            tool_y += 50.0;
        }

        // Learn items (right column)
        let learn_x = self.width - item_width - padding - 20.0;
        let mut learn_y = y;

        for crate_info in &learns {
            let display_name = crate_info
                .name
                .replace("-learn", "")
                .replace("-", " ")
                .to_uppercase();
            self.cards.push(Card {
                name: crate_info.name.clone(),
                description: display_name,
                color: Colors::LEARN,
                x: learn_x,
                y: learn_y,
                width: item_width,
                height: 40.0,
                children: vec![],
                expanded: false,
                audit: Some(CrateAudit::new(crate_info.name.clone())),
            });
            learn_y += 50.0;
        }

        // SIMULATIONS section (center bottom)
        let sims: Vec<_> = graph
            .crates
            .iter()
            .filter(|c| c.path.starts_with("SIMULATIONS/"))
            .collect();

        if !sims.is_empty() {
            let sim_y = tool_y.max(learn_y) + section_gap;

            self.cards.push(Card {
                name: "SIMULATIONS".to_string(),
                description: "Physics simulations".to_string(),
                color: Colors::PROJECT,
                x: center_x,
                y: sim_y,
                width: card_width,
                height: 50.0,
                children: sims.iter().map(|c| c.name.clone()).collect(),
                expanded: true,
                audit: None,
            });

            let sim_item_y = sim_y + 60.0;
            let sim_total_width = sims.len() as f64 * (item_width + padding) - padding;
            let mut sim_x = (self.width - sim_total_width) / 2.0;

            for crate_info in &sims {
                self.cards.push(Card {
                    name: crate_info.name.clone(),
                    description: crate_info.name.to_uppercase(),
                    color: Colors::PROJECT,
                    x: sim_x,
                    y: sim_item_y,
                    width: item_width,
                    height: 40.0,
                    children: vec![],
                    expanded: false,
                    audit: Some(CrateAudit::new(crate_info.name.clone())),
                });
                sim_x += item_width + padding;
            }
        }
    }

    fn card_at(&self, x: f64, y: f64) -> Option<String> {
        let scroll_y = y + self.scroll_y;
        for card in self.cards.iter().rev() {
            if x >= card.x
                && x <= card.x + card.width
                && scroll_y >= card.y
                && scroll_y <= card.y + card.height
            {
                return Some(card.name.clone());
            }
        }
        None
    }

    fn render(&self) {
        let ctx = &self.ctx;

        // Clear
        ctx.set_fill_style(&JsValue::from_str(Colors::BG));
        ctx.fill_rect(0.0, 0.0, self.width, self.height);

        // Apply scroll
        ctx.save();
        ctx.translate(0.0, -self.scroll_y).ok();

        // Draw cards
        for card in &self.cards {
            self.draw_card(card);
        }

        ctx.restore();

        // Draw header
        self.draw_header();
    }

    fn draw_card(&self, card: &Card) {
        let ctx = &self.ctx;
        let is_hovered = self.hovered_card.as_ref() == Some(&card.name);

        // Card background
        ctx.set_fill_style(&JsValue::from_str(Colors::CARD_BG));
        self.rounded_rect(card.x, card.y, card.width, card.height, 8.0);
        ctx.fill();

        // Border
        let border_color = if is_hovered {
            Colors::CARD_HOVER
        } else {
            card.color
        };
        ctx.set_stroke_style(&JsValue::from_str(border_color));
        ctx.set_line_width(if is_hovered { 2.0 } else { 1.0 });
        self.rounded_rect(card.x, card.y, card.width, card.height, 8.0);
        ctx.stroke();

        // Left accent bar
        ctx.set_fill_style(&JsValue::from_str(card.color));
        self.rounded_rect(card.x, card.y, 4.0, card.height, 2.0);
        ctx.fill();

        // Title
        ctx.set_fill_style(&JsValue::from_str(Colors::TEXT_PRIMARY));
        ctx.set_font("bold 14px 'JetBrains Mono', monospace");
        ctx.set_text_align("left");
        ctx.set_text_baseline("top");
        ctx.fill_text(&card.description, card.x + 16.0, card.y + 12.0)
            .ok();

        // Subtitle (name if different)
        if card.name != card.description && !card.children.is_empty() {
            ctx.set_fill_style(&JsValue::from_str(Colors::TEXT_MUTED));
            ctx.set_font("11px 'JetBrains Mono', monospace");
            ctx.fill_text(
                &format!("{} items", card.children.len()),
                card.x + 16.0,
                card.y + 32.0,
            )
            .ok();
        }
    }

    fn rounded_rect(&self, x: f64, y: f64, w: f64, h: f64, r: f64) {
        let ctx = &self.ctx;
        ctx.begin_path();
        ctx.move_to(x + r, y);
        ctx.line_to(x + w - r, y);
        ctx.arc_to(x + w, y, x + w, y + r, r).ok();
        ctx.line_to(x + w, y + h - r);
        ctx.arc_to(x + w, y + h, x + w - r, y + h, r).ok();
        ctx.line_to(x + r, y + h);
        ctx.arc_to(x, y + h, x, y + h - r, r).ok();
        ctx.line_to(x, y + r);
        ctx.arc_to(x, y, x + r, y, r).ok();
        ctx.close_path();
    }

    fn draw_header(&self) {
        let ctx = &self.ctx;

        // Header background
        ctx.set_fill_style(&JsValue::from_str("rgba(10, 10, 15, 0.95)"));
        ctx.fill_rect(0.0, 0.0, self.width, 50.0);

        // Title
        ctx.set_fill_style(&JsValue::from_str(Colors::TEXT_PRIMARY));
        ctx.set_font("bold 16px 'JetBrains Mono', monospace");
        ctx.set_text_align("left");
        ctx.set_text_baseline("middle");
        ctx.fill_text("ARCH", 20.0, 25.0).ok();

        ctx.set_fill_style(&JsValue::from_str(Colors::TEXT_SECONDARY));
        ctx.set_font("14px 'JetBrains Mono', monospace");
        ctx.fill_text("Architecture Explorer", 80.0, 25.0).ok();

        // Stats
        ctx.set_text_align("right");
        ctx.set_fill_style(&JsValue::from_str(Colors::TEXT_MUTED));
        ctx.set_font("12px 'JetBrains Mono', monospace");
        ctx.fill_text("antimony-labs monorepo", self.width - 20.0, 25.0)
            .ok();
    }
}

// ============================================================================
// WASM ENTRY
// ============================================================================

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

    let _container = document
        .get_element_by_id("canvas-container")
        .ok_or("No container")?;

    let ctx = canvas
        .get_context("2d")?
        .ok_or("No 2d context")?
        .dyn_into::<CanvasRenderingContext2d>()?;

    let state = AppState::new(canvas.clone(), ctx);
    APP.with(|app| *app.borrow_mut() = Some(state));

    render();
    setup_events(&document, &canvas)?;

    Ok(())
}

fn setup_events(_document: &web_sys::Document, canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
    // Note: _document prefixed to suppress unused warning
    // Mouse move
    let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
        let should_render = APP.with(|app| {
            if let Some(ref mut state) = *app.borrow_mut() {
                let rect = state.canvas.get_bounding_client_rect();
                let x = event.client_x() as f64 - rect.left();
                let y = event.client_y() as f64 - rect.top();
                let old_hover = state.hovered_card.clone();
                state.hovered_card = state.card_at(x, y);
                state.hovered_card != old_hover
            } else {
                false
            }
        });
        if should_render {
            render();
        }
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
    closure.forget();

    // Click
    let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
        APP.with(|app| {
            if let Some(ref mut state) = *app.borrow_mut() {
                let rect = state.canvas.get_bounding_client_rect();
                let x = event.client_x() as f64 - rect.left();
                let y = event.client_y() as f64 - rect.top();
                if let Some(card_name) = state.card_at(x, y) {
                    web_sys::console::log_1(&format!("Clicked: {}", card_name).into());
                    // Future: Open info panel or navigate
                }
            }
        });
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();

    // Wheel scroll
    let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
        event.prevent_default();
        APP.with(|app| {
            if let Some(ref mut state) = *app.borrow_mut() {
                state.scroll_y = (state.scroll_y + event.delta_y() * 0.5).max(0.0);
            }
        });
        render();
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

fn render() {
    APP.with(|app| {
        if let Some(ref state) = *app.borrow() {
            state.render();
        }
    });
}
