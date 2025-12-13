//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: demo_runner.rs | SLAM/src/demo_runner.rs
//! PURPOSE: Demo runners for all SLAM lessons
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → SLAM
//! ═══════════════════════════════════════════════════════════════════════════════

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use learn_core::demos::{
    ComplementaryFilterDemo,
    ParticleFilterDemo, PFPhase,
    KalmanFilterDemo, KFPhase,
    EkfSlamDemo,
    GraphSlamDemo,
};
use learn_core::Demo;
use learn_web::{AnimationLoop, Canvas};

// Thread-local state for the currently running demos
thread_local! {
    static COMPLEMENTARY_FILTER_DEMO: RefCell<Option<ComplementaryFilterDemoRunner>> = RefCell::new(None);
    static PARTICLE_FILTER_DEMO: RefCell<Option<ParticleFilterDemoRunner>> = RefCell::new(None);
    static KALMAN_FILTER_DEMO: RefCell<Option<KalmanFilterDemoRunner>> = RefCell::new(None);
    static EKF_SLAM_DEMO: RefCell<Option<EkfSlamDemoRunner>> = RefCell::new(None);
    static GRAPH_SLAM_DEMO: RefCell<Option<GraphSlamDemoRunner>> = RefCell::new(None);
}

/// Dispatch to the appropriate demo based on lesson index
/// Order: 0=Complementary, 1=Kalman, 2=Particle, 3=EKF, 4=Graph
pub fn start_demo_for_lesson(lesson_idx: usize, canvas_id: &str, seed: u64) -> Result<(), JsValue> {
    match lesson_idx {
        0 => ComplementaryFilterDemoRunner::start(canvas_id, seed),
        1 => KalmanFilterDemoRunner::start(canvas_id, seed),
        2 => ParticleFilterDemoRunner::start(canvas_id, seed),
        3 => EkfSlamDemoRunner::start(canvas_id, seed),
        4 => GraphSlamDemoRunner::start(canvas_id, seed),
        _ => Ok(()),
    }
}

