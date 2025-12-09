use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, MouseEvent, WheelEvent};

pub struct NodeMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub complexity: &'static str,
    pub status: &'static str, // "Active", "Deprecated", "Experimental", "Ghost"
}

struct DiagramState {
    scale: f64,
    translate_x: f64,
    translate_y: f64,
    is_dragging: bool,
    last_mouse_x: f64,
    last_mouse_y: f64,
    registry: HashMap<&'static str, NodeMetadata>,
    _selected_node: Option<&'static str>,
}

impl DiagramState {
    fn new() -> Self {
        let mut registry = HashMap::new();
        populate_registry(&mut registry);
        Self {
            scale: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            is_dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            registry,
            _selected_node: None,
        }
    }
}

fn populate_registry(map: &mut HashMap<&'static str, NodeMetadata>) {
    // =========================================================
    // 1. FOUNDATION CLUSTER (Base Utilities)
    // =========================================================
    map.insert(
        "DNA",
        NodeMetadata {
            name: "DNA (Crate Root)",
            description: "The library entry point. Re-exports modules for the Antimony system.",
            complexity: "O(1)",
            status: "Active",
        },
    );
    map.insert(
        "Random",
        NodeMetadata {
            name: "random.rs",
            description: "Deterministic PCG-based RNG for seeded simulations.",
            complexity: "O(1)",
            status: "Active",
        },
    );
    map.insert(
        "Color",
        NodeMetadata {
            name: "color.rs",
            description: "HSL/RGB color space conversions and palette generation.",
            complexity: "O(1)",
            status: "Active",
        },
    );
    map.insert(
        "Mat2",
        NodeMetadata {
            name: "mat2.rs",
            description: "2x2 Matrix operations for 2D rotations and transforms.",
            complexity: "O(1)",
            status: "Active",
        },
    );
    map.insert(
        "Stats",
        NodeMetadata {
            name: "statistics.rs",
            description: "Statistical analysis tools (Mean, Variance, Bell Curves).",
            complexity: "O(n)",
            status: "Active",
        },
    );

    // =========================================================
    // 2. SPACE CLUSTER (Coordinates & Grid)
    // =========================================================
    map.insert(
        "Grid",
        NodeMetadata {
            name: "spatial.rs (SpatialGrid)",
            description: "Hashed grid partition for O(1) mostly-uniform density spatial queries.",
            complexity: "O(1) Avg",
            status: "Active",
        },
    );
    map.insert(
        "Coords",
        NodeMetadata {
            name: "coordinates.rs",
            description: "Coordinate system transforms (Cartesian <-> Polar <-> Screen).",
            complexity: "O(1)",
            status: "Active",
        },
    );
    map.insert(
        "Zones",
        NodeMetadata {
            name: "zones.rs",
            description: "Definition of simulation boundaries and 'Chakravyu' safe zones.",
            complexity: "O(1)",
            status: "Active",
        },
    );

    // =========================================================
    // 3. PHYSICS CLUSTER (Forces & Fields)
    // =========================================================
    map.insert(
        "Inter",
        NodeMetadata {
            name: "interaction.rs",
            description: "Force calculations: Gravity, Repulsion, Friction.",
            complexity: "O(1)",
            status: "Active",
        },
    );
    map.insert(
        "Helio",
        NodeMetadata {
            name: "heliosphere_model.rs",
            description: "Solar wind and magnetic field pressure simulation models.",
            complexity: "O(n)",
            status: "Active",
        },
    );
    map.insert(
        "Wind",
        NodeMetadata {
            name: "solar_wind.rs",
            description: "Vector field generation for charged particle flow.",
            complexity: "O(1) Per Particle",
            status: "Active",
        },
    );
    map.insert(
        "Chladni",
        NodeMetadata {
            name: "chladni.rs (Sim)",
            description: "Nodal pattern solver for vibrating plates (Cymatics).",
            complexity: "O(w*h)",
            status: "Active",
        },
    );

    // =========================================================
    // 4. ALGORITHMS CLUSTER (Logic & Agents)
    // =========================================================
    map.insert(
        "Boids",
        NodeMetadata {
            name: "BoidArena",
            description: "Flocking simulation (Separation/Alignment/Cohesion).",
            complexity: "O(n log n)",
            status: "Active",
        },
    );
    map.insert(
        "EKF",
        NodeMetadata {
            name: "ekf.rs",
            description: "Extended Kalman Filter for sensor fusion/state estimation.",
            complexity: "O(k^3)",
            status: "Experimental",
        },
    );
    map.insert(
        "Path",
        NodeMetadata {
            name: "pathfinding.rs",
            description: "A* and Flow Field pathfinding algorithms.",
            complexity: "O(V+E)",
            status: "Active",
        },
    );
    map.insert(
        "L-Sys",
        NodeMetadata {
            name: "genetics/lsystem",
            description: "Procedural generation grammars.",
            complexity: "O(k^n)",
            status: "Active",
        },
    );
    map.insert(
        "SPICE",
        NodeMetadata {
            name: "spice/bridge",
            description: "NASA SPICE toolkit integration for high-precision astronomy.",
            complexity: "O(1)",
            status: "Experimental",
        },
    );

    // =========================================================
    // 5. GHOSTS (Obsolete/Legacy)
    // =========================================================
    map.insert(
        "Compute",
        NodeMetadata {
            name: "src/compute",
            description: "Legacy Compute Shader pipelines. Replaced by WebGL.",
            complexity: "N/A",
            status: "Ghost",
        },
    );
    map.insert(
        "Schema",
        NodeMetadata {
            name: "src/schema",
            description: "Cap'n Proto schemas. Removed in v0.4.",
            complexity: "N/A",
            status: "Ghost",
        },
    );

    // =========================================================
    // 6. APPS & CONSUMERS
    // =========================================================
    map.insert(
        "HELIOS",
        NodeMetadata {
            name: "HELIOS App",
            description: "Solar System Visualization & Orrery.",
            complexity: "App",
            status: "Active",
        },
    );
    map.insert(
        "SIMS",
        NodeMetadata {
            name: "SIMS App",
            description: "General Simulation Playground.",
            complexity: "App",
            status: "Active",
        },
    );
    map.insert(
        "TOOLS",
        NodeMetadata {
            name: "TOOLS Module",
            description: "Engineering Utilities (PLL, Sensors).",
            complexity: "App",
            status: "Active",
        },
    );
    map.insert(
        "LEARN",
        NodeMetadata {
            name: "LEARN",
            description: "Educational Modules.",
            complexity: "Content",
            status: "Active",
        },
    );
    map.insert(
        "BLOG",
        NodeMetadata {
            name: "BLOG",
            description: "Technical Articles.",
            complexity: "Content",
            status: "Active",
        },
    );
    map.insert(
        "ABOUT",
        NodeMetadata {
            name: "ABOUT",
            description: "User Profile.",
            complexity: "Content",
            status: "Active",
        },
    );
    map.insert(
        "X",
        NodeMetadata {
            name: "X",
            description: "Social Link.",
            complexity: "Link",
            status: "Active",
        },
    );
}

