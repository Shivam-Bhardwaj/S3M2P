//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: render.rs | SLAM/src/render.rs
//! PURPOSE: DOM rendering for SLAM lessons with intuition-first layout
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → SLAM
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::lessons::{Lesson, GLOSSARY};
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
                <p class="subtitle">From Sensor Fusion to Simultaneous Localization and Mapping</p>
            </header>
            <section class="phase">
                <h2>Learning Path</h2>
                <p class="phase-intro">Start with the fundamentals of sensor fusion, then progress to advanced SLAM algorithms.</p>
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

    /// Apply glossary tooltips - wrap technical terms with tooltip spans
    fn apply_glossary(text: &str) -> String {
        let mut result = text.to_string();
        for term in GLOSSARY {
            // Case-insensitive search and replace
            let pattern = term.word;
            if let Some(pos) = result.to_lowercase().find(&pattern.to_lowercase()) {
                let original = &result[pos..pos + pattern.len()];
                let tooltip = format!(
                    r#"<span class="term" data-tooltip="{}">{}</span>"#,
                    term.short, original
                );
                result = format!("{}{}{}", &result[..pos], tooltip, &result[pos + pattern.len()..]);
            }
        }
        result
    }

    pub fn render_lesson(&self, lesson: &Lesson) -> Result<(), JsValue> {
        // Build key takeaways list
        let takeaways_html: String = lesson
            .key_takeaways
            .iter()
            .map(|t| format!(r#"<li class="takeaway-item">{}</li>"#, t))
            .collect::<Vec<_>>()
            .join("");

        // Demo controls for each lesson (reordered: 0=Comp, 1=Kalman, 2=Particle, 3=EKF, 4=Graph)
        let demo_controls = match lesson.id {
            0 => {
                // Complementary Filter controls - BEST defaults (min noise), increase to degrade
                r#"
                <div class="demo-controls" id="demo-controls">
                    <div class="control-group">
                        <h4>Filter Parameters</h4>
                        <div class="control-row">
                            <label>Alpha (α): <span id="alpha-value">0.98</span></label>
                            <input type="range" id="alpha-slider" min="0.5" max="0.995" step="0.005" value="0.98">
                        </div>
                        <div class="control-row">
                            <label>Accel Noise: <span id="accel-noise-value">1.0</span>°</label>
                            <input type="range" id="accel-noise-slider" min="1" max="20" step="1" value="1">
                        </div>
                        <div class="control-row">
                            <label>Gyro Drift: <span id="gyro-drift-value">0.0</span>°/s</label>
                            <input type="range" id="gyro-drift-slider" min="0" max="2" step="0.1" value="0">
                        </div>
                        <div class="control-row">
                            <label>Motion Speed: <span id="motion-speed-value">0.3</span></label>
                            <input type="range" id="motion-speed-slider" min="0.1" max="1" step="0.1" value="0.3">
                        </div>
                    </div>
                    <div class="control-group">
                        <h4>Playback</h4>
                        <div class="control-buttons">
                            <button id="reset-btn" class="demo-btn">🔄 Reset</button>
                            <button id="pause-btn" class="demo-btn">⏸ Pause</button>
                        </div>
                    </div>
                    <div class="demo-hint">
                        <strong>Try this:</strong> Increase Accel Noise to see jitter. Increase Gyro Drift to see divergence.
                        Lower α to trust the jittery accelerometer more. See how each change degrades tracking!
                    </div>
                </div>
                "#.to_string()
            }
            1 => {
                // Kalman Filter controls - BEST defaults (min noise, frequent GPS), increase to degrade
                r#"
                <div class="demo-controls" id="demo-controls">
                    <div class="control-group">
                        <h4>Noise Parameters</h4>
                        <div class="control-row">
                            <label>Process Noise (Q): <span id="process-noise-value">0.01</span></label>
                            <input type="range" id="process-noise-slider" min="0.01" max="1.0" step="0.01" value="0.01">
                        </div>
                        <div class="control-row">
                            <label>Measurement Noise (R): <span id="measurement-noise-value">0.1</span></label>
                            <input type="range" id="measurement-noise-slider" min="0.1" max="2.0" step="0.1" value="0.1">
                        </div>
                        <div class="control-row">
                            <label>GPS Update Interval: <span id="gps-interval-value">1</span> frames</label>
                            <input type="range" id="gps-interval-slider" min="1" max="50" step="1" value="1">
                        </div>
                    </div>
                    <div class="control-group">
                        <h4>Playback</h4>
                        <div class="control-buttons">
                            <button id="reset-btn" class="demo-btn">🔄 Reset</button>
                            <button id="pause-btn" class="demo-btn">⏸ Pause</button>
                        </div>
                    </div>
                    <div class="demo-hint">
                        <strong>Try this:</strong> Increase Process Noise to see more drift. Increase GPS Interval
                        to see larger uncertainty growth. Increase Measurement Noise to trust GPS less.
                    </div>
                </div>
                "#.to_string()
            }
            2 => {
                // Particle Filter controls - BEST defaults (max particles, min noise), reduce/increase to degrade
                r#"
                <div class="demo-controls" id="demo-controls">
                    <div class="control-group">
                        <h4>Algorithm Parameters</h4>
                        <div class="control-row">
                            <label>Particles: <span id="particles-value">500</span></label>
                            <input type="range" id="particles-slider" min="10" max="500" step="10" value="500">
                        </div>
                        <div class="control-row">
                            <label>Motion Noise (σ): <span id="motion-value">0.000</span></label>
                            <input type="range" id="motion-slider" min="0" max="0.1" step="0.005" value="0">
                        </div>
                        <div class="control-row">
                            <label>Sensor Noise (σ): <span id="sensor-value">0.01</span></label>
                            <input type="range" id="sensor-slider" min="0.01" max="0.15" step="0.005" value="0.01">
                        </div>
                    </div>
                    <div class="control-group">
                        <h4>Playback</h4>
                        <div class="control-buttons">
                            <button id="reset-btn" class="demo-btn">🔄 Reset</button>
                            <button id="pause-btn" class="demo-btn">⏸ Pause</button>
                            <button id="step-mode-btn" class="demo-btn">👣 Step Mode</button>
                            <button id="step-btn" class="demo-btn" style="display: none">⏭ Next Step</button>
                        </div>
                    </div>
                    <div class="demo-hint">
                        <strong>Try this:</strong> Reduce Particles to see worse tracking. Increase Motion Noise
                        to watch particles spread. Increase Sensor Noise to see slower convergence.
                    </div>
                </div>
                "#.to_string()
            }
            3 => {
                // EKF SLAM controls - BEST defaults (max range, min noise), reduce/increase to degrade
                r#"
                <div class="demo-controls" id="demo-controls">
                    <div class="control-group">
                        <h4>SLAM Parameters</h4>
                        <div class="control-row">
                            <label>Sensor Range: <span id="sensor-range-value">0.6</span></label>
                            <input type="range" id="sensor-range-slider" min="0.1" max="0.6" step="0.05" value="0.6">
                        </div>
                        <div class="control-row">
                            <label>Motion Noise: <span id="motion-noise-value">0.005</span></label>
                            <input type="range" id="motion-noise-slider" min="0.005" max="0.1" step="0.005" value="0.005">
                        </div>
                        <div class="control-row">
                            <label>Observation Noise: <span id="obs-noise-value">0.01</span></label>
                            <input type="range" id="obs-noise-slider" min="0.01" max="0.2" step="0.01" value="0.01">
                        </div>
                    </div>
                    <div class="control-group">
                        <h4>Visualization</h4>
                        <div class="control-buttons">
                            <button id="reset-btn" class="demo-btn">🔄 Reset</button>
                            <button id="pause-btn" class="demo-btn">⏸ Pause</button>
                            <button id="show-cov-btn" class="demo-btn">📊 Show Covariance</button>
                        </div>
                    </div>
                    <div class="demo-hint">
                        <strong>Try this:</strong> Reduce Sensor Range to see fewer landmarks discovered.
                        Increase Motion/Observation Noise to see larger uncertainty ellipses and worse tracking.
                    </div>
                </div>
                "#.to_string()
            }
            4 => {
                // Graph SLAM controls - BEST defaults (min noise), increase to degrade
                r#"
                <div class="demo-controls" id="demo-controls">
                    <div class="control-group">
                        <h4>Graph Parameters</h4>
                        <div class="control-row">
                            <label>Odometry Noise: <span id="odom-noise-value">0.005</span></label>
                            <input type="range" id="odom-noise-slider" min="0.005" max="0.1" step="0.005" value="0.005">
                        </div>
                        <div class="control-row">
                            <label>Loop Closure Threshold: <span id="lc-threshold-value">0.12</span></label>
                            <input type="range" id="lc-threshold-slider" min="0.05" max="0.3" step="0.01" value="0.12">
                        </div>
                    </div>
                    <div class="control-group">
                        <h4>Actions</h4>
                        <div class="control-buttons">
                            <button id="reset-btn" class="demo-btn">🔄 Reset</button>
                            <button id="pause-btn" class="demo-btn">⏸ Pause</button>
                            <button id="optimize-btn" class="demo-btn">⚡ Optimize Graph</button>
                            <button id="add-lc-btn" class="demo-btn">🔗 Add Loop Closure</button>
                        </div>
                    </div>
                    <div class="demo-hint">
                        <strong>Try this:</strong> Increase Odometry Noise to see more drift accumulate.
                        Click "Optimize Graph" after loop closures to see the graph snap into consistency!
                    </div>
                </div>
                "#.to_string()
            }
            _ => r#"<p class="canvas-hint">Interactive visualization coming soon</p>"#.to_string()
        };

        // Apply glossary tooltips to intuition text
        let intuition_html = Self::apply_glossary(lesson.intuition);

        let html = format!(
            r#"
            <article class="lesson-view">
                <nav class="lesson-nav">
                    <button onclick="go_home()" class="back-btn">← All Lessons</button>
                </nav>

                <header class="lesson-header">
                    <span class="lesson-icon-large">{icon}</span>
                    <div>
                        <h1>{title}</h1>
                        <p class="subtitle">{subtitle}</p>
                    </div>
                </header>

                <div class="lesson-content">
                    <!-- 1. Why It Matters (Hook) -->
                    <section class="why-it-matters">
                        <p class="hook">{why_it_matters}</p>
                    </section>

                    <!-- 2. Intuition (No jargon, builds understanding) -->
                    <section class="intuition">
                        <h3>💡 The Idea</h3>
                        <div class="intuition-text">{intuition}</div>
                    </section>

                    <!-- 3. Interactive Demo -->
                    <section class="visualization">
                        <h3>🎮 Try It Yourself</h3>
                        <canvas id="lesson-canvas" width="800" height="500"></canvas>
                        <div class="demo-explanation">
                            <p>{demo_explanation}</p>
                        </div>
                        {controls}
                    </section>

                    <!-- 4. Key Takeaways -->
                    <section class="takeaways">
                        <h3>📝 Key Takeaways</h3>
                        <ul class="takeaway-list">{takeaways}</ul>
                    </section>

                    <!-- 5. Going Deeper (Expandable) -->
                    <details class="going-deeper">
                        <summary><h3>🔬 Going Deeper</h3></summary>
                        <p>{going_deeper}</p>
                    </details>

                    <!-- 6. Math Details (Hidden by default) -->
                    <details class="math-details">
                        <summary><h3>📐 Mathematical Details</h3></summary>
                        <pre class="math-text">{math_details}</pre>
                    </details>
                </div>

                <nav class="lesson-footer">
                    {prev_btn}
                    {next_btn}
                </nav>
            </article>
        "#,
            icon = lesson.icon,
            title = lesson.title,
            subtitle = lesson.subtitle,
            why_it_matters = lesson.why_it_matters,
            intuition = intuition_html.replace('\n', "<br>"),
            demo_explanation = lesson.demo_explanation.replace('\n', "<br>"),
            controls = demo_controls,
            takeaways = takeaways_html,
            going_deeper = lesson.going_deeper,
            math_details = lesson.math_details,
            prev_btn = if lesson.id > 0 {
                format!(
                    r#"<button onclick="go_to_lesson({})" class="nav-btn">← Previous</button>"#,
                    lesson.id - 1
                )
            } else {
                String::from(r#"<span></span>"#)
            },
            next_btn = if lesson.id < 4 {  // Updated for 5 lessons (0-4)
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
