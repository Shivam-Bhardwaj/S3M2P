//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: render.rs | SLAM/src/render.rs
//! PURPOSE: DOM rendering for SLAM lessons
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → SLAM
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::lessons::Lesson;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};

pub struct LessonRenderer {
    #[allow(dead_code)]
    document: Document,
    root: Element,
}

impl LessonRenderer {
    pub fn new(root_id: &str) -> Result<Self, JsValue> {
        let document = web_sys::window()
            .ok_or("No window")?
            .document()
            .ok_or("No document")?;

        let root = document
            .get_element_by_id(root_id)
            .ok_or("Root not found")?;

        Ok(Self { document, root })
    }

    pub fn render_home(&self, lessons: &[Lesson]) -> Result<(), JsValue> {
        let mut html = String::from(
            r#"
            <header class="hero">
                <h1>SLAM</h1>
                <p class="subtitle">Simultaneous Localization and Mapping</p>
            </header>
            <section class="phase">
                <h2>Localization & Mapping</h2>
                <div class="lesson-grid">
        "#,
        );

        for lesson in lessons {
            html.push_str(&format!(
                r#"
                <div class="lesson-card" onclick="go_to_lesson({})">
                    <span class="lesson-icon">{}</span>
                    <h3>{}</h3>
                    <p class="lesson-subtitle">{}</p>
                </div>
            "#,
                lesson.id, lesson.icon, lesson.title, lesson.subtitle
            ));
        }

        html.push_str(
            r#"
                </div>
            </section>
            <footer>
                <a href="https://too.foo">← back to too.foo</a>
            </footer>
        "#,
        );

        self.root.set_inner_html(&html);
        Ok(())
    }

    pub fn render_lesson(&self, lesson: &Lesson) -> Result<(), JsValue> {
        let concepts_html: String = lesson
            .key_concepts
            .iter()
            .map(|c| format!(r#"<span class="concept">{}</span>"#, c))
            .collect::<Vec<_>>()
            .join("");

        // Demo controls for specific lessons
        let demo_controls = if lesson.id == 0 {
            // Particle Filter controls
            r#"
            <div class="demo-controls" id="demo-controls">
                <div class="control-row">
                    <label>Particles: <span id="particles-value">100</span></label>
                    <input type="range" id="particles-slider" min="10" max="500" step="10" value="100">
                </div>
                <div class="control-row">
                    <label>Motion Noise: <span id="motion-value">0.02</span></label>
                    <input type="range" id="motion-slider" min="0" max="0.2" step="0.01" value="0.02">
                </div>
                <div class="control-row">
                    <label>Sensor Noise: <span id="sensor-value">0.05</span></label>
                    <input type="range" id="sensor-slider" min="0.01" max="0.3" step="0.01" value="0.05">
                </div>
                <div class="control-buttons">
                    <button id="reset-btn" class="demo-btn">🔄 Reset</button>
                    <button id="pause-btn" class="demo-btn">⏸ Pause</button>
                </div>
            </div>
            "#.to_string()
        } else {
            r#"<p class="canvas-hint">Coming soon: interactive visualization</p>"#.to_string()
        };

        let html = format!(
            r#"
            <article class="lesson-view">
                <nav class="lesson-nav">
                    <button onclick="go_home()" class="back-btn">← All Lessons</button>
                </nav>

                <header class="lesson-header">
                    <span class="lesson-icon-large">{}</span>
                    <div>
                        <h1>{}</h1>
                        <p class="subtitle">{}</p>
                    </div>
                </header>

                <div class="lesson-content">
                    <section class="description">
                        <p>{}</p>
                    </section>

                    <section class="intuition">
                        <h3>Intuition</h3>
                        <p>{}</p>
                    </section>

                    <section class="concepts">
                        <h3>Key Concepts</h3>
                        <div class="concept-list">{}</div>
                    </section>

                    <section class="visualization">
                        <h3>Interactive Demo</h3>
                        <canvas id="lesson-canvas" width="600" height="400"></canvas>
                        {}
                    </section>
                </div>

                <nav class="lesson-footer">
                    {}
                    {}
                </nav>
            </article>
        "#,
            lesson.icon,
            lesson.title,
            lesson.subtitle,
            lesson.description,
            lesson.intuition,
            concepts_html,
            demo_controls,
            if lesson.id > 0 {
                format!(
                    r#"<button onclick="go_to_lesson({})" class="nav-btn">← Previous</button>"#,
                    lesson.id - 1
                )
            } else {
                String::from(r#"<span></span>"#)
            },
            if lesson.id < 3 {
                format!(
                    r#"<button onclick="go_to_lesson({})" class="nav-btn">Next →</button>"#,
                    lesson.id + 1
                )
            } else {
                String::from(r#"<span></span>"#)
            },
        );

        self.root.set_inner_html(&html);
        Ok(())
    }
}
