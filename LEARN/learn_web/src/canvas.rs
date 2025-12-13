//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: canvas.rs | LEARN/learn_web/src/canvas.rs
//! PURPOSE: Hi-DPI canvas wrapper for sharp rendering on retina displays
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_web
//! ═══════════════════════════════════════════════════════════════════════════════

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Hi-DPI aware canvas wrapper
///
/// Automatically handles device pixel ratio scaling for sharp rendering
/// on retina displays.
#[derive(Clone)]
pub struct Canvas {
    element: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    dpr: f64,
    logical_width: u32,
    logical_height: u32,
}

impl Canvas {
    /// Create a canvas wrapper from an element ID
    pub fn new(id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;

        let element = document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("Canvas '{}' not found", id)))?
            .dyn_into::<HtmlCanvasElement>()?;

        let ctx = element
            .get_context("2d")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let dpr = window.device_pixel_ratio();

        let mut canvas = Self {
            element,
            ctx,
            dpr,
            logical_width: 0,
            logical_height: 0,
        };

        canvas.resize();
        Ok(canvas)
    }

    /// Resize canvas to match current element dimensions with Hi-DPI scaling
    pub fn resize(&mut self) {
        let w = self.element.client_width() as u32;
        let h = self.element.client_height() as u32;

        if w == 0 || h == 0 {
            return;
        }

        self.logical_width = w;
        self.logical_height = h;

        // Set physical resolution for Hi-DPI
        let physical_w = (w as f64 * self.dpr) as u32;
        let physical_h = (h as f64 * self.dpr) as u32;

        self.element.set_width(physical_w);
        self.element.set_height(physical_h);

        // Scale context so drawing operations use logical coordinates
        let _ = self.ctx.scale(self.dpr, self.dpr);
    }

    /// Get logical width (CSS pixels)
    #[inline]
    pub fn width(&self) -> f64 {
        self.logical_width as f64
    }

    /// Get logical height (CSS pixels)
    #[inline]
    pub fn height(&self) -> f64 {
        self.logical_height as f64
    }

    /// Get the 2D rendering context
    #[inline]
    pub fn ctx(&self) -> &CanvasRenderingContext2d {
        &self.ctx
    }

    /// Get the underlying canvas element
    #[inline]
    pub fn element(&self) -> &HtmlCanvasElement {
        &self.element
    }

    /// Get device pixel ratio
    #[inline]
    pub fn dpr(&self) -> f64 {
        self.dpr
    }

    /// Clear the canvas with a color
    pub fn clear(&self, color: &str) {
        self.ctx.set_fill_style(&JsValue::from_str(color));
        self.ctx.fill_rect(0.0, 0.0, self.width(), self.height());
    }

    /// Draw a filled circle
    pub fn fill_circle(&self, x: f64, y: f64, radius: f64, color: &str) {
        self.ctx.set_fill_style(&JsValue::from_str(color));
        self.ctx.begin_path();
        let _ = self
            .ctx
            .arc(x, y, radius, 0.0, std::f64::consts::TAU);
        self.ctx.fill();
    }

    /// Draw a stroked circle
    pub fn stroke_circle(&self, x: f64, y: f64, radius: f64, color: &str, line_width: f64) {
        self.ctx.set_stroke_style(&JsValue::from_str(color));
        self.ctx.set_line_width(line_width);
        self.ctx.begin_path();
        let _ = self
            .ctx
            .arc(x, y, radius, 0.0, std::f64::consts::TAU);
        self.ctx.stroke();
    }

    /// Draw a line
    pub fn line(&self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, line_width: f64) {
        self.ctx.set_stroke_style(&JsValue::from_str(color));
        self.ctx.set_line_width(line_width);
        self.ctx.begin_path();
        self.ctx.move_to(x1, y1);
        self.ctx.line_to(x2, y2);
        self.ctx.stroke();
    }

    /// Draw text
    pub fn text(&self, text: &str, x: f64, y: f64, color: &str, font: &str) {
        self.ctx.set_fill_style(&JsValue::from_str(color));
        self.ctx.set_font(font);
        let _ = self.ctx.fill_text(text, x, y);
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&self, x: f64, y: f64, w: f64, h: f64, color: &str) {
        self.ctx.set_fill_style(&JsValue::from_str(color));
        self.ctx.fill_rect(x, y, w, h);
    }

    /// Draw a stroked rectangle
    pub fn stroke_rect(&self, x: f64, y: f64, w: f64, h: f64, color: &str, line_width: f64) {
        self.ctx.set_stroke_style(&JsValue::from_str(color));
        self.ctx.set_line_width(line_width);
        self.ctx.stroke_rect(x, y, w, h);
    }

    /// Set the global alpha (transparency)
    pub fn set_alpha(&self, alpha: f64) {
        self.ctx.set_global_alpha(alpha);
    }

    /// Reset global alpha to 1.0
    pub fn reset_alpha(&self) {
        self.ctx.set_global_alpha(1.0);
    }

    /// Save context state
    pub fn save(&self) {
        self.ctx.save();
    }

    /// Restore context state
    pub fn restore(&self) {
        self.ctx.restore();
    }

    /// Translate the context origin
    pub fn translate(&self, x: f64, y: f64) {
        let _ = self.ctx.translate(x, y);
    }

    /// Rotate the context
    pub fn rotate(&self, angle: f64) {
        let _ = self.ctx.rotate(angle);
    }

    /// Draw a triangle (useful for robot pose visualization)
    pub fn fill_triangle(&self, x: f64, y: f64, size: f64, angle: f64, color: &str) {
        self.ctx.save();
        let _ = self.ctx.translate(x, y);
        let _ = self.ctx.rotate(angle);

        self.ctx.set_fill_style(&JsValue::from_str(color));
        self.ctx.begin_path();
        self.ctx.move_to(size, 0.0);
        self.ctx.line_to(-size * 0.5, size * 0.5);
        self.ctx.line_to(-size * 0.5, -size * 0.5);
        self.ctx.close_path();
        self.ctx.fill();

        self.ctx.restore();
    }
}
