//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: render.rs | UBUNTU/src/render.rs
//! PURPOSE: DOM rendering for Ubuntu lessons
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → UBUNTU
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
                <h1>Ubuntu Linux</h1>
                <p class="subtitle">System Administration & Permissions</p>
            </header>
            <section class="phase">
                <h2>Filesystem & Permissions</h2>
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

        // Terminal-based demo for lesson 0
        let demo_section = if lesson.id == 0 {
            r#"
            <section class="terminal-section">
                <h3>Interactive Terminal</h3>
                <div class="terminal" id="terminal">
                    <div class="terminal-output" id="terminal-output"></div>
                    <div class="terminal-input-line">
                        <span class="terminal-prompt" id="terminal-prompt">user@ubuntu:~$ </span>
                        <input type="text" id="terminal-input" class="terminal-input" autocomplete="off" spellcheck="false" autofocus>
                    </div>
                </div>
                <div class="terminal-hints">
                    <p>Try: <code>ls -l</code>, <code>cat readme.txt</code>, <code>chmod 777 readme.txt</code>, <code>su root</code>, <code>help</code></p>
                </div>
            </section>
            "#.to_string()
        } else {
            r#"<p class="canvas-hint">Coming soon: interactive terminal</p>"#.to_string()
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

                    {}
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
            demo_section,
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