pub fn render_architecture_diagram(document: &Document) {
    let container = document
        .get_element_by_id("arch-container")
        .expect("Architecture container not found");

    // Clear existing content
    container.set_inner_html("");

    // Add Info Panel Container (Hidden by default)
    let info_panel = document.create_element("div").unwrap();
    info_panel.set_id("arch-info-panel");
    info_panel.set_attribute("style", "position: absolute; top: 20px; right: 20px; width: 300px; background: rgba(5,5,10,0.95); border: 1px solid #4cc9f0; padding: 20px; border-radius: 8px; color: white; display: none; box-shadow: 0 0 20px rgba(76, 201, 240, 0.2); font-family: 'Courier New'; z-index: 1000; pointer-events: none;").unwrap();
    container.append_child(&info_panel).unwrap();

    // Create SVG
    let svg_ns = "http://www.w3.org/2000/svg";
    let svg = document.create_element_ns(Some(svg_ns), "svg").unwrap();
    svg.set_attribute("width", "100%").unwrap();
    svg.set_attribute("height", "100%").unwrap();
    svg.set_attribute("style", "cursor: grab; background: #050508;")
        .unwrap();

    // Group that will be transformed
    let content_group = document.create_element_ns(Some(svg_ns), "g").unwrap();
    content_group.set_id("arch-content");
    svg.append_child(&content_group).unwrap();

    // Styles
    let style = document.create_element_ns(Some(svg_ns), "style").unwrap();
    style.set_text_content(Some(
        r#"
        .arch-node { fill: rgba(10, 15, 20, 0.9); stroke-width: 2; transition: all 0.2s; cursor: pointer; }
        .arch-node:hover { filter: drop-shadow(0 0 10px currentColor); stroke-width: 3; }
        .arch-ghost { stroke-dasharray: 4,4; opacity: 0.5; fill: rgba(50,50,50,0.2); }
        .arch-text { font-family: 'Courier New', monospace; text-anchor: middle; fill: white; pointer-events: none; }
        .arch-label { font-weight: bold; font-size: 14px; }
        .arch-sub { font-size: 10px; opacity: 0.7; }
        .arch-conn { stroke-width: 1; stroke-dasharray: 4,4; opacity: 0.5; fill: none; }
        "#,
    ));
    svg.append_child(&style).unwrap();

    // State management
    let state = Rc::new(RefCell::new(DiagramState::new()));

    // Draw content into content_group
    draw_diagram(document, &content_group, state.clone());

    container.append_child(&svg).unwrap();

    // Center initial view
    let window = web_sys::window().unwrap();
    let win_w = window.inner_width().unwrap().as_f64().unwrap();
    let win_h = window.inner_height().unwrap().as_f64().unwrap();
    {
        let mut s = state.borrow_mut();
        s.translate_x = win_w / 2.0;
        s.translate_y = win_h / 2.0;
        update_transform(&content_group, &s);
    }

    // EVENT LISTENERS
    // 1. Wheel (Zoom)
    let state_wheel = state.clone();
    let group_wheel = content_group.clone();
    let on_wheel = Closure::wrap(Box::new(move |e: WheelEvent| {
        e.prevent_default();
        let mut s = state_wheel.borrow_mut();
        let delta = -e.delta_y() * 0.001;
        let zoom_factor = 1.0 + delta;
        let new_scale = (s.scale * zoom_factor).clamp(0.1, 5.0);
        s.scale = new_scale;
        update_transform(&group_wheel, &s);
    }) as Box<dyn FnMut(_)>);
    svg.add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())
        .unwrap();
    on_wheel.forget();

    // 2. Mouse Down (Start Pan)
    let state_down = state.clone();
    let svg_down = svg.clone();
    let on_mousedown = Closure::wrap(Box::new(move |e: MouseEvent| {
        let mut s = state_down.borrow_mut();
        s.is_dragging = true;
        s.last_mouse_x = e.client_x() as f64;
        s.last_mouse_y = e.client_y() as f64;
        svg_down
            .set_attribute("style", "cursor: grabbing; background: #050508;")
            .unwrap();
    }) as Box<dyn FnMut(_)>);
    svg.add_event_listener_with_callback("mousedown", on_mousedown.as_ref().unchecked_ref())
        .unwrap();
    on_mousedown.forget();

    // 3. Mouse Move (Pan)
    let state_move = state.clone();
    let group_move = content_group.clone();
    let on_mousemove = Closure::wrap(Box::new(move |e: MouseEvent| {
        let mut s = state_move.borrow_mut();
        if s.is_dragging {
            e.prevent_default();
            let dx = e.client_x() as f64 - s.last_mouse_x;
            let dy = e.client_y() as f64 - s.last_mouse_y;
            s.translate_x += dx;
            s.translate_y += dy;
            s.last_mouse_x = e.client_x() as f64;
            s.last_mouse_y = e.client_y() as f64;
            update_transform(&group_move, &s);
        }
    }) as Box<dyn FnMut(_)>);
    svg.add_event_listener_with_callback("mousemove", on_mousemove.as_ref().unchecked_ref())
        .unwrap();
    on_mousemove.forget();

    // 4. Mouse Up/Leave (Stop Pan)
    let state_up = state.clone();
    let svg_up = svg.clone();
    let on_mouseup = Closure::wrap(Box::new(move |_e: MouseEvent| {
        let mut s = state_up.borrow_mut();
        s.is_dragging = false;
        svg_up
            .set_attribute("style", "cursor: grab; background: #050508;")
            .unwrap();
    }) as Box<dyn FnMut(_)>);
    svg.add_event_listener_with_callback("mouseup", on_mouseup.as_ref().unchecked_ref())
        .unwrap();
    svg.add_event_listener_with_callback("mouseleave", on_mouseup.as_ref().unchecked_ref())
        .unwrap();
    on_mouseup.forget();
}

