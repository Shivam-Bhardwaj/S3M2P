use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub struct BackgroundEffect {
    time: f64,
    width: f64,
    height: f64,
}

impl BackgroundEffect {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            time: 0.0,
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }

    pub fn update(&mut self, dt: f64) {
        self.time += dt;
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d) {
        let t = self.time;

        // 1. Darker Tech Background
        ctx.set_fill_style(&JsValue::from_str("#050508"));
        ctx.fill_rect(0.0, 0.0, self.width, self.height);

        // 2. Moving Grid Lines (Perspective Floor)
        // A rolling grid in 3D-ish perspective or just a flat scrolling grid

        let grid_size = 60.0;
        let scroll_y = (t * 20.0) % grid_size;
        let scroll_x = (t * 10.0) % grid_size;

        ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 255, 0.03)")); // Very faint cyan
        ctx.set_line_width(1.0);

        // Vertical lines
        let start_x = -scroll_x;
        let mut x = start_x;
        while x < self.width {
            ctx.begin_path();
            ctx.move_to(x, 0.0);
            ctx.line_to(x, self.height);
            ctx.stroke();
            x += grid_size;
        }

        // Horizontal lines
        let start_y = -scroll_y;
        let mut y = start_y;
        while y < self.height {
            ctx.begin_path();
            ctx.move_to(0.0, y);
            ctx.line_to(self.width, y);
            ctx.stroke();
            y += grid_size;
        }

        // 3. Digital Rain / Matrix Data Stream drops
        // Random vertical streaks
        // Use a pseudo-random number generator based on time and position

        ctx.set_fill_style(&JsValue::from_str("rgba(0, 255, 100, 0.05)"));

        let columns = (self.width / 20.0) as usize;
        for i in 0..columns {
            // Pseudo-random offset for each column
            let seed = i as f64 * 13.37;
            let speed = 50.0 + (seed.sin() * 25.0).abs(); // Varying speeds
            let offset = t * speed;
            let y_pos = (offset + seed * 100.0) % (self.height + 200.0) - 100.0;

            // Only draw some columns
            if seed.cos() > 0.5 {
                let x = i as f64 * 20.0;
                ctx.fill_rect(x, y_pos, 2.0, 40.0 + (seed.sin() * 20.0));
            }
        }

        // 4. Vignette (Dark corners)
        // Use a radial gradient to darken edges
        let center_x = self.width / 2.0;
        let center_y = self.height / 2.0;
        let radius = self.width.max(self.height) * 0.8;

        let grad = ctx
            .create_radial_gradient(center_x, center_y, radius * 0.5, center_x, center_y, radius)
            .unwrap();
        grad.add_color_stop(0.0, "transparent").unwrap();
        grad.add_color_stop(1.0, "rgba(0, 0, 0, 0.6)").unwrap();

        ctx.set_fill_style(&grad);
        ctx.fill_rect(0.0, 0.0, self.width, self.height);

        // 5. Scanline Overlay
        ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.2)"));
        for i in (0..self.height as i32).step_by(3) {
            ctx.fill_rect(0.0, i as f64, self.width, 1.0);
        }
    }
}
