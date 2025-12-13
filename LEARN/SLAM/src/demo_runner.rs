//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: demo_runner.rs | SLAM/src/demo_runner.rs
//! PURPOSE: Particle Filter demo runner and visualization
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → SLAM
//! ═══════════════════════════════════════════════════════════════════════════════

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use learn_core::demos::ParticleFilterDemo;
use learn_core::Demo;
use learn_web::{AnimationLoop, Canvas};

// Thread-local state for the currently running demo
thread_local! {
    static CURRENT_DEMO: RefCell<Option<ParticleFilterDemoRunner>> = RefCell::new(None);
}

/// Particle Filter demo runner
pub struct ParticleFilterDemoRunner {
    demo: ParticleFilterDemo,
    canvas: Canvas,
    animation: Option<Rc<AnimationLoop>>,
    paused: bool,
}

impl ParticleFilterDemoRunner {
    /// Start the Particle Filter demo
    pub fn start(canvas_id: &str, seed: u64) -> Result<(), JsValue> {
        let canvas = Canvas::new(canvas_id)?;
        let mut demo = ParticleFilterDemo::default();
        demo.reset(seed);

        let runner = ParticleFilterDemoRunner {
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
        // Particles slider
        if let Ok(slider) = get_input("particles-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("particles-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        CURRENT_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("num_particles", value);
                            }
                        });
                        update_text("particles-value", &format!("{}", value as i32));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Motion noise slider
        if let Ok(slider) = get_input("motion-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("motion-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        CURRENT_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("motion_noise", value);
                            }
                        });
                        update_text("motion-value", &format!("{:.2}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Sensor noise slider
        if let Ok(slider) = get_input("sensor-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("sensor-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        CURRENT_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("sensor_noise", value);
                            }
                        });
                        update_text("sensor-value", &format!("{:.2}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
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

        Ok(())
    }

    fn render(&mut self) {
        let ctx = self.canvas.ctx();
        let w = self.canvas.width();
        let h = self.canvas.height();

        // Clear background
        self.canvas.clear("#0a0a12");

        let margin = 30.0;
        let plot_size = (w - 2.0 * margin).min(h - 2.0 * margin);
        let offset_x = (w - plot_size) / 2.0;
        let offset_y = (h - plot_size) / 2.0;

        // Coordinate transform: [0, 1] -> canvas
        let to_x = |x: f32| -> f64 { offset_x + (x as f64) * plot_size };
        let to_y = |y: f32| -> f64 { offset_y + (1.0 - y as f64) * plot_size };

        // Draw border
        self.canvas.stroke_rect(offset_x, offset_y, plot_size, plot_size, "rgba(100, 255, 218, 0.3)", 1.0);

        // Draw grid
        ctx.set_stroke_style(&JsValue::from_str("rgba(100, 255, 218, 0.1)"));
        ctx.set_line_width(1.0);
        for i in 1..5 {
            let pos = i as f64 / 5.0;
            ctx.begin_path();
            ctx.move_to(offset_x + pos * plot_size, offset_y);
            ctx.line_to(offset_x + pos * plot_size, offset_y + plot_size);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(offset_x, offset_y + pos * plot_size);
            ctx.line_to(offset_x + plot_size, offset_y + pos * plot_size);
            ctx.stroke();
        }

        // Draw landmarks as blue squares
        for lm in &self.demo.landmarks {
            self.canvas.fill_rect(to_x(lm.x) - 6.0, to_y(lm.y) - 6.0, 12.0, 12.0, "#4488ff");
        }

        // Draw particles with alpha based on weight
        let max_weight = self.demo.particles.iter().map(|p| p.weight).fold(0.0_f32, f32::max);
        for particle in &self.demo.particles {
            let alpha = if max_weight > 0.0 {
                (particle.weight / max_weight).sqrt().min(1.0)
            } else {
                0.3
            };
            let color = format!("rgba(255, 150, 100, {:.2})", alpha * 0.6 + 0.1);
            self.canvas.fill_circle(to_x(particle.pos.x), to_y(particle.pos.y), 3.0, &color);
        }

        // Draw estimated pose (cyan triangle)
        self.canvas.fill_triangle(
            to_x(self.demo.est_pos.x),
            to_y(self.demo.est_pos.y),
            12.0,
            -self.demo.est_theta as f64,
            "#00ffff",
        );

        // Draw true robot pose (green triangle)
        self.canvas.fill_triangle(
            to_x(self.demo.true_pos.x),
            to_y(self.demo.true_pos.y),
            15.0,
            -self.demo.true_theta as f64,
            "#00ff88",
        );

        // Draw sensor rays from true pose to landmarks (dim)
        ctx.set_stroke_style(&JsValue::from_str("rgba(255, 255, 100, 0.15)"));
        ctx.set_line_width(1.0);
        for lm in &self.demo.landmarks {
            ctx.begin_path();
            ctx.move_to(to_x(self.demo.true_pos.x), to_y(self.demo.true_pos.y));
            ctx.line_to(to_x(lm.x), to_y(lm.y));
            ctx.stroke();
        }

        // Draw legend
        ctx.set_font("12px 'Inter', sans-serif");

        ctx.set_fill_style(&JsValue::from_str("#00ff88"));
        let _ = ctx.fill_text("▲ True Pose", w - margin - 90.0, margin + 15.0);

        ctx.set_fill_style(&JsValue::from_str("#00ffff"));
        let _ = ctx.fill_text("▲ Estimated", w - margin - 90.0, margin + 32.0);

        ctx.set_fill_style(&JsValue::from_str("#ff9664"));
        let _ = ctx.fill_text("● Particles", w - margin - 90.0, margin + 49.0);

        ctx.set_fill_style(&JsValue::from_str("#4488ff"));
        let _ = ctx.fill_text("■ Landmarks", w - margin - 90.0, margin + 66.0);

        // Draw error stats
        let error = self.demo.error();
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text(
            &format!("Error: {:.3}", error),
            margin,
            h - 10.0,
        );
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