fn update_transform(element: &Element, state: &DiagramState) {
    let transform = format!(
        "translate({}, {}) scale({})",
        state.translate_x, state.translate_y, state.scale
    );
    element.set_attribute("transform", &transform).unwrap();
}

fn update_info_panel(document: &Document, metadata: &NodeMetadata) {
    if let Some(panel) = document.get_element_by_id("arch-info-panel") {
        panel.set_inner_html(&format!(
            "<h3 style='margin-top:0; border-bottom: 1px solid #333; padding-bottom: 10px; color: #4cc9f0;'>{}</h3>
             <p style='font-size: 14px; line-height: 1.4; color: #ddd;'>{}</p>
             <div style='margin-top: 15px; font-size: 12px; color: #888;'>
                 <div><strong>Complexity:</strong> <span style='color: #f72585;'>{}</span></div>
                 <div><strong>Status:</strong> <span style='color: {};'>{}</span></div>
             </div>
            ",
            metadata.name,
            metadata.description,
            metadata.complexity,
            if metadata.status.contains("Active") { "#4cc9f0" } else { "#f72585" },
            metadata.status
        ));
        panel.set_attribute("style", "position: absolute; top: 20px; right: 20px; width: 300px; background: rgba(5,5,10,0.95); border: 1px solid #4cc9f0; padding: 20px; border-radius: 8px; color: white; display: block; box-shadow: 0 0 20px rgba(76, 201, 240, 0.2); font-family: 'Courier New'; z-index: 1000; animation: fadeIn 0.2s ease-out;").unwrap();
    }
}

