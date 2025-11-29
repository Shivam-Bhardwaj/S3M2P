// Learn - Zero to AGI Interactive Platform
// Client-side ML learning with visualizations
#![allow(unexpected_cfgs)]

use wasm_bindgen::prelude::*;

pub mod lessons;
pub mod render;

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

/// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let app = App::new()?;
    app.render_home()?;

    web_sys::console::log_1(&"Learn platform initialized".into());
    Ok(())
}

/// Navigate to lesson (called from JS)
#[wasm_bindgen]
pub fn go_to_lesson(idx: usize) {
    if let Ok(renderer) = LessonRenderer::new("app") {
        if let Some(lesson) = LESSONS.get(idx) {
            let _ = renderer.render_lesson(lesson);
        }
    }
}

/// Go back to home
#[wasm_bindgen]
pub fn go_home() {
    if let Ok(renderer) = LessonRenderer::new("app") {
        let _ = renderer.render_home(LESSONS);
    }
}
