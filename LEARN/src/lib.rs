//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | src/lib.rs
//! PURPOSE: Library crate root module with public API exports
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → src
//! ═══════════════════════════════════════════════════════════════════════════════
// Learn - ML Fundamentals Interactive Platform
// Client-side ML learning with visualizations
#![allow(unexpected_cfgs)]

use wasm_bindgen::prelude::*;

pub mod demo_runner;
pub mod lessons;
pub mod render;

use demo_runner::LinearRegressionDemoRunner;
use lessons::LESSONS;
use render::LessonRenderer;

/// App state
pub struct App {
    current_lesson: usize,
    renderer: LessonRenderer,
}

impl App {
    pub fn new() -> Result<Self, JsValue> {
        let renderer = LessonRenderer::new("app")?;
        Ok(Self {
            current_lesson: 0,
            renderer,
        })
    }

    pub fn render_current(&self) -> Result<(), JsValue> {
        if let Some(lesson) = LESSONS.get(self.current_lesson) {
            self.renderer.render_lesson(lesson)?;
        }
        Ok(())
    }

    pub fn navigate(&mut self, lesson_idx: usize) -> Result<(), JsValue> {
        if lesson_idx < LESSONS.len() {
            self.current_lesson = lesson_idx;
            self.render_current()?;
        }
        Ok(())
    }

    pub fn render_home(&self) -> Result<(), JsValue> {
        self.renderer.render_home(LESSONS)
    }
}

/// Expose functions to window for onclick handlers
fn expose_to_window() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;

    let go_to_lesson_fn = Closure::wrap(Box::new(|idx: usize| {
        go_to_lesson(idx);
    }) as Box<dyn Fn(usize)>);

    let go_home_fn = Closure::wrap(Box::new(|| {
        go_home();
    }) as Box<dyn Fn()>);

    js_sys::Reflect::set(&window, &"go_to_lesson".into(), go_to_lesson_fn.as_ref())?;
    js_sys::Reflect::set(&window, &"go_home".into(), go_home_fn.as_ref())?;

    go_to_lesson_fn.forget();
    go_home_fn.forget();

    Ok(())
}

/// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    expose_to_window()?;

    let app = App::new()?;
    app.render_home()?;

    web_sys::console::log_1(&"Learn platform initialized".into());
    Ok(())
}

/// Navigate to lesson (called from JS)
#[wasm_bindgen]
pub fn go_to_lesson(idx: usize) {
    // Stop any running demo
    demo_runner::stop_demo();

    if let Ok(renderer) = LessonRenderer::new("app") {
        if let Some(lesson) = LESSONS.get(idx) {
            let _ = renderer.render_lesson(lesson);

            // Start canvas demo for all lessons
            let closure = wasm_bindgen::closure::Closure::once_into_js(move || {
                if let Err(e) = LinearRegressionDemoRunner::start("lesson-canvas", 42) {
                    web_sys::console::error_1(&e);
                }
            });
            let _ = web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    50,
                );
        }
    }
}

/// Go back to home
#[wasm_bindgen]
pub fn go_home() {
    // Stop any running demo
    demo_runner::stop_demo();

    if let Ok(renderer) = LessonRenderer::new("app") {
        let _ = renderer.render_home(LESSONS);
    }
}
