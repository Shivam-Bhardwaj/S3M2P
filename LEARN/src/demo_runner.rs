//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: demo_runner.rs | LEARN/src/demo_runner.rs
//! PURPOSE: Demo runner for interactive lesson simulations
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → src
//! ═══════════════════════════════════════════════════════════════════════════════

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use learn_core::demos::LinearRegressionDemo;
use learn_core::Demo;
use learn_web::{AnimationLoop, Canvas};

// Thread-local state for the currently running demo
thread_local! {
    static CURRENT_DEMO: RefCell<Option<LinearRegressionDemoRunner>> = RefCell::new(None);
}

/// Linear Regression demo runner
pub struct LinearRegressionDemoRunner {
    demo: LinearRegressionDemo,
    canvas: Canvas,
    animation: Option<Rc<AnimationLoop>>,
    paused: bool,
}

impl LinearRegressionDemoRunner {
    /// Start the Linear Regression demo
    pub fn start(canvas_id: &str, seed: u64) -> Result<(), JsValue> {
        let canvas = Canvas::new(canvas_id)?;
        let mut demo = LinearRegressionDemo::default();
        demo.reset(seed);

        let runner = LinearRegressionDemoRunner {
            demo,
            canvas,
            animation: None,
            paused: false,
        };

        CURRENT_DEMO.with(|d| {
            *d.borrow_mut() = Some(runner);
        });

        // Start animation loop
        Self::start_animation()?;

        // Wire controls
        Self::wire_controls()?;

        Ok(())
    }

    fn start_animation() -> Result<(), JsValue> {
        let animation = AnimationLoop::new(move |dt| {
            CURRENT_DEMO.with(|d| {
                if let Some(runner) = d.borrow_mut().as_mut() {
                    if !runner.paused {
                        runner.demo.step(dt as f32);
                    }
                    runner.render();
                }
            });
        });

        animation.start();

        CURRENT_DEMO.with(|d| {
            if let Some(runner) = d.borrow_mut().as_mut() {
                runner.animation = Some(Rc::new(animation));
            }
        });

        Ok(())
    }

