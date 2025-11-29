// DOM rendering for lessons

use crate::lessons::Lesson;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};

pub struct LessonRenderer {
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
        let phases = [
            "Foundations",
            "Deep Learning",
            "Reinforcement Learning",
            "Towards AGI",
        ];

        let mut html = String::from(
            r#"
            <header class="hero">
                <h1>Zero to AGI</h1>
                <p class="subtitle">Learn machine learning from scratch, implemented in Rust</p>
            </header>
        "#,
        );

        for phase in phases {
            let phase_lessons: Vec<_> = lessons.iter().filter(|l| l.phase == phase).collect();

            if phase_lessons.is_empty() {
                continue;
            }

            html.push_str(&format!(
                r#"
                <section class="phase">
                    <h2>{}</h2>
                    <div class="lesson-grid">
            "#,
                phase
            ));

            for lesson in phase_lessons {
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

            html.push_str("</div></section>");
        }

        html.push_str(
            r#"
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

        let math_html = if lesson.math.is_empty() {
            String::new()
        } else {
            format!(
                r#"
                <div class="math-section">
                    <h3>Mathematics</h3>
                    <div class="math" data-formula="{}">{}</div>
                </div>
            "#,
                lesson.math, lesson.math
            )
        };

        let html = format!(
            r#"
            <article class="lesson-view">
                <nav class="lesson-nav">
                    <button onclick="go_home()" class="back-btn">← All Lessons</button>
                    <span class="phase-badge">{}</span>
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

                    {}

                    <section class="concepts">
                        <h3>Key Concepts</h3>
                        <div class="concept-list">{}</div>
                    </section>

                    <section class="visualization">
                        <h3>Interactive Demo</h3>
                        <canvas id="lesson-canvas" width="600" height="400"></canvas>
                        <p class="canvas-hint">Coming soon: interactive visualization</p>
                    </section>
                </div>

                <nav class="lesson-footer">
                    {}
                    {}
                </nav>
            </article>
        "#,
            lesson.phase,
            lesson.icon,
            lesson.title,
            lesson.subtitle,
            lesson.description,
            lesson.intuition,
            math_html,
            concepts_html,
            if lesson.id > 0 {
                format!(
                    r#"<button onclick="go_to_lesson({})" class="nav-btn">← Previous</button>"#,
                    lesson.id - 1
                )
            } else {
                String::from(r#"<span></span>"#)
            },
            if lesson.id < 11 {
                format!(
                    r#"<button onclick="go_to_lesson({})" class="nav-btn">Next →</button>"#,
                    lesson.id + 1
                )
            } else {
                String::from(r#"<span></span>"#)
            },
        );

        self.root.set_inner_html(&html);

        // Trigger KaTeX rendering if available
        let _ = js_sys::eval("if(typeof renderMath === 'function') renderMath();");

        Ok(())
    }
}