fn draw_diagram(document: &Document, root: &Element, state: Rc<RefCell<DiagramState>>) {
    // ==========================================
    // LEVEL 1: DNA CORE (THE SOURCE) - Top
    // ==========================================
    let dna_y = -250.0;

    // Central Node: DNA
    draw_node(
        document,
        root,
        state.clone(),
        0.0,
        dna_y,
        60.0,
        "DNA",
        "Core Engine",
        "#ff0080",
        false,
    );

    // DNA Sub-systems
    draw_node(
        document,
        root,
        state.clone(),
        -80.0,
        dna_y - 40.0,
        25.0,
        "Grid",
        "Spatial",
        "#ff0080",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        80.0,
        dna_y - 40.0,
        25.0,
        "L-Sys",
        "Gen",
        "#ff0080",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        -100.0,
        dna_y + 40.0,
        25.0,
        "Boids",
        "Sim",
        "#ff0080",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        100.0,
        dna_y + 40.0,
        25.0,
        "Chladni",
        "Phys",
        "#ff0080",
        false,
    );

    // Advanced Layer
    draw_node(
        document,
        root,
        state.clone(),
        0.0,
        dna_y - 90.0,
        25.0,
        "Fungal",
        "Net",
        "#ff0080",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        -140.0,
        dna_y,
        25.0,
        "EKF",
        "Est",
        "#ff0080",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        140.0,
        dna_y,
        25.0,
        "SPICE",
        "Nav",
        "#ff0080",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        0.0,
        dna_y + 90.0,
        25.0,
        "Path",
        "A*",
        "#ff0080",
        false,
    );

    // Ghost Nodes (Obsolescence)
    draw_node(
        document,
        root,
        state.clone(),
        -180.0,
        dna_y - 80.0,
        20.0,
        "Compute",
        "Ghost",
        "#666",
        true,
    );
    draw_node(
        document,
        root,
        state.clone(),
        180.0,
        dna_y - 80.0,
        20.0,
        "Schema",
        "Ghost",
        "#666",
        true,
    );

    // Connections within DNA Cluster
    draw_conn(document, root, 0.0, dna_y, -80.0, dna_y - 40.0, "#ff0080");
    draw_conn(document, root, 0.0, dna_y, 80.0, dna_y - 40.0, "#ff0080");
    draw_conn(document, root, 0.0, dna_y, -100.0, dna_y + 40.0, "#ff0080");
    draw_conn(document, root, 0.0, dna_y, 100.0, dna_y + 40.0, "#ff0080");
    draw_conn(document, root, 0.0, dna_y, 0.0, dna_y - 90.0, "#ff0080");
    draw_conn(document, root, 0.0, dna_y, -140.0, dna_y, "#ff0080");
    draw_conn(document, root, 0.0, dna_y, 140.0, dna_y, "#ff0080");
    draw_conn(document, root, 0.0, dna_y, 0.0, dna_y + 90.0, "#ff0080");
    // Ghost connections
    draw_conn(document, root, 0.0, dna_y, -180.0, dna_y - 80.0, "#444");
    draw_conn(document, root, 0.0, dna_y, 180.0, dna_y - 80.0, "#444");

    // ==========================================
    // LEVEL 2: ACTIVE APPLICATIONS
    // ==========================================
    let app_y = 0.0;

    let h_x = -250.0;
    draw_node(
        document,
        root,
        state.clone(),
        h_x,
        app_y,
        45.0,
        "HELIOS",
        "Solar Sim",
        "#4cc9f0",
        false,
    );
    draw_conn(
        document,
        root,
        -60.0,
        dna_y + 40.0,
        h_x,
        app_y - 45.0,
        "#555555",
    );

    let s_x = 0.0;
    draw_node(
        document,
        root,
        state.clone(),
        s_x,
        app_y + 50.0,
        55.0,
        "SIMS",
        "Playground",
        "#4cc9f0",
        false,
    );
    draw_conn(
        document,
        root,
        0.0,
        dna_y + 60.0,
        s_x,
        app_y + 50.0 - 55.0,
        "#555555",
    );

    let t_x = 250.0;
    draw_node(
        document,
        root,
        state.clone(),
        t_x,
        app_y,
        45.0,
        "TOOLS",
        "Utils",
        "#4361ee",
        false,
    );
    draw_conn(
        document,
        root,
        60.0,
        dna_y + 40.0,
        t_x,
        app_y - 45.0,
        "#555555",
    );

    // ==========================================
    // LEVEL 3: KNOWLEDGE & SOCIAL
    // ==========================================
    let leaf_y = 250.0;

    draw_node(
        document,
        root,
        state.clone(),
        -300.0,
        leaf_y,
        35.0,
        "LEARN",
        "AI/ML",
        "#7209b7",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        -100.0,
        leaf_y,
        35.0,
        "BLOG",
        "Articles",
        "#7209b7",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        100.0,
        leaf_y,
        35.0,
        "ABOUT",
        "Profile",
        "#7209b7",
        false,
    );
    draw_node(
        document,
        root,
        state.clone(),
        300.0,
        leaf_y,
        35.0,
        "X",
        "Social",
        "#f72585",
        false,
    );

    draw_conn(
        document,
        root,
        s_x,
        app_y + 105.0,
        -300.0,
        leaf_y - 35.0,
        "#333333",
    );
    draw_conn(
        document,
        root,
        s_x,
        app_y + 105.0,
        -100.0,
        leaf_y - 35.0,
        "#333333",
    );
    draw_conn(
        document,
        root,
        s_x,
        app_y + 105.0,
        100.0,
        leaf_y - 35.0,
        "#333333",
    );
    draw_conn(
        document,
        root,
        s_x,
        app_y + 105.0,
        300.0,
        leaf_y - 35.0,
        "#333333",
    );

    // ==========================================
    // USAGE FLOW
    // ==========================================
    draw_usage_flow_curve(
        document,
        root,
        -100.0,
        dna_y + 40.0,
        s_x - 30.0,
        app_y + 50.0 - 30.0,
        "#ff0080",
        50.0,
    );
    draw_usage_flow_curve(
        document,
        root,
        100.0,
        dna_y + 40.0,
        s_x + 30.0,
        app_y + 50.0 - 30.0,
        "#ff0080",
        -50.0,
    );
    draw_usage_flow_curve(
        document,
        root,
        140.0,
        dna_y,
        h_x + 30.0,
        app_y - 30.0,
        "#ff0080",
        50.0,
    );
    draw_usage_flow_curve(
        document,
        root,
        -80.0,
        dna_y - 40.0,
        h_x,
        app_y - 45.0,
        "#ff0080",
        -150.0,
    );
    draw_usage_flow_curve(
        document,
        root,
        -140.0,
        dna_y,
        t_x - 30.0,
        app_y - 30.0,
        "#4361ee",
        -50.0,
    );
    draw_usage_flow_curve(
        document,
        root,
        80.0,
        dna_y - 40.0,
        -300.0,
        leaf_y - 35.0,
        "#ff0080",
        100.0,
    );
}