    fn wire_controls() -> Result<(), JsValue> {
        // Learning rate slider
        if let Ok(lr_slider) = get_input("lr-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("lr-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        CURRENT_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("learning_rate", value);
                            }
                        });
                        // Update label
                        update_text("lr-value", &format!("{:.3}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            lr_slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Noise slider
        if let Ok(noise_slider) = get_input("noise-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("noise-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        CURRENT_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("noise", value);
                            }
                        });
                        update_text("noise-value", &format!("{:.2}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            noise_slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Reset button
        if let Some(btn) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("reset-btn"))
        {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                CURRENT_DEMO.with(|d| {
                    if let Some(runner) = d.borrow_mut().as_mut() {
                        // Generate new random seed
                        let seed = (js_sys::Math::random() * 1_000_000.0) as u64;
                        runner.demo.reset(seed);
                    }
                });
            }) as Box<dyn FnMut(_)>);
            btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Pause button
        if let Some(btn) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("pause-btn"))
        {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                CURRENT_DEMO.with(|d| {
                    if let Some(runner) = d.borrow_mut().as_mut() {
                        runner.paused = !runner.paused;
                        // Update button text
                        if let Some(btn) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.get_element_by_id("pause-btn"))
                        {
                            btn.set_text_content(Some(if runner.paused { "▶ Play" } else { "⏸ Pause" }));
                        }
                    }
                });
            }) as Box<dyn FnMut(_)>);
            btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Step button
        if let Some(btn) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("step-btn"))
        {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                CURRENT_DEMO.with(|d| {
                    if let Some(runner) = d.borrow_mut().as_mut() {
                        runner.demo.step(0.016);
                        runner.render();
                    }
                });
            }) as Box<dyn FnMut(_)>);
            btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }

    fn render(&mut self) {
        let ctx = self.canvas.ctx();
        let w = self.canvas.width();
        let h = self.canvas.height();

        // Clear background
        self.canvas.clear("#0a0a12");

        let margin = 40.0;
        let plot_h = h * 0.6;

        // Coordinate transform: data space [-1.5, 1.5] x [-2, 4] -> canvas
        let data_x_min = -1.5;
        let data_x_max = 1.5;
        let data_y_min = -2.0;
        let data_y_max = 4.0;

        let to_screen_x = |x: f32| -> f64 {
            margin + ((x as f64 - data_x_min) / (data_x_max - data_x_min)) * (w - 2.0 * margin)
        };

        let to_screen_y = |y: f32| -> f64 {
            margin + plot_h - ((y as f64 - data_y_min) / (data_y_max - data_y_min)) * plot_h
        };

        // Draw grid
        ctx.set_stroke_style(&JsValue::from_str("rgba(100, 255, 218, 0.1)"));
        ctx.set_line_width(1.0);

        // Vertical grid lines
        for i in 0..=6 {
            let x = margin + (i as f64 / 6.0) * (w - 2.0 * margin);
            ctx.begin_path();
            ctx.move_to(x, margin);
            ctx.line_to(x, margin + plot_h);
            ctx.stroke();
        }

        // Horizontal grid lines
        for i in 0..=6 {
            let y = margin + (i as f64 / 6.0) * plot_h;
            ctx.begin_path();
            ctx.move_to(margin, y);
            ctx.line_to(w - margin, y);
            ctx.stroke();
        }

        // Draw axes
        ctx.set_stroke_style(&JsValue::from_str("rgba(100, 255, 218, 0.3)"));
        ctx.set_line_width(2.0);

        // X-axis at y=0
        let y_zero = to_screen_y(0.0);
        ctx.begin_path();
        ctx.move_to(margin, y_zero);
        ctx.line_to(w - margin, y_zero);
        ctx.stroke();

        // Y-axis at x=0
        let x_zero = to_screen_x(0.0);
        ctx.begin_path();
        ctx.move_to(x_zero, margin);
        ctx.line_to(x_zero, margin + plot_h);
        ctx.stroke();

        // Draw target line (green, dashed)
        let (target_w, target_b) = self.demo.target();
        ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 100, 0.4)"));
        ctx.set_line_width(2.0);
        let _ = ctx.set_line_dash(&js_sys::Array::of2(&JsValue::from(5.0), &JsValue::from(5.0)));
        ctx.begin_path();
        ctx.move_to(to_screen_x(-1.5), to_screen_y(target_w * -1.5 + target_b));
        ctx.line_to(to_screen_x(1.5), to_screen_y(target_w * 1.5 + target_b));
        ctx.stroke();
        let _ = ctx.set_line_dash(&js_sys::Array::new());

        // Draw learned line (cyan, solid)
        ctx.set_stroke_style(&JsValue::from_str("#64ffda"));
        ctx.set_line_width(2.5);
        ctx.begin_path();
        ctx.move_to(to_screen_x(-1.5), to_screen_y(self.demo.w * -1.5 + self.demo.b));
        ctx.line_to(to_screen_x(1.5), to_screen_y(self.demo.w * 1.5 + self.demo.b));
        ctx.stroke();

        // Draw data points
        for p in self.demo.points() {
            self.canvas.fill_circle(to_screen_x(p.x), to_screen_y(p.y), 4.0, "#ff6b6b");
        }

        // Draw loss trace at bottom
        let loss_y = margin + plot_h + 40.0;
        let loss_h = h - loss_y - 30.0;
        let loss_history = self.demo.loss_history();

        if !loss_history.is_empty() {
            // Find max loss for scaling
            let max_loss = loss_history.iter().cloned().fold(0.1_f32, f32::max);

            ctx.set_stroke_style(&JsValue::from_str("#ff6b6b"));
            ctx.set_line_width(1.5);
            ctx.begin_path();

            let step = (w - 2.0 * margin) / loss_history.len().max(1) as f64;
            for (i, &loss) in loss_history.iter().enumerate() {
                let x = margin + i as f64 * step;
                let y = loss_y + loss_h - (loss / max_loss) as f64 * loss_h;
                if i == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.stroke();

            // Loss label
            self.canvas.text("Loss", margin, loss_y - 5.0, "#888", "12px 'Inter', sans-serif");
        }

        // Draw stats
        let step_count = self.demo.step_count();
        let loss = self.demo.compute_loss();

        ctx.set_font("13px 'JetBrains Mono', monospace");
        ctx.set_fill_style(&JsValue::from_str("#fff"));
        let _ = ctx.fill_text(&format!("Step: {}", step_count), margin, h - 10.0);
        let _ = ctx.fill_text(
            &format!("w: {:.3}  b: {:.3}  Loss: {:.4}", self.demo.w, self.demo.b, loss),
            margin + 100.0,
            h - 10.0,
        );

        // Legend
        ctx.set_font("11px 'Inter', sans-serif");

        ctx.set_fill_style(&JsValue::from_str("rgba(0, 255, 100, 0.7)"));
        let _ = ctx.fill_text("-- Target", w - margin - 60.0, margin + 15.0);

        ctx.set_fill_style(&JsValue::from_str("#64ffda"));
        let _ = ctx.fill_text("— Learned", w - margin - 60.0, margin + 30.0);
    }
}

/// Stop the current demo
pub fn stop_demo() {
    CURRENT_DEMO.with(|d| {
        if let Some(runner) = d.borrow().as_ref() {
            if let Some(animation) = &runner.animation {
                animation.stop();
            }
        }
        *d.borrow_mut() = None;
    });
}

fn get_input(id: &str) -> Result<HtmlInputElement, JsValue> {
    web_sys::window()
        .ok_or("No window")?
        .document()
        .ok_or("No document")?
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", id)))?
        .dyn_into::<HtmlInputElement>()
        .map_err(|_| JsValue::from_str("Not an input element"))
}

fn update_text(id: &str, text: &str) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    {
        el.set_text_content(Some(text));
    }
}
