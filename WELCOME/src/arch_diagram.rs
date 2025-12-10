//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: arch_diagram.rs | WELCOME/src/arch_diagram.rs
//! PURPOSE: Architecture diagram renderer with iframe integration for external micro-frontend
//! MODIFIED: 2025-12-09
//! LAYER: WELCOME (landing)
//! ═══════════════════════════════════════════════════════════════════════════════

use web_sys::Document;

/// Render the architecture diagram (via iframe to external micro-frontend)
pub fn render_architecture_diagram(document: &Document) {
    let container = document
        .get_element_by_id("arch-container")
        .expect("Architecture container not found");

    // Clear existing content
    container.set_inner_html("");

    // Determine environment URL
    let window = web_sys::window().unwrap();
    let hostname = window.location().hostname().unwrap_or_default();

    // In development (localhost), we assume ARCH is running on port 8087
    // In production, we use the subdomain
    let src_url = if hostname == "localhost" || hostname == "127.0.0.1" {
        "http://127.0.0.1:8087"
    } else {
        "https://arch.too.foo"
    };

    // Create Iframe
    let iframe = document.create_element("iframe").unwrap();
    iframe.set_attribute("src", src_url).unwrap();
    iframe
        .set_attribute("style", "width: 100%; height: 100%; border: none;")
        .unwrap();
    iframe
        .set_attribute("title", "Architecture Diagram")
        .unwrap();

    // Add loading indicator? The container background (rgba(5, 5, 8, 0.95)) is dark enough.

    container.append_child(&iframe).unwrap();
}