/// Particle Filter demo runner
pub struct ParticleFilterDemoRunner {
    demo: ParticleFilterDemo,
    canvas: Canvas,
    animation: Option<Rc<AnimationLoop>>,
    paused: bool,
    step_mode: bool,
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
            step_mode: false,
        };

        PARTICLE_FILTER_DEMO.with(|d| {
            *d.borrow_mut() = Some(runner);
        });

        // Start animation loop
        Self::start_animation()?;

        // Wire controls
        Self::wire_controls()?;

        Ok(())
    }

    /// Advance one step in step mode
    pub fn step_once() {
        PARTICLE_FILTER_DEMO.with(|d| {
            if let Some(runner) = d.borrow_mut().as_mut() {
                if runner.step_mode {
                    runner.demo.next_phase(0.1); // Fixed dt for step mode
                }
            }
        });
    }

    fn start_animation() -> Result<(), JsValue> {
        let animation = AnimationLoop::new(move |dt| {
            PARTICLE_FILTER_DEMO.with(|d| {
                if let Some(runner) = d.borrow_mut().as_mut() {
                    if !runner.paused {
                        runner.demo.step(dt as f32);
                    }
                    runner.render();
                }
            });
        });

        animation.start();

        PARTICLE_FILTER_DEMO.with(|d| {
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
                        PARTICLE_FILTER_DEMO.with(|d| {
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
                        PARTICLE_FILTER_DEMO.with(|d| {
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
                        PARTICLE_FILTER_DEMO.with(|d| {
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
                PARTICLE_FILTER_DEMO.with(|d| {
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
                PARTICLE_FILTER_DEMO.with(|d| {
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

        // Step mode toggle
        if let Some(btn) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("step-mode-btn"))
        {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                PARTICLE_FILTER_DEMO.with(|d| {
                    if let Some(runner) = d.borrow_mut().as_mut() {
                        runner.step_mode = !runner.step_mode;
                        runner.demo.set_param("step_mode", if runner.step_mode { 1.0 } else { 0.0 });
                        if let Some(btn) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.get_element_by_id("step-mode-btn"))
                        {
                            btn.set_text_content(Some(if runner.step_mode { "🔄 Continuous" } else { "👣 Step Mode" }));
                        }
                        // Show/hide step button
                        if let Some(step_btn) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.get_element_by_id("step-btn"))
                        {
                            let _ = step_btn.set_attribute("style", if runner.step_mode { "display: inline-block" } else { "display: none" });
                        }
                    }
                });
            }) as Box<dyn FnMut(_)>);
            btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Step button (advance one phase)
        if let Some(btn) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("step-btn"))
        {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                ParticleFilterDemoRunner::step_once();
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

        // Layout: main plot area on left, info panel on right
        let info_width = 160.0;
        let margin = 20.0;
        let plot_size = ((w - info_width - 2.0 * margin).min(h - 2.0 * margin - 40.0)).max(200.0);
        let offset_x = margin;
        let offset_y = margin + 30.0; // Leave room for phase indicator at top

        // Coordinate transform: [0, 1] -> canvas
        let to_x = |x: f32| -> f64 { offset_x + (x as f64) * plot_size };
        let to_y = |y: f32| -> f64 { offset_y + (1.0 - y as f64) * plot_size };

        // === PHASE INDICATOR (top) ===
        let phase_color = match self.demo.phase {
            PFPhase::Predict => "#ffaa00",
            PFPhase::Update => "#00aaff",
            PFPhase::Resample => "#ff55aa",
            PFPhase::Estimate => "#00ff88",
        };

        ctx.set_font("bold 14px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str(phase_color));
        let phase_text = if self.step_mode {
            format!("STEP MODE: {}", self.demo.phase.name())
        } else {
            "CONTINUOUS MODE".to_string()
        };
        let _ = ctx.fill_text(&phase_text, margin, 20.0);

        ctx.set_font("12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text(self.demo.phase.description(), margin + 180.0, 20.0);

        // === MAIN PLOT AREA ===
        // Draw border
        self.canvas.stroke_rect(offset_x, offset_y, plot_size, plot_size, "rgba(100, 255, 218, 0.3)", 1.0);

        // Draw grid
        ctx.set_stroke_style(&JsValue::from_str("rgba(100, 255, 218, 0.08)"));
        ctx.set_line_width(1.0);
        for i in 1..10 {
            let pos = i as f64 / 10.0;
            ctx.begin_path();
            ctx.move_to(offset_x + pos * plot_size, offset_y);
            ctx.line_to(offset_x + pos * plot_size, offset_y + plot_size);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(offset_x, offset_y + pos * plot_size);
            ctx.line_to(offset_x + plot_size, offset_y + pos * plot_size);
            ctx.stroke();
        }

        // Draw sensor rays from true pose to landmarks
        // Color intensity based on measurement quality
        for meas in &self.demo.measurements {
            let lm = self.demo.landmarks[meas.landmark_idx];
            let error = (meas.noisy_range - meas.range).abs();
            let alpha = (1.0 - error / 0.1).max(0.1).min(0.5);

            ctx.set_stroke_style(&JsValue::from_str(&format!("rgba(255, 255, 100, {:.2})", alpha)));
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(to_x(self.demo.true_pos.x), to_y(self.demo.true_pos.y));
            ctx.line_to(to_x(lm.x), to_y(lm.y));
            ctx.stroke();
        }

        // Draw landmarks as blue squares with labels
        ctx.set_font("10px 'Inter', sans-serif");
        for (i, lm) in self.demo.landmarks.iter().enumerate() {
            // Draw square
            self.canvas.fill_rect(to_x(lm.x) - 5.0, to_y(lm.y) - 5.0, 10.0, 10.0, "#4488ff");
            // Draw outline
            self.canvas.stroke_rect(to_x(lm.x) - 5.0, to_y(lm.y) - 5.0, 10.0, 10.0, "#88bbff", 1.0);
            // Label
            ctx.set_fill_style(&JsValue::from_str("#4488ff"));
            let _ = ctx.fill_text(&format!("L{}", i), to_x(lm.x) + 8.0, to_y(lm.y) + 4.0);
        }

        // Draw particles with color/size based on weight
        let max_weight = self.demo.particles.iter()
            .map(|p| p.weight)
            .fold(0.0_f32, f32::max);

        for particle in &self.demo.particles {
            let norm_weight = if max_weight > 0.0 {
                (particle.weight / max_weight).sqrt()
            } else {
                0.3
            };

            // Color: low weight = red/dim, high weight = orange/bright
            let r = 255;
            let g = (100.0 + 155.0 * norm_weight) as u8;
            let b = (50.0 + 100.0 * norm_weight) as u8;
            let alpha = 0.3 + 0.7 * norm_weight;
            let color = format!("rgba({}, {}, {}, {:.2})", r, g, b, alpha);

            // Size: 2-5 pixels based on weight
            let size = 2.0 + 3.0 * norm_weight as f64;

            self.canvas.fill_circle(to_x(particle.pos.x), to_y(particle.pos.y), size, &color);

            // Draw heading indicator for high-weight particles
            if norm_weight > 0.5 {
                let len = 8.0;
                let dx = len * (particle.theta as f64).cos();
                let dy = -len * (particle.theta as f64).sin(); // flip Y
                ctx.set_stroke_style(&JsValue::from_str(&format!("rgba(255, 200, 100, {:.2})", alpha * 0.5)));
                ctx.set_line_width(1.0);
                ctx.begin_path();
                ctx.move_to(to_x(particle.pos.x), to_y(particle.pos.y));
                ctx.line_to(to_x(particle.pos.x) + dx, to_y(particle.pos.y) + dy);
                ctx.stroke();
            }
        }

        // Draw estimated pose (cyan triangle with uncertainty)
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
            14.0,
            -self.demo.true_theta as f64,
            "#00ff88",
        );

        // === INFO PANEL (right side) ===
        let panel_x = offset_x + plot_size + 20.0;
        let mut panel_y = offset_y;

        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("LEGEND", panel_x, panel_y);
        panel_y += 20.0;

        ctx.set_font("11px 'Inter', sans-serif");

        // Legend items
        self.canvas.fill_triangle(panel_x + 8.0, panel_y, 8.0, 0.0, "#00ff88");
        ctx.set_fill_style(&JsValue::from_str("#00ff88"));
        let _ = ctx.fill_text("True Robot", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_triangle(panel_x + 8.0, panel_y, 8.0, 0.0, "#00ffff");
        ctx.set_fill_style(&JsValue::from_str("#00ffff"));
        let _ = ctx.fill_text("Estimated", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_circle(panel_x + 8.0, panel_y, 4.0, "#ff9664");
        ctx.set_fill_style(&JsValue::from_str("#ff9664"));
        let _ = ctx.fill_text("Particles", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_rect(panel_x + 4.0, panel_y - 4.0, 8.0, 8.0, "#4488ff");
        ctx.set_fill_style(&JsValue::from_str("#4488ff"));
        let _ = ctx.fill_text("Landmarks", panel_x + 22.0, panel_y + 4.0);
        panel_y += 30.0;

        // Statistics
        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("STATISTICS", panel_x, panel_y);
        panel_y += 18.0;

        ctx.set_font("11px 'Inter', sans-serif");
        let error = self.demo.error();
        let error_color = if error < 0.05 { "#00ff88" } else if error < 0.1 { "#ffaa00" } else { "#ff5555" };
        ctx.set_fill_style(&JsValue::from_str(error_color));
        let _ = ctx.fill_text(&format!("Error: {:.3}", error), panel_x, panel_y);
        panel_y += 16.0;

        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text(&format!("N_eff: {:.0}/{}", self.demo.effective_particles, self.demo.particles.len()), panel_x, panel_y);
        panel_y += 16.0;

        let _ = ctx.fill_text(&format!("Max w: {:.4}", self.demo.best_particle_weight), panel_x, panel_y);
        panel_y += 30.0;

        // Error history mini-plot
        if !self.demo.error_history.is_empty() {
            ctx.set_font("bold 12px 'Inter', sans-serif");
            ctx.set_fill_style(&JsValue::from_str("#aaa"));
            let _ = ctx.fill_text("ERROR HISTORY", panel_x, panel_y);
            panel_y += 10.0;

            let plot_w = 120.0;
            let plot_h = 50.0;

            // Background
            self.canvas.fill_rect(panel_x, panel_y, plot_w, plot_h, "rgba(30, 30, 40, 0.8)");
            self.canvas.stroke_rect(panel_x, panel_y, plot_w, plot_h, "#333", 1.0);

            // Draw error line
            let history = &self.demo.error_history;
            let max_error = history.iter().copied().fold(0.0_f32, f32::max).max(0.1);

            ctx.set_stroke_style(&JsValue::from_str("#ff9664"));
            ctx.set_line_width(1.5);
            ctx.begin_path();

            for (i, &err) in history.iter().enumerate() {
                let x = panel_x + (i as f64 / history.len() as f64) * plot_w;
                let y = panel_y + plot_h - (err as f64 / max_error as f64) * plot_h;
                if i == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.stroke();

            // Draw threshold line
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 136, 0.3)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            let threshold_y = panel_y + plot_h - (0.05_f64 / max_error as f64) * plot_h;
            ctx.move_to(panel_x, threshold_y);
            ctx.line_to(panel_x + plot_w, threshold_y);
            ctx.stroke();
        }

        // === ALGORITHM EXPLANATION (bottom) ===
        let explain_y = offset_y + plot_size + 15.0;
        ctx.set_font("11px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#666"));

        let explanation = match self.demo.phase {
            PFPhase::Predict => "PREDICT: Each particle moves according to the robot's motion command, plus random noise. This spreads particles to represent motion uncertainty.",
            PFPhase::Update => "UPDATE: Particles are weighted by how well their predicted sensor readings match actual measurements. Particles near the true position get higher weights.",
            PFPhase::Resample => "RESAMPLE: Low-weight particles are removed and high-weight particles are duplicated. This focuses particles on likely positions.",
            PFPhase::Estimate => "ESTIMATE: The robot's position is estimated as the weighted average of all particles. The cyan triangle shows this estimate.",
        };

        // Word wrap the explanation
        let max_chars = 100;
        let words: Vec<&str> = explanation.split_whitespace().collect();
        let mut line = String::new();
        let mut y = explain_y;

        for word in words {
            if line.len() + word.len() + 1 > max_chars {
                let _ = ctx.fill_text(&line, margin, y);
                y += 14.0;
                line = word.to_string();
            } else {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
        }
        if !line.is_empty() {
            let _ = ctx.fill_text(&line, margin, y);
        }
    }
}

/// Stop the current demo
pub fn stop_demo() {
    // Stop complementary filter demo
    COMPLEMENTARY_FILTER_DEMO.with(|d| {
        if let Some(runner) = d.borrow().as_ref() {
            if let Some(animation) = &runner.animation {
                animation.stop();
            }
        }
        *d.borrow_mut() = None;
    });

    // Stop particle filter demo
    PARTICLE_FILTER_DEMO.with(|d| {
        if let Some(runner) = d.borrow().as_ref() {
            if let Some(animation) = &runner.animation {
                animation.stop();
            }
        }
        *d.borrow_mut() = None;
    });

    // Stop Kalman filter demo
    KALMAN_FILTER_DEMO.with(|d| {
        if let Some(runner) = d.borrow().as_ref() {
            if let Some(animation) = &runner.animation {
                animation.stop();
            }
        }
        *d.borrow_mut() = None;
    });

    // Stop EKF SLAM demo
    EKF_SLAM_DEMO.with(|d| {
        if let Some(runner) = d.borrow().as_ref() {
            if let Some(animation) = &runner.animation {
                animation.stop();
            }
        }
        *d.borrow_mut() = None;
    });

    // Stop Graph SLAM demo
    GRAPH_SLAM_DEMO.with(|d| {
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

// ═══════════════════════════════════════════════════════════════════════════════
// KALMAN FILTER DEMO RUNNER
// ═══════════════════════════════════════════════════════════════════════════════

/// Kalman Filter demo runner for sensor fusion visualization
pub struct KalmanFilterDemoRunner {
    demo: KalmanFilterDemo,
    canvas: Canvas,
    animation: Option<Rc<AnimationLoop>>,
    paused: bool,
}

impl KalmanFilterDemoRunner {
    /// Start the Kalman Filter demo
    pub fn start(canvas_id: &str, seed: u64) -> Result<(), JsValue> {
        let canvas = Canvas::new(canvas_id)?;
        let mut demo = KalmanFilterDemo::default();
        demo.reset(seed);

        let runner = KalmanFilterDemoRunner {
            demo,
            canvas,
            animation: None,
            paused: false,
        };

        KALMAN_FILTER_DEMO.with(|d| {
            *d.borrow_mut() = Some(runner);
        });

        Self::start_animation()?;
        Self::wire_controls()?;

        Ok(())
    }

    fn start_animation() -> Result<(), JsValue> {
        let animation = AnimationLoop::new(move |dt| {
            KALMAN_FILTER_DEMO.with(|d| {
                if let Some(runner) = d.borrow_mut().as_mut() {
                    if !runner.paused {
                        runner.demo.step(dt as f32);
                    }
                    runner.render();
                }
            });
        });

        animation.start();

        KALMAN_FILTER_DEMO.with(|d| {
            if let Some(runner) = d.borrow_mut().as_mut() {
                runner.animation = Some(Rc::new(animation));
            }
        });

        Ok(())
    }

    fn wire_controls() -> Result<(), JsValue> {
        // Process noise slider
        if let Ok(slider) = get_input("process-noise-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("process-noise-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        KALMAN_FILTER_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("process_noise", value);
                            }
                        });
                        update_text("process-noise-value", &format!("{:.2}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Measurement noise slider
        if let Ok(slider) = get_input("measurement-noise-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("measurement-noise-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        KALMAN_FILTER_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("measurement_noise", value);
                            }
                        });
                        update_text("measurement-noise-value", &format!("{:.1}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // GPS interval slider
        if let Ok(slider) = get_input("gps-interval-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("gps-interval-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        KALMAN_FILTER_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("gps_interval", value);
                            }
                        });
                        update_text("gps-interval-value", &format!("{}", value as i32));
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
                KALMAN_FILTER_DEMO.with(|d| {
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
                KALMAN_FILTER_DEMO.with(|d| {
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

        // Layout
        let margin = 20.0;
        let info_width = 160.0;
        let plot_size = ((w - info_width - 2.0 * margin).min(h - 2.0 * margin - 40.0)).max(200.0);
        let offset_x = margin;
        let offset_y = margin + 30.0;

        // Coordinate transform: [0, 1] -> canvas
        let to_x = |x: f32| -> f64 { offset_x + (x as f64) * plot_size };
        let to_y = |y: f32| -> f64 { offset_y + (1.0 - y as f64) * plot_size };

        // === PHASE INDICATOR ===
        let phase_color = match self.demo.phase {
            KFPhase::Predict => "#ffaa00",
            KFPhase::Update => "#00ff88",
        };

        ctx.set_font("bold 14px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str(phase_color));
        let _ = ctx.fill_text(&format!("Phase: {}", self.demo.phase.name()), margin, 20.0);

        ctx.set_font("12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text(self.demo.phase.description(), margin + 120.0, 20.0);

        // === MAIN PLOT ===
        self.canvas.stroke_rect(offset_x, offset_y, plot_size, plot_size, "rgba(100, 255, 218, 0.3)", 1.0);

        // Grid
        ctx.set_stroke_style(&JsValue::from_str("rgba(100, 255, 218, 0.08)"));
        ctx.set_line_width(1.0);
        for i in 1..10 {
            let pos = i as f64 / 10.0;
            ctx.begin_path();
            ctx.move_to(offset_x + pos * plot_size, offset_y);
            ctx.line_to(offset_x + pos * plot_size, offset_y + plot_size);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(offset_x, offset_y + pos * plot_size);
            ctx.line_to(offset_x + plot_size, offset_y + pos * plot_size);
            ctx.stroke();
        }

        // Draw true path (faint green)
        if self.demo.true_path.len() > 1 {
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 136, 0.3)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(to_x(self.demo.true_path[0].x), to_y(self.demo.true_path[0].y));
            for pos in &self.demo.true_path[1..] {
                ctx.line_to(to_x(pos.x), to_y(pos.y));
            }
            ctx.stroke();
        }

        // Draw estimated path (faint cyan)
        if self.demo.kf_path.len() > 1 {
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 255, 0.3)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(to_x(self.demo.kf_path[0].x), to_y(self.demo.kf_path[0].y));
            for pos in &self.demo.kf_path[1..] {
                ctx.line_to(to_x(pos.x), to_y(pos.y));
            }
            ctx.stroke();
        }

        // Draw GPS measurements (yellow dots)
        for gps in &self.demo.gps_history {
            self.canvas.fill_circle(to_x(gps.x), to_y(gps.y), 3.0, "rgba(255, 255, 100, 0.5)");
        }

        // Draw last GPS measurement (bright yellow)
        if let Some(gps) = self.demo.last_gps {
            self.canvas.fill_circle(to_x(gps.x), to_y(gps.y), 6.0, "#ffff00");
            ctx.set_stroke_style(&JsValue::from_str("#ffff00"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            let _ = ctx.arc(to_x(gps.x), to_y(gps.y), 10.0, 0.0, std::f64::consts::TAU);
            ctx.stroke();
        }

        // Draw uncertainty ellipse
        let (semi_a, semi_b, angle) = self.demo.covariance_ellipse();
        let scale = plot_size as f32;
        ctx.save();
        ctx.translate(to_x(self.demo.kf_pos.x), to_y(self.demo.kf_pos.y)).ok();
        ctx.rotate(-angle as f64).ok();
        ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 255, 0.5)"));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        let _ = ctx.ellipse(
            0.0, 0.0,
            (semi_a * scale) as f64, (semi_b * scale) as f64,
            0.0, 0.0, std::f64::consts::TAU,
        );
        ctx.stroke();
        ctx.restore();

        // Draw estimated position (cyan)
        self.canvas.fill_circle(to_x(self.demo.kf_pos.x), to_y(self.demo.kf_pos.y), 8.0, "#00ffff");

        // Draw true position (green)
        self.canvas.fill_circle(to_x(self.demo.true_pos.x), to_y(self.demo.true_pos.y), 6.0, "#00ff88");

        // === INFO PANEL ===
        let panel_x = offset_x + plot_size + 20.0;
        let mut panel_y = offset_y;

        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("LEGEND", panel_x, panel_y);
        panel_y += 20.0;

        ctx.set_font("11px 'Inter', sans-serif");

        self.canvas.fill_circle(panel_x + 6.0, panel_y, 4.0, "#00ff88");
        ctx.set_fill_style(&JsValue::from_str("#00ff88"));
        let _ = ctx.fill_text("True Position", panel_x + 18.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_circle(panel_x + 6.0, panel_y, 4.0, "#00ffff");
        ctx.set_fill_style(&JsValue::from_str("#00ffff"));
        let _ = ctx.fill_text("KF Estimate", panel_x + 18.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_circle(panel_x + 6.0, panel_y, 3.0, "#ffff00");
        ctx.set_fill_style(&JsValue::from_str("#ffff00"));
        let _ = ctx.fill_text("GPS Measurement", panel_x + 18.0, panel_y + 4.0);
        panel_y += 18.0;

        ctx.set_stroke_style(&JsValue::from_str("#00ffff"));
        ctx.set_line_width(1.0);
        ctx.begin_path();
        let _ = ctx.ellipse(panel_x + 6.0, panel_y, 8.0, 5.0, 0.0, 0.0, std::f64::consts::TAU);
        ctx.stroke();
        ctx.set_fill_style(&JsValue::from_str("#00ffff"));
        let _ = ctx.fill_text("Uncertainty (2σ)", panel_x + 18.0, panel_y + 4.0);
        panel_y += 30.0;

        // Statistics
        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("STATISTICS", panel_x, panel_y);
        panel_y += 18.0;

        ctx.set_font("11px 'Inter', sans-serif");
        let error = self.demo.error();
        let error_color = if error < 0.02 { "#00ff88" } else if error < 0.05 { "#ffaa00" } else { "#ff5555" };
        ctx.set_fill_style(&JsValue::from_str(error_color));
        let _ = ctx.fill_text(&format!("Error: {:.4}", error), panel_x, panel_y);
        panel_y += 16.0;

        ctx.set_fill_style(&JsValue::from_str("#888"));
        let uncertainty = self.demo.uncertainty();
        let _ = ctx.fill_text(&format!("Uncertainty: {:.4}", uncertainty), panel_x, panel_y);
        panel_y += 30.0;

        // Kalman Gain display
        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("KALMAN GAIN", panel_x, panel_y);
        panel_y += 18.0;

        ctx.set_font("10px monospace");
        ctx.set_fill_style(&JsValue::from_str("#666"));
        let k = &self.demo.kalman_gain;
        let _ = ctx.fill_text(&format!("[{:.2} {:.2}]", k.m00, k.m01), panel_x, panel_y);
        panel_y += 14.0;
        let _ = ctx.fill_text(&format!("[{:.2} {:.2}]", k.m10, k.m11), panel_x, panel_y);
        panel_y += 20.0;

        // Explanation
        ctx.set_font("10px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#555"));
        let _ = ctx.fill_text("K near 1 → trust GPS", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("K near 0 → trust odometry", panel_x, panel_y);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPLEMENTARY FILTER DEMO RUNNER
// ═══════════════════════════════════════════════════════════════════════════════

/// Complementary Filter demo runner for IMU sensor fusion visualization
pub struct ComplementaryFilterDemoRunner {
    demo: ComplementaryFilterDemo,
    canvas: Canvas,
    animation: Option<Rc<AnimationLoop>>,
    paused: bool,
}

impl ComplementaryFilterDemoRunner {
    /// Start the Complementary Filter demo
    pub fn start(canvas_id: &str, seed: u64) -> Result<(), JsValue> {
        let canvas = Canvas::new(canvas_id)?;
        let mut demo = ComplementaryFilterDemo::default();
        demo.reset(seed);

        let runner = ComplementaryFilterDemoRunner {
            demo,
            canvas,
            animation: None,
            paused: false,
        };

        COMPLEMENTARY_FILTER_DEMO.with(|d| {
            *d.borrow_mut() = Some(runner);
        });

        Self::start_animation()?;
        Self::wire_controls()?;

        Ok(())
    }

    fn start_animation() -> Result<(), JsValue> {
        let animation = AnimationLoop::new(move |dt| {
            COMPLEMENTARY_FILTER_DEMO.with(|d| {
                if let Some(runner) = d.borrow_mut().as_mut() {
                    if !runner.paused {
                        runner.demo.step(dt as f32);
                    }
                    runner.render();
                }
            });
        });

        animation.start();

        COMPLEMENTARY_FILTER_DEMO.with(|d| {
            if let Some(runner) = d.borrow_mut().as_mut() {
                runner.animation = Some(Rc::new(animation));
            }
        });

        Ok(())
    }

    fn wire_controls() -> Result<(), JsValue> {
        // Alpha slider
        if let Ok(slider) = get_input("alpha-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("alpha-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        COMPLEMENTARY_FILTER_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("alpha", value);
                            }
                        });
                        update_text("alpha-value", &format!("{:.3}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Accel noise slider
        if let Ok(slider) = get_input("accel-noise-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("accel-noise-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        COMPLEMENTARY_FILTER_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("accel_noise", value);
                            }
                        });
                        update_text("accel-noise-value", &format!("{:.1}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Gyro drift slider
        if let Ok(slider) = get_input("gyro-drift-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("gyro-drift-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        COMPLEMENTARY_FILTER_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("gyro_drift", value);
                            }
                        });
                        update_text("gyro-drift-value", &format!("{:.1}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Motion speed slider
        if let Ok(slider) = get_input("motion-speed-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("motion-speed-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        COMPLEMENTARY_FILTER_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("motion_speed", value);
                            }
                        });
                        update_text("motion-speed-value", &format!("{:.1}", value));
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
                COMPLEMENTARY_FILTER_DEMO.with(|d| {
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
                COMPLEMENTARY_FILTER_DEMO.with(|d| {
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
        let _h = self.canvas.height();

        // Clear background
        self.canvas.clear("#0a0a12");

        // Layout: 3 signal plots (accel, fused, gyro) + info panel
        let margin = 20.0;
        let plot_height = 120.0;
        let plot_width = w - 2.0 * margin - 140.0;  // Leave room for info panel

        // Title bar
        ctx.set_font("bold 14px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#64ffda"));
        let _ = ctx.fill_text("IMU Sensor Fusion Demo", margin, 20.0);

        ctx.set_font("12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text(&format!("α = {:.3}", self.demo.alpha), margin + 200.0, 20.0);

        // === PLOT 1: ACCELEROMETER (jittery, no drift) ===
        let plot1_y = 40.0;
        self.render_signal_plot(
            margin, plot1_y, plot_width, plot_height,
            "Accelerometer (jittery, no drift)",
            "#ff6666",
            &self.demo.history.accel,
            &self.demo.history.true_angle,
        );

        // === PLOT 2: FUSED OUTPUT (best of both) ===
        let plot2_y = plot1_y + plot_height + 20.0;
        self.render_signal_plot(
            margin, plot2_y, plot_width, plot_height,
            "Fused Output (complementary filter)",
            "#00ff88",
            &self.demo.history.fused,
            &self.demo.history.true_angle,
        );

        // === PLOT 3: GYROSCOPE (smooth, drifts) ===
        let plot3_y = plot2_y + plot_height + 20.0;
        self.render_signal_plot(
            margin, plot3_y, plot_width, plot_height,
            "Gyroscope (smooth, drifts over time)",
            "#66aaff",
            &self.demo.history.gyro,
            &self.demo.history.true_angle,
        );

        // === INFO PANEL ===
        let panel_x = margin + plot_width + 20.0;
        let mut panel_y = 50.0;

        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("CURRENT ERRORS", panel_x, panel_y);
        panel_y += 20.0;

        ctx.set_font("11px 'Inter', sans-serif");

        // Accel error
        let accel_err = self.demo.accel_error();
        let accel_color = if accel_err < 5.0 { "#00ff88" } else if accel_err < 10.0 { "#ffaa00" } else { "#ff5555" };
        ctx.set_fill_style(&JsValue::from_str(accel_color));
        let _ = ctx.fill_text(&format!("Accel: {:.1}°", accel_err), panel_x, panel_y);
        panel_y += 16.0;

        // Fused error
        let fused_err = self.demo.fusion_error();
        let fused_color = if fused_err < 2.0 { "#00ff88" } else if fused_err < 5.0 { "#ffaa00" } else { "#ff5555" };
        ctx.set_fill_style(&JsValue::from_str(fused_color));
        let _ = ctx.fill_text(&format!("Fused: {:.1}°", fused_err), panel_x, panel_y);
        panel_y += 16.0;

        // Gyro error
        let gyro_err = self.demo.gyro_error();
        let gyro_color = if gyro_err < 5.0 { "#00ff88" } else if gyro_err < 15.0 { "#ffaa00" } else { "#ff5555" };
        ctx.set_fill_style(&JsValue::from_str(gyro_color));
        let _ = ctx.fill_text(&format!("Gyro: {:.1}°", gyro_err), panel_x, panel_y);
        panel_y += 30.0;

        // Legend
        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("LEGEND", panel_x, panel_y);
        panel_y += 18.0;

        ctx.set_font("11px 'Inter', sans-serif");

        // True angle
        ctx.set_stroke_style(&JsValue::from_str("rgba(150, 150, 150, 0.8)"));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        ctx.move_to(panel_x, panel_y);
        ctx.line_to(panel_x + 20.0, panel_y);
        ctx.stroke();
        ctx.set_fill_style(&JsValue::from_str("#999"));
        let _ = ctx.fill_text("True angle", panel_x + 25.0, panel_y + 4.0);
        panel_y += 16.0;

        // Sensor reading
        ctx.set_stroke_style(&JsValue::from_str("#ff6666"));
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(panel_x, panel_y);
        ctx.line_to(panel_x + 20.0, panel_y);
        ctx.stroke();
        ctx.set_fill_style(&JsValue::from_str("#ff6666"));
        let _ = ctx.fill_text("Sensor", panel_x + 25.0, panel_y + 4.0);
        panel_y += 30.0;

        // Formula
        ctx.set_font("bold 11px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("FORMULA", panel_x, panel_y);
        panel_y += 16.0;

        ctx.set_font("10px monospace");
        ctx.set_fill_style(&JsValue::from_str("#666"));
        let _ = ctx.fill_text("angle = ", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("  α×(angle+gyro×dt)", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("  +(1-α)×accel", panel_x, panel_y);
    }

    fn render_signal_plot(
        &self,
        x: f64, y: f64,
        w: f64, h: f64,
        title: &str,
        color: &str,
        signal: &[f32],
        truth: &[f32],
    ) {
        let ctx = self.canvas.ctx();

        // Background
        self.canvas.fill_rect(x, y, w, h, "rgba(20, 20, 30, 0.8)");
        self.canvas.stroke_rect(x, y, w, h, "rgba(100, 255, 218, 0.2)", 1.0);

        // Title
        ctx.set_font("11px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str(color));
        let _ = ctx.fill_text(title, x + 5.0, y + 14.0);

        if signal.is_empty() || truth.is_empty() {
            return;
        }

        // Determine Y-axis range (auto-scale based on data)
        let all_values: Vec<f32> = signal.iter().chain(truth.iter()).copied().collect();
        let min_val = all_values.iter().copied().fold(f32::INFINITY, f32::min);
        let max_val = all_values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let range = (max_val - min_val).max(10.0);  // At least 10 degrees range
        let center = (max_val + min_val) / 2.0;
        let y_min = center - range * 0.6;
        let y_max = center + range * 0.6;

        // Convert value to Y coordinate
        let to_y_coord = |val: f32| -> f64 {
            let normalized = (val - y_min) / (y_max - y_min);
            y + h - 20.0 - normalized as f64 * (h - 30.0)
        };

        // Draw zero line
        if y_min < 0.0 && y_max > 0.0 {
            ctx.set_stroke_style(&JsValue::from_str("rgba(100, 100, 100, 0.3)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(x, to_y_coord(0.0));
            ctx.line_to(x + w, to_y_coord(0.0));
            ctx.stroke();
        }

        // Draw true angle (gray dashed line)
        ctx.set_stroke_style(&JsValue::from_str("rgba(150, 150, 150, 0.5)"));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        for (i, &val) in truth.iter().enumerate() {
            let px = x + 5.0 + (i as f64 / truth.len() as f64) * (w - 10.0);
            let py = to_y_coord(val);
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.stroke();

        // Draw signal
        ctx.set_stroke_style(&JsValue::from_str(color));
        ctx.set_line_width(1.5);
        ctx.begin_path();
        for (i, &val) in signal.iter().enumerate() {
            let px = x + 5.0 + (i as f64 / signal.len() as f64) * (w - 10.0);
            let py = to_y_coord(val);
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.stroke();

        // Y-axis labels
        ctx.set_font("9px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#555"));
        let _ = ctx.fill_text(&format!("{:.0}°", y_max), x + w - 30.0, y + 25.0);
        let _ = ctx.fill_text(&format!("{:.0}°", y_min), x + w - 30.0, y + h - 8.0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EKF SLAM DEMO RUNNER
// ═══════════════════════════════════════════════════════════════════════════════

/// EKF SLAM demo runner - localization AND mapping simultaneously
pub struct EkfSlamDemoRunner {
    demo: EkfSlamDemo,
    canvas: Canvas,
    animation: Option<Rc<AnimationLoop>>,
    paused: bool,
}

impl EkfSlamDemoRunner {
    /// Start the EKF SLAM demo
    pub fn start(canvas_id: &str, seed: u64) -> Result<(), JsValue> {
        let canvas = Canvas::new(canvas_id)?;
        let mut demo = EkfSlamDemo::default();
        demo.reset(seed);

        let runner = EkfSlamDemoRunner {
            demo,
            canvas,
            animation: None,
            paused: false,
        };

        EKF_SLAM_DEMO.with(|d| {
            *d.borrow_mut() = Some(runner);
        });

        Self::start_animation()?;
        Self::wire_controls()?;

        Ok(())
    }

    fn start_animation() -> Result<(), JsValue> {
        let animation = AnimationLoop::new(move |dt| {
            EKF_SLAM_DEMO.with(|d| {
                if let Some(runner) = d.borrow_mut().as_mut() {
                    if !runner.paused {
                        runner.demo.step(dt as f32);
                    }
                    runner.render();
                }
            });
        });

        animation.start();

        EKF_SLAM_DEMO.with(|d| {
            if let Some(runner) = d.borrow_mut().as_mut() {
                runner.animation = Some(Rc::new(animation));
            }
        });

        Ok(())
    }

    fn wire_controls() -> Result<(), JsValue> {
        // Sensor range slider
        if let Ok(slider) = get_input("sensor-range-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("sensor-range-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        EKF_SLAM_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("sensor_range", value);
                            }
                        });
                        update_text("sensor-range-value", &format!("{:.2}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Motion noise slider
        if let Ok(slider) = get_input("motion-noise-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("motion-noise-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        EKF_SLAM_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("motion_noise", value);
                            }
                        });
                        update_text("motion-noise-value", &format!("{:.3}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Observation noise slider
        if let Ok(slider) = get_input("obs-noise-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("obs-noise-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        EKF_SLAM_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("sensor_noise", value);
                            }
                        });
                        update_text("obs-noise-value", &format!("{:.2}", value));
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
                EKF_SLAM_DEMO.with(|d| {
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
                EKF_SLAM_DEMO.with(|d| {
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

        // Layout
        let margin = 20.0;
        let info_width = 160.0;
        let plot_size = ((w - info_width - 2.0 * margin).min(h - 2.0 * margin - 40.0)).max(200.0);
        let offset_x = margin;
        let offset_y = margin + 30.0;

        // Coordinate transform: [0, 1] -> canvas
        let to_x = |x: f32| -> f64 { offset_x + (x as f64) * plot_size };
        let to_y = |y: f32| -> f64 { offset_y + (1.0 - y as f64) * plot_size };

        // === TITLE ===
        ctx.set_font("bold 14px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#64ffda"));
        let _ = ctx.fill_text("EKF SLAM - Simultaneous Localization and Mapping", margin, 20.0);

        // === MAIN PLOT ===
        self.canvas.stroke_rect(offset_x, offset_y, plot_size, plot_size, "rgba(100, 255, 218, 0.3)", 1.0);

        // Grid
        ctx.set_stroke_style(&JsValue::from_str("rgba(100, 255, 218, 0.08)"));
        ctx.set_line_width(1.0);
        for i in 1..10 {
            let pos = i as f64 / 10.0;
            ctx.begin_path();
            ctx.move_to(offset_x + pos * plot_size, offset_y);
            ctx.line_to(offset_x + pos * plot_size, offset_y + plot_size);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(offset_x, offset_y + pos * plot_size);
            ctx.line_to(offset_x + plot_size, offset_y + pos * plot_size);
            ctx.stroke();
        }

        // Draw sensor range circle around robot
        ctx.set_stroke_style(&JsValue::from_str("rgba(255, 255, 100, 0.2)"));
        ctx.set_line_width(1.0);
        ctx.begin_path();
        let _ = ctx.arc(
            to_x(self.demo.true_pos.x),
            to_y(self.demo.true_pos.y),
            self.demo.sensor_range as f64 * plot_size,
            0.0, std::f64::consts::TAU
        );
        ctx.stroke();

        // Draw robot paths
        if self.demo.robot_path.len() > 1 {
            // True path (faint green)
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 136, 0.3)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(to_x(self.demo.robot_path[0].x), to_y(self.demo.robot_path[0].y));
            for pos in &self.demo.robot_path[1..] {
                ctx.line_to(to_x(pos.x), to_y(pos.y));
            }
            ctx.stroke();

            // Estimated path (faint cyan)
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 255, 0.3)"));
            ctx.begin_path();
            ctx.move_to(to_x(self.demo.est_path[0].x), to_y(self.demo.est_path[0].y));
            for pos in &self.demo.est_path[1..] {
                ctx.line_to(to_x(pos.x), to_y(pos.y));
            }
            ctx.stroke();
        }

        // Draw discovered landmarks with uncertainty ellipses
        for (i, lm) in self.demo.landmarks.iter().enumerate() {
            // Uncertainty ellipse
            let sigma_x = lm.variance.x.sqrt() * 3.0; // 3-sigma ellipse
            let sigma_y = lm.variance.y.sqrt() * 3.0;

            ctx.set_stroke_style(&JsValue::from_str("rgba(255, 150, 100, 0.5)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            let _ = ctx.ellipse(
                to_x(lm.pos.x), to_y(lm.pos.y),
                (sigma_x as f64 * plot_size).max(3.0),
                (sigma_y as f64 * plot_size).max(3.0),
                0.0, 0.0, std::f64::consts::TAU
            );
            ctx.stroke();

            // Landmark point
            let color = if Some(i) == self.demo.last_observed_idx {
                if self.demo.last_was_new { "#ffff00" } else { "#00ff88" }
            } else {
                "#ff9664"
            };
            self.canvas.fill_rect(to_x(lm.pos.x) - 4.0, to_y(lm.pos.y) - 4.0, 8.0, 8.0, color);

            // Observation count
            ctx.set_font("9px 'Inter', sans-serif");
            ctx.set_fill_style(&JsValue::from_str("#888"));
            let _ = ctx.fill_text(&format!("×{}", lm.observations), to_x(lm.pos.x) + 6.0, to_y(lm.pos.y) - 6.0);
        }

        // Draw robot uncertainty ellipse
        let robot_sigma_x = self.demo.robot_variance.x.sqrt() * 3.0;
        let robot_sigma_y = self.demo.robot_variance.y.sqrt() * 3.0;
        ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 255, 0.5)"));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        let _ = ctx.ellipse(
            to_x(self.demo.est_pos.x), to_y(self.demo.est_pos.y),
            (robot_sigma_x as f64 * plot_size).max(5.0),
            (robot_sigma_y as f64 * plot_size).max(5.0),
            0.0, 0.0, std::f64::consts::TAU
        );
        ctx.stroke();

        // Draw estimated robot (cyan)
        self.canvas.fill_triangle(
            to_x(self.demo.est_pos.x),
            to_y(self.demo.est_pos.y),
            12.0,
            -self.demo.est_theta as f64,
            "#00ffff",
        );

        // Draw true robot (green)
        self.canvas.fill_triangle(
            to_x(self.demo.true_pos.x),
            to_y(self.demo.true_pos.y),
            14.0,
            -self.demo.true_theta as f64,
            "#00ff88",
        );

        // === INFO PANEL ===
        let panel_x = offset_x + plot_size + 20.0;
        let mut panel_y = offset_y;

        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("LEGEND", panel_x, panel_y);
        panel_y += 20.0;

        ctx.set_font("11px 'Inter', sans-serif");

        self.canvas.fill_triangle(panel_x + 8.0, panel_y, 8.0, 0.0, "#00ff88");
        ctx.set_fill_style(&JsValue::from_str("#00ff88"));
        let _ = ctx.fill_text("True Robot", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_triangle(panel_x + 8.0, panel_y, 8.0, 0.0, "#00ffff");
        ctx.set_fill_style(&JsValue::from_str("#00ffff"));
        let _ = ctx.fill_text("Estimated", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_rect(panel_x + 4.0, panel_y - 4.0, 8.0, 8.0, "#ff9664");
        ctx.set_fill_style(&JsValue::from_str("#ff9664"));
        let _ = ctx.fill_text("Landmarks", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_rect(panel_x + 4.0, panel_y - 4.0, 8.0, 8.0, "#ffff00");
        ctx.set_fill_style(&JsValue::from_str("#ffff00"));
        let _ = ctx.fill_text("New Discovery", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_rect(panel_x + 4.0, panel_y - 4.0, 8.0, 8.0, "#00ff88");
        ctx.set_fill_style(&JsValue::from_str("#00ff88"));
        let _ = ctx.fill_text("Re-observed", panel_x + 22.0, panel_y + 4.0);
        panel_y += 30.0;

        // Statistics
        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("STATISTICS", panel_x, panel_y);
        panel_y += 18.0;

        ctx.set_font("11px 'Inter', sans-serif");

        let robot_error = self.demo.robot_error();
        let robot_color = if robot_error < 0.03 { "#00ff88" } else if robot_error < 0.08 { "#ffaa00" } else { "#ff5555" };
        ctx.set_fill_style(&JsValue::from_str(robot_color));
        let _ = ctx.fill_text(&format!("Robot Error: {:.3}", robot_error), panel_x, panel_y);
        panel_y += 16.0;

        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text(&format!("Landmarks: {}", self.demo.landmarks.len()), panel_x, panel_y);
        panel_y += 16.0;

        let map_error = self.demo.map_error();
        let map_color = if map_error < 0.02 { "#00ff88" } else if map_error < 0.05 { "#ffaa00" } else { "#ff5555" };
        ctx.set_fill_style(&JsValue::from_str(map_color));
        let _ = ctx.fill_text(&format!("Map Error: {:.3}", map_error), panel_x, panel_y);
        panel_y += 30.0;

        // Explanation
        ctx.set_font("10px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#555"));
        let _ = ctx.fill_text("Watch uncertainty", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("ellipses shrink when", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("landmarks are", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("re-observed!", panel_x, panel_y);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GRAPH SLAM DEMO RUNNER
// ═══════════════════════════════════════════════════════════════════════════════

/// Graph SLAM demo runner - pose graph optimization visualization
pub struct GraphSlamDemoRunner {
    demo: GraphSlamDemo,
    canvas: Canvas,
    animation: Option<Rc<AnimationLoop>>,
    paused: bool,
}

impl GraphSlamDemoRunner {
    /// Start the Graph SLAM demo
    pub fn start(canvas_id: &str, seed: u64) -> Result<(), JsValue> {
        let canvas = Canvas::new(canvas_id)?;
        let mut demo = GraphSlamDemo::default();
        demo.reset(seed);

        let runner = GraphSlamDemoRunner {
            demo,
            canvas,
            animation: None,
            paused: false,
        };

        GRAPH_SLAM_DEMO.with(|d| {
            *d.borrow_mut() = Some(runner);
        });

        Self::start_animation()?;
        Self::wire_controls()?;

        Ok(())
    }

    fn start_animation() -> Result<(), JsValue> {
        let animation = AnimationLoop::new(move |dt| {
            GRAPH_SLAM_DEMO.with(|d| {
                if let Some(runner) = d.borrow_mut().as_mut() {
                    if !runner.paused {
                        runner.demo.step(dt as f32);
                    }
                    runner.render();
                }
            });
        });

        animation.start();

        GRAPH_SLAM_DEMO.with(|d| {
            if let Some(runner) = d.borrow_mut().as_mut() {
                runner.animation = Some(Rc::new(animation));
            }
        });

        Ok(())
    }

    fn wire_controls() -> Result<(), JsValue> {
        // Odometry noise slider
        if let Ok(slider) = get_input("odom-noise-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("odom-noise-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        GRAPH_SLAM_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("odometry_noise", value);
                            }
                        });
                        update_text("odom-noise-value", &format!("{:.2}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Loop closure threshold slider
        if let Ok(slider) = get_input("lc-threshold-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("lc-threshold-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        GRAPH_SLAM_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("loop_threshold", value);
                            }
                        });
                        update_text("lc-threshold-value", &format!("{:.2}", value));
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
                GRAPH_SLAM_DEMO.with(|d| {
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
                GRAPH_SLAM_DEMO.with(|d| {
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

        // Optimize button
        if let Some(btn) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("optimize-btn"))
        {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                GRAPH_SLAM_DEMO.with(|d| {
                    if let Some(runner) = d.borrow_mut().as_mut() {
                        // Run several optimization iterations
                        for _ in 0..20 {
                            runner.demo.optimize_step();
                        }
                    }
                });
            }) as Box<dyn FnMut(_)>);
            btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Add loop closure button
        if let Some(btn) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("add-lc-btn"))
        {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                GRAPH_SLAM_DEMO.with(|d| {
                    if let Some(runner) = d.borrow_mut().as_mut() {
                        runner.demo.add_manual_loop_closure();
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

        // Layout
        let margin = 20.0;
        let info_width = 160.0;
        let plot_size = ((w - info_width - 2.0 * margin).min(h - 2.0 * margin - 40.0)).max(200.0);
        let offset_x = margin;
        let offset_y = margin + 30.0;

        // Coordinate transform: [0, 1] -> canvas
        let to_x = |x: f32| -> f64 { offset_x + (x as f64) * plot_size };
        let to_y = |y: f32| -> f64 { offset_y + (1.0 - y as f64) * plot_size };

        // === TITLE ===
        ctx.set_font("bold 14px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#64ffda"));
        let _ = ctx.fill_text("Graph SLAM - Pose Graph Optimization", margin, 20.0);

        // === MAIN PLOT ===
        self.canvas.stroke_rect(offset_x, offset_y, plot_size, plot_size, "rgba(100, 255, 218, 0.3)", 1.0);

        // Grid
        ctx.set_stroke_style(&JsValue::from_str("rgba(100, 255, 218, 0.08)"));
        ctx.set_line_width(1.0);
        for i in 1..10 {
            let pos = i as f64 / 10.0;
            ctx.begin_path();
            ctx.move_to(offset_x + pos * plot_size, offset_y);
            ctx.line_to(offset_x + pos * plot_size, offset_y + plot_size);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(offset_x, offset_y + pos * plot_size);
            ctx.line_to(offset_x + plot_size, offset_y + pos * plot_size);
            ctx.stroke();
        }

        // Draw true path (faint green)
        if self.demo.true_path.len() > 1 {
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 136, 0.2)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(to_x(self.demo.true_path[0].x), to_y(self.demo.true_path[0].y));
            for pos in &self.demo.true_path[1..] {
                ctx.line_to(to_x(pos.x), to_y(pos.y));
            }
            ctx.stroke();
        }

        // Draw edges
        for edge in &self.demo.edges {
            let from = &self.demo.nodes[edge.from];
            let to = &self.demo.nodes[edge.to];

            let color = if edge.is_loop_closure {
                "rgba(255, 100, 100, 0.8)" // Red for loop closures
            } else {
                "rgba(100, 150, 255, 0.4)" // Blue for odometry
            };

            ctx.set_stroke_style(&JsValue::from_str(color));
            ctx.set_line_width(if edge.is_loop_closure { 2.0 } else { 1.0 });
            ctx.begin_path();
            ctx.move_to(to_x(from.pos.x), to_y(from.pos.y));
            ctx.line_to(to_x(to.pos.x), to_y(to.pos.y));
            ctx.stroke();
        }

        // Highlight last loop closure
        if let Some((from_idx, to_idx)) = self.demo.last_loop_closure {
            if from_idx < self.demo.nodes.len() && to_idx < self.demo.nodes.len() {
                let from = &self.demo.nodes[from_idx];
                let to = &self.demo.nodes[to_idx];

                ctx.set_stroke_style(&JsValue::from_str("#ff5555"));
                ctx.set_line_width(3.0);
                ctx.begin_path();
                ctx.move_to(to_x(from.pos.x), to_y(from.pos.y));
                ctx.line_to(to_x(to.pos.x), to_y(to.pos.y));
                ctx.stroke();
            }
        }

        // Draw nodes
        for (i, node) in self.demo.nodes.iter().enumerate() {
            let is_latest = i == self.demo.nodes.len().saturating_sub(1);
            let is_first = i == 0;

            let color = if is_latest {
                "#00ffff" // Cyan for latest
            } else if is_first {
                "#ffff00" // Yellow for first (anchor)
            } else {
                "#6688ff" // Blue for others
            };

            let size = if is_latest || is_first { 6.0 } else { 4.0 };
            self.canvas.fill_circle(to_x(node.pos.x), to_y(node.pos.y), size, color);
        }

        // Draw true robot position (green)
        self.canvas.fill_triangle(
            to_x(self.demo.true_pos.x),
            to_y(self.demo.true_pos.y),
            12.0,
            -self.demo.true_theta as f64,
            "#00ff88",
        );

        // === INFO PANEL ===
        let panel_x = offset_x + plot_size + 20.0;
        let mut panel_y = offset_y;

        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("LEGEND", panel_x, panel_y);
        panel_y += 20.0;

        ctx.set_font("11px 'Inter', sans-serif");

        self.canvas.fill_triangle(panel_x + 8.0, panel_y, 8.0, 0.0, "#00ff88");
        ctx.set_fill_style(&JsValue::from_str("#00ff88"));
        let _ = ctx.fill_text("True Robot", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_circle(panel_x + 8.0, panel_y, 4.0, "#ffff00");
        ctx.set_fill_style(&JsValue::from_str("#ffff00"));
        let _ = ctx.fill_text("First Node", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_circle(panel_x + 8.0, panel_y, 4.0, "#00ffff");
        ctx.set_fill_style(&JsValue::from_str("#00ffff"));
        let _ = ctx.fill_text("Latest Node", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        self.canvas.fill_circle(panel_x + 8.0, panel_y, 3.0, "#6688ff");
        ctx.set_fill_style(&JsValue::from_str("#6688ff"));
        let _ = ctx.fill_text("Pose Nodes", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        ctx.set_stroke_style(&JsValue::from_str("#6699ff"));
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(panel_x, panel_y);
        ctx.line_to(panel_x + 16.0, panel_y);
        ctx.stroke();
        ctx.set_fill_style(&JsValue::from_str("#6699ff"));
        let _ = ctx.fill_text("Odometry", panel_x + 22.0, panel_y + 4.0);
        panel_y += 18.0;

        ctx.set_stroke_style(&JsValue::from_str("#ff5555"));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        ctx.move_to(panel_x, panel_y);
        ctx.line_to(panel_x + 16.0, panel_y);
        ctx.stroke();
        ctx.set_fill_style(&JsValue::from_str("#ff5555"));
        let _ = ctx.fill_text("Loop Closure", panel_x + 22.0, panel_y + 4.0);
        panel_y += 30.0;

        // Statistics
        ctx.set_font("bold 12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#aaa"));
        let _ = ctx.fill_text("STATISTICS", panel_x, panel_y);
        panel_y += 18.0;

        ctx.set_font("11px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text(&format!("Nodes: {}", self.demo.nodes.len()), panel_x, panel_y);
        panel_y += 16.0;

        let _ = ctx.fill_text(&format!("Edges: {}", self.demo.edges.len()), panel_x, panel_y);
        panel_y += 16.0;

        let lc_count = self.demo.loop_closure_count();
        let lc_color = if lc_count > 0 { "#00ff88" } else { "#888" };
        ctx.set_fill_style(&JsValue::from_str(lc_color));
        let _ = ctx.fill_text(&format!("Loop Closures: {}", lc_count), panel_x, panel_y);
        panel_y += 16.0;

        let drift = self.demo.drift_error();
        let drift_color = if drift < 0.05 { "#00ff88" } else if drift < 0.1 { "#ffaa00" } else { "#ff5555" };
        ctx.set_fill_style(&JsValue::from_str(drift_color));
        let _ = ctx.fill_text(&format!("Drift: {:.3}", drift), panel_x, panel_y);
        panel_y += 16.0;

        let graph_err = self.demo.graph_error();
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text(&format!("Graph Error: {:.2}", graph_err), panel_x, panel_y);
        panel_y += 30.0;

        // Instructions
        ctx.set_font("10px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#555"));
        let _ = ctx.fill_text("Click 'Optimize' to", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("run graph optimization", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("after loop closures", panel_x, panel_y);
        panel_y += 12.0;
        let _ = ctx.fill_text("are detected.", panel_x, panel_y);
    }
}
