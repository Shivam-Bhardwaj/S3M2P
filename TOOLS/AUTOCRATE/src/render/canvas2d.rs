//! 2D Canvas renderer for technical drawings

use glam::Vec3;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// 2D orthographic view direction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewDirection {
    Front,  // Looking along -Y
    Top,    // Looking along -Z
    Side,   // Looking along +X
}

/// 2D technical drawing renderer
pub struct Canvas2DRenderer {
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    view: ViewDirection,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl Canvas2DRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|e| format!("Failed to get 2D context: {:?}", e))?
            .ok_or("2D context is None")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|e| format!("Failed to cast to 2D context: {:?}", e))?;

        Ok(Self {
            ctx,
            canvas,
            view: ViewDirection::Front,
            scale: 2.0,
            offset_x: 0.0,
            offset_y: 0.0,
        })
    }

    /// Set view direction
    pub fn set_view(&mut self, view: ViewDirection) {
        self.view = view;
    }

    /// Clear canvas with dark background
    pub fn clear(&self) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // too.foo dark theme
        self.ctx.set_fill_style(&"#0a0a0f".into());
        self.ctx.fill_rect(0.0, 0.0, width, height);
    }

    /// Project 3D point to 2D screen space
    fn project(&self, point: Vec3) -> (f64, f64) {
        let (x, y) = match self.view {
            ViewDirection::Front => (point.x, point.z),
            ViewDirection::Top => (point.x, point.y),
            ViewDirection::Side => (point.y, point.z),
        };

        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Center and scale
        let screen_x = width / 2.0 + (x as f64 + self.offset_x as f64) * self.scale as f64;
        let screen_y = height / 2.0 - (y as f64 + self.offset_y as f64) * self.scale as f64;

        (screen_x, screen_y)
    }

    /// Draw a line between two 3D points
    pub fn draw_line(&self, p1: Vec3, p2: Vec3, color: &str, width: f64) {
        let (x1, y1) = self.project(p1);
        let (x2, y2) = self.project(p2);

        self.ctx.set_stroke_style(&color.into());
        self.ctx.set_line_width(width);
        self.ctx.begin_path();
        self.ctx.move_to(x1, y1);
        self.ctx.line_to(x2, y2);
        self.ctx.stroke();
    }

    /// Draw a box outline
    pub fn draw_box_outline(&self, min: Vec3, max: Vec3, color: &str) {
        let vertices = [
            Vec3::new(min.x, min.y, min.z), // 0
            Vec3::new(max.x, min.y, min.z), // 1
            Vec3::new(max.x, max.y, min.z), // 2
            Vec3::new(min.x, max.y, min.z), // 3
            Vec3::new(min.x, min.y, max.z), // 4
            Vec3::new(max.x, min.y, max.z), // 5
            Vec3::new(max.x, max.y, max.z), // 6
            Vec3::new(min.x, max.y, max.z), // 7
        ];

        // Draw visible edges based on view
        let edges: Vec<(usize, usize)> = match self.view {
            ViewDirection::Front => vec![
                (0, 1), (1, 2), (2, 3), (3, 0), // Back face
                (4, 5), (5, 6), (6, 7), (7, 4), // Front face
                (0, 4), (1, 5), (2, 6), (3, 7), // Connecting edges
            ],
            ViewDirection::Top => vec![
                (0, 1), (1, 5), (5, 4), (4, 0), // Bottom
                (3, 2), (2, 6), (6, 7), (7, 3), // Top
                (0, 3), (1, 2), (4, 7), (5, 6), // Sides
            ],
            ViewDirection::Side => vec![
                (0, 3), (3, 7), (7, 4), (4, 0), // Left
                (1, 2), (2, 6), (6, 5), (5, 1), // Right
                (0, 1), (3, 2), (4, 5), (7, 6), // Connecting
            ],
        };

        for (i, j) in edges {
            self.draw_line(vertices[i], vertices[j], color, 1.0);
        }
    }

    /// Draw dimension line with arrows and text
    pub fn draw_dimension(
        &self,
        p1: Vec3,
        p2: Vec3,
        offset: f32,
        label: &str,
    ) {
        let (x1, y1) = self.project(p1);
        let (x2, y2) = self.project(p2);

        // Dimension line offset perpendicular to the line
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        let nx = -dy / len * offset as f64;
        let ny = dx / len * offset as f64;

        let dim_x1 = x1 + nx;
        let dim_y1 = y1 + ny;
        let dim_x2 = x2 + nx;
        let dim_y2 = y2 + ny;

        // Draw dimension line
        self.ctx.set_stroke_style(&"#3498db".into()); // Accent color
        self.ctx.set_line_width(1.0);
        self.ctx.begin_path();
        self.ctx.move_to(dim_x1, dim_y1);
        self.ctx.line_to(dim_x2, dim_y2);
        self.ctx.stroke();

        // Draw extension lines
        self.ctx.begin_path();
        self.ctx.move_to(x1, y1);
        self.ctx.line_to(dim_x1, dim_y1);
        self.ctx.move_to(x2, y2);
        self.ctx.line_to(dim_x2, dim_y2);
        self.ctx.stroke();

        // Draw arrows (simple triangles)
        let arrow_size = 5.0;
        let angle = dy.atan2(dx);

        // Arrow at start
        self.ctx.begin_path();
        self.ctx.move_to(dim_x1, dim_y1);
        self.ctx.line_to(
            dim_x1 + arrow_size * (angle + 2.5).cos(),
            dim_y1 + arrow_size * (angle + 2.5).sin(),
        );
        self.ctx.line_to(
            dim_x1 + arrow_size * (angle - 2.5).cos(),
            dim_y1 + arrow_size * (angle - 2.5).sin(),
        );
        self.ctx.close_path();
        self.ctx.fill();

        // Arrow at end
        self.ctx.begin_path();
        self.ctx.move_to(dim_x2, dim_y2);
        self.ctx.line_to(
            dim_x2 - arrow_size * (angle + 2.5).cos(),
            dim_y2 - arrow_size * (angle + 2.5).sin(),
        );
        self.ctx.line_to(
            dim_x2 - arrow_size * (angle - 2.5).cos(),
            dim_y2 - arrow_size * (angle - 2.5).sin(),
        );
        self.ctx.close_path();
        self.ctx.fill();

        // Draw label
        let mid_x = (dim_x1 + dim_x2) / 2.0;
        let mid_y = (dim_y1 + dim_y2) / 2.0;

        self.ctx.set_fill_style(&"#ecf0f1".into()); // Light text
        self.ctx.set_font("12px JustSans, sans-serif");
        self.ctx.set_text_align("center");
        self.ctx.set_text_baseline("middle");
        let _ = self.ctx.fill_text(label, mid_x, mid_y - 10.0);
    }

    /// Draw text annotation
    pub fn draw_text(&self, pos: Vec3, text: &str, color: &str) {
        let (x, y) = self.project(pos);

        self.ctx.set_fill_style(&color.into());
        self.ctx.set_font("11px JustSans, monospace");
        self.ctx.set_text_align("left");
        let _ = self.ctx.fill_text(text, x + 5.0, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_direction() {
        assert_eq!(ViewDirection::Front, ViewDirection::Front);
        assert_ne!(ViewDirection::Front, ViewDirection::Top);
    }
}
