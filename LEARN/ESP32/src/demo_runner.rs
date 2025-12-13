//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: demo_runner.rs | ESP32/src/demo_runner.rs
//! PURPOSE: GPIO Debounce demo runner and visualization
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → ESP32
//! ═══════════════════════════════════════════════════════════════════════════════

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use learn_core::demos::GpioDebounceDemo;
use learn_core::Demo;
use learn_web::{AnimationLoop, Canvas};

// Thread-local state for the currently running demo
thread_local! {
    static CURRENT_DEMO: RefCell<Option<GpioDebounceDemoRunner>> = RefCell::new(None);
}

/// GPIO Debounce demo runner
pub struct GpioDebounceDemoRunner {
    demo: GpioDebounceDemo,
    canvas: Canvas,
    animation: Option<Rc<AnimationLoop>>,
    paused: bool,
}

impl GpioDebounceDemoRunner {
    /// Start the GPIO Debounce demo
    pub fn start(canvas_id: &str, seed: u64) -> Result<(), JsValue> {
        let canvas = Canvas::new(canvas_id)?;
        let mut demo = GpioDebounceDemo::default();
        demo.reset(seed);

        let runner = GpioDebounceDemoRunner {
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
        // Bounce severity slider
        if let Ok(slider) = get_input("bounce-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("bounce-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        CURRENT_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("bounce_severity", value);
                            }
                        });
                        update_text("bounce-value", &format!("{:.2}", value));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Sample rate slider (display as Hz, but convert ms to seconds internally)
        if let Ok(slider) = get_input("sample-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("sample-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        CURRENT_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("sample_rate", value * 10.0); // Scale up
                            }
                        });
                        update_text("sample-value", &format!("{}", value as i32));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            slider.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Debounce window slider (ms to seconds)
        if let Ok(slider) = get_input("window-slider") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(slider) = get_input("window-slider") {
                    if let Ok(value) = slider.value().parse::<f32>() {
                        CURRENT_DEMO.with(|d| {
                            if let Some(runner) = d.borrow_mut().as_mut() {
                                runner.demo.set_param("debounce_window", value / 1000.0);
                            }
                        });
                        update_text("window-value", &format!("{}", value as i32));
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
        let timeline_height = 60.0;
        let gap = 40.0;
        let led_size = 40.0;

        // Timeline dimensions
        let timeline_width = w - 2.0 * margin - led_size - 30.0;
        let timeline_x = margin;

        // Draw labels
        ctx.set_font("12px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text("Raw Signal", timeline_x, margin + 10.0);
        let _ = ctx.fill_text("Debounced", timeline_x, margin + timeline_height + gap + 10.0);

        // Draw raw signal timeline
        let raw_y = margin + 25.0;
        self.draw_timeline(timeline_x, raw_y, timeline_width, timeline_height - 15.0, &self.demo.raw_history, "#ff6644");

        // Draw debounced signal timeline
        let debounce_y = margin + timeline_height + gap + 25.0;
        self.draw_timeline(timeline_x, debounce_y, timeline_width, timeline_height - 15.0, &self.demo.debounced_history, "#44ff88");

        // Draw LED indicator
        let led_x = w - margin - led_size / 2.0;
        let led_y = margin + timeline_height + gap / 2.0;

        // LED glow
        if self.demo.debounced_state {
            ctx.set_fill_style(&JsValue::from_str("rgba(68, 255, 136, 0.3)"));
            ctx.begin_path();
            let _ = ctx.arc(led_x, led_y, led_size * 0.8, 0.0, std::f64::consts::TAU);
            ctx.fill();
        }

        // LED body
        let led_color = if self.demo.debounced_state { "#44ff88" } else { "#442222" };
        self.canvas.fill_circle(led_x, led_y, led_size / 2.0, led_color);

        // LED border
        ctx.set_stroke_style(&JsValue::from_str(if self.demo.debounced_state { "#88ffaa" } else { "#664444" }));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        let _ = ctx.arc(led_x, led_y, led_size / 2.0, 0.0, std::f64::consts::TAU);
        ctx.stroke();

        // LED label
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text("LED", led_x - 10.0, led_y + led_size / 2.0 + 15.0);

        // Draw bouncing indicator
        let bounce_y = h - margin - 30.0;
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let _ = ctx.fill_text("Status:", margin, bounce_y);

        let (status_text, status_color) = if self.demo.is_bouncing() {
            ("BOUNCING", "#ff6644")
        } else if self.demo.debounced_state {
            ("HIGH (Stable)", "#44ff88")
        } else {
            ("LOW (Stable)", "#666")
        };
        ctx.set_fill_style(&JsValue::from_str(status_color));
        ctx.set_font("bold 14px 'Inter', sans-serif");
        let _ = ctx.fill_text(status_text, margin + 60.0, bounce_y);

        // Draw raw vs debounced state
        ctx.set_font("12px 'JetBrains Mono', monospace");
        ctx.set_fill_style(&JsValue::from_str("#888"));
        let raw_indicator = if self.demo.raw_state { "1" } else { "0" };
        let deb_indicator = if self.demo.debounced_state { "1" } else { "0" };
        let _ = ctx.fill_text(
            &format!("Raw: {} | Debounced: {}", raw_indicator, deb_indicator),
            margin + 200.0,
            bounce_y,
        );

        // Draw time
        ctx.set_fill_style(&JsValue::from_str("#555"));
        let _ = ctx.fill_text(&format!("Time: {:.2}s", self.demo.time), w - margin - 80.0, bounce_y);
    }

    fn draw_timeline(&self, x: f64, y: f64, width: f64, height: f64, history: &[bool], color: &str) {
        let ctx = self.canvas.ctx();

        // Background
        ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.03)"));
        ctx.fill_rect(x, y, width, height);

        // Border
        ctx.set_stroke_style(&JsValue::from_str("rgba(255, 255, 255, 0.1)"));
        ctx.set_line_width(1.0);
        ctx.stroke_rect(x, y, width, height);

        // Draw signal
        if history.is_empty() {
            return;
        }

        ctx.set_stroke_style(&JsValue::from_str(color));
        ctx.set_line_width(2.0);
        ctx.begin_path();

        let py_high = y + 5.0;
        let py_low = y + height - 5.0;

        let len = history.len();
        let step = width / len as f64;

        let mut prev_state = history[0];
        let start_py = if prev_state { py_high } else { py_low };
        ctx.move_to(x, start_py);

        for (i, &state) in history.iter().enumerate() {
            let px = x + (i as f64) * step;

            if state != prev_state {
                // Draw horizontal line at previous level
                let prev_py = if prev_state { py_high } else { py_low };
                ctx.line_to(px, prev_py);
                // Then vertical transition
                let curr_py = if state { py_high } else { py_low };
                ctx.line_to(px, curr_py);
            }

            prev_state = state;
        }

        // Final horizontal segment
        let final_py = if prev_state { py_high } else { py_low };
        ctx.line_to(x + width, final_py);

        ctx.stroke();

        // Draw HIGH/LOW labels
        ctx.set_font("10px 'Inter', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#555"));
        let _ = ctx.fill_text("1", x - 12.0, py_high + 4.0);
        let _ = ctx.fill_text("0", x - 12.0, py_low + 4.0);
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