fn draw_usage_flow_curve(
    document: &Document,
    parent: &Element,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: &str,
    curve_offset: f64,
) {
    let ns = "http://www.w3.org/2000/svg";
    let path = document.create_element_ns(Some(ns), "path").unwrap();
    let mid_x = (x1 + x2) / 2.0 + curve_offset;
    let mid_y = (y1 + y2) / 2.0;

    let d = format!("M {} {} Q {} {} {} {}", x1, y1, mid_x, mid_y, x2, y2);
    path.set_attribute("d", &d).unwrap();
    path.set_attribute("stroke", color).unwrap();
    path.set_attribute("stroke-width", "2").unwrap();
    path.set_attribute("fill", "none").unwrap();
    path.set_attribute("stroke-dasharray", "8,8").unwrap();
    path.set_attribute("opacity", "0.8").unwrap();

    let anim = document.create_element_ns(Some(ns), "animate").unwrap();
    anim.set_attribute("attributeName", "stroke-dashoffset")
        .unwrap();
    anim.set_attribute("from", "32").unwrap();
    anim.set_attribute("to", "0").unwrap();
    anim.set_attribute("dur", "2s").unwrap();
    anim.set_attribute("repeatCount", "indefinite").unwrap();

    path.append_child(&anim).unwrap();
    parent.append_child(&path).unwrap();
}

fn draw_node(
    document: &Document,
    parent: &Element,
    state: Rc<RefCell<DiagramState>>,
    x: f64,
    y: f64,
    r: f64,
    title: &str,
    sub: &str,
    color: &str,
    is_ghost: bool,
) {
    let ns = "http://www.w3.org/2000/svg";
    let group = document.create_element_ns(Some(ns), "g").unwrap();

    // Circle
    let circle = document.create_element_ns(Some(ns), "circle").unwrap();
    circle.set_attribute("cx", &x.to_string()).unwrap();
    circle.set_attribute("cy", &y.to_string()).unwrap();
    circle.set_attribute("r", &r.to_string()).unwrap();

    if is_ghost {
        circle
            .set_attribute("class", "arch-node arch-ghost")
            .unwrap();
    } else {
        circle.set_attribute("class", "arch-node").unwrap();
    }

    circle.set_attribute("fill", "#0a0a0f").unwrap();
    circle.set_attribute("stroke", color).unwrap();
    group.append_child(&circle).unwrap();

    // Text
    let text_title = document.create_element_ns(Some(ns), "text").unwrap();
    text_title.set_attribute("x", &x.to_string()).unwrap();
    text_title
        .set_attribute("y", &(y - 5.0).to_string())
        .unwrap();
    text_title
        .set_attribute("class", "arch-text arch-label")
        .unwrap();
    text_title
        .set_attribute("fill", if is_ghost { "#666" } else { color })
        .unwrap();
    text_title.set_text_content(Some(title));
    group.append_child(&text_title).unwrap();

    let text_sub = document.create_element_ns(Some(ns), "text").unwrap();
    text_sub.set_attribute("x", &x.to_string()).unwrap();
    text_sub
        .set_attribute("y", &(y + 10.0).to_string())
        .unwrap();
    text_sub
        .set_attribute("class", "arch-text arch-sub")
        .unwrap();
    text_sub
        .set_attribute("fill", if is_ghost { "#444" } else { "#aaaaaa" })
        .unwrap();
    text_sub.set_text_content(Some(sub));
    group.append_child(&text_sub).unwrap();

    parent.append_child(&group).unwrap();

    // CLICK HANDLER
    let title_owned = title.to_string(); // Static string lifetime trick might be needed or just clone
                                         // Since we used static in the struct but title here is &str, we rely on the key matching.
                                         // Ideally we'd use string ownership but for now we trust the static strings map.

    let click_cb = Closure::wrap(Box::new(move |e: MouseEvent| {
        e.stop_propagation(); // Don't drag map
        let s = state.borrow();
        // Lookup metadata
        // Note: The map keys must match 'title' exactly.
        if let Some(meta) = s.registry.get(title_owned.as_str()) {
            // We can't actually pass 'document' into here easily because it's not clonable for closures easily without setup.
            // BUT, we can get the global window -> document.
            let window = web_sys::window().unwrap();
            let doc = window.document().unwrap();
            update_info_panel(&doc, meta);
        } else {
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "No metadata for {}",
                title_owned
            )));
        }
    }) as Box<dyn FnMut(_)>);

    group
        .add_event_listener_with_callback("mousedown", click_cb.as_ref().unchecked_ref())
        .unwrap();
    click_cb.forget();
}

fn draw_conn(
    document: &Document,
    parent: &Element,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: &str,
) {
    let ns = "http://www.w3.org/2000/svg";
    let line = document.create_element_ns(Some(ns), "line").unwrap();
    line.set_attribute("x1", &x1.to_string()).unwrap();
    line.set_attribute("y1", &y1.to_string()).unwrap();
    line.set_attribute("x2", &x2.to_string()).unwrap();
    line.set_attribute("y2", &y2.to_string()).unwrap();
    line.set_attribute("stroke", color).unwrap();
    line.set_attribute("class", "arch-conn").unwrap();
    parent.append_child(&line).unwrap();
}
