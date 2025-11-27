// Canvas 2D Renderer - Following too.foo patterns
// No GPU required, efficient CPU rendering

use crate::simulation::{SimulationState, AU_KM, SOLAR_RADIUS_KM, ORBIT_SEGMENTS};
use std::f64::consts::PI;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

// ============================================================================
// DRAWING UTILITIES
// ============================================================================

/// Draw the entire scene
pub fn render(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let w = state.view.width;
    let h = state.view.height;

    // Clear with space background
    ctx.set_fill_style(&JsValue::from_str("#000008"));
    ctx.fill_rect(0.0, 0.0, w, h);

    // Draw layers back to front
    draw_starfield(ctx, state, time);
    draw_heliosphere_boundaries(ctx, state);
    draw_orbits(ctx, state);
    draw_missions(ctx, state, time);
    draw_sun(ctx, state, time);
    draw_planets(ctx, state, time);
    draw_ui_overlay(ctx, state);
}

// ============================================================================
// STARFIELD (Procedural, no storage)
// ============================================================================

fn draw_starfield(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let w = state.view.width;
    let h = state.view.height;

    // Pseudo-random star positions based on screen position
    // Stars parallax slowly with pan for depth effect
    let parallax = 0.1;
    let offset_x = state.view.center_x * parallax;
    let offset_y = state.view.center_y * parallax;

    ctx.set_fill_style(&JsValue::from_str("white"));

    // Generate ~200 stars procedurally
    for i in 0..200 {
        let seed = i as f64 * 17.31;
        let x = ((seed * 7.13 + offset_x * 10.0) % w + w) % w;
        let y = ((seed * 11.37 + offset_y * 10.0) % h + h) % h;

        // Brightness variation
        let brightness = 0.3 + (seed * 3.7).sin().abs() * 0.7;
        // Twinkle
        let twinkle = 0.8 + ((time * 2.0 + seed).sin() * 0.2);
        let alpha = brightness * twinkle;

        // Size based on "magnitude"
        let size = 0.5 + (seed * 2.3).sin().abs() * 1.5;

        ctx.set_global_alpha(alpha);
        ctx.begin_path();
        ctx.arc(x, y, size, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }

    ctx.set_global_alpha(1.0);
}

// ============================================================================
// HELIOSPHERE BOUNDARIES
// ============================================================================

fn draw_heliosphere_boundaries(ctx: &CanvasRenderingContext2d, state: &SimulationState) {
    let view = &state.view;

    // Only draw if zoomed out enough
    if view.zoom < 0.5 {
        return;
    }

    // Bow shock (outermost)
    draw_boundary_ellipse(ctx, state, state.bow_shock_au, "rgba(231, 76, 60, 0.1)", 1.0);

    // Heliopause
    draw_boundary_ellipse(ctx, state, state.heliopause_au, "rgba(155, 89, 182, 0.15)", 1.5);

    // Termination shock
    draw_boundary_ellipse(ctx, state, state.termination_shock_au, "rgba(52, 152, 219, 0.2)", 2.0);

    // Labels
    if view.zoom > 0.8 {
        let lod = view.lod_level();
        if lod == 0 {
            draw_boundary_label(ctx, state, state.termination_shock_au, "Termination Shock");
            draw_boundary_label(ctx, state, state.heliopause_au, "Heliopause");
            draw_boundary_label(ctx, state, state.bow_shock_au, "Bow Shock");
        }
    }
}

fn draw_boundary_ellipse(ctx: &CanvasRenderingContext2d, state: &SimulationState,
                          radius_au: f64, color: &str, line_width: f64) {
    let (cx, cy) = state.view.au_to_screen(0.0, 0.0);
    let r_pixels = radius_au / state.view.zoom;

    // Don't draw if too small or too large
    if r_pixels < 10.0 || r_pixels > state.view.width * 3.0 {
        return;
    }

    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(line_width);

    // Ellipse (stretched in tail direction)
    ctx.begin_path();
    ctx.ellipse(cx, cy, r_pixels * 0.8, r_pixels, 0.0, 0.0, 2.0 * PI).unwrap_or(());
    ctx.stroke();

    // Fill with very transparent version
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.fill();
}

fn draw_boundary_label(ctx: &CanvasRenderingContext2d, state: &SimulationState,
                        radius_au: f64, label: &str) {
    let (cx, cy) = state.view.au_to_screen(radius_au * 0.7, 0.0);

    ctx.set_font("12px monospace");
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.5)"));
    ctx.fill_text(label, cx, cy).unwrap_or(());
}

// ============================================================================
// ORBIT PATHS
// ============================================================================

fn draw_orbits(ctx: &CanvasRenderingContext2d, state: &SimulationState) {
    ctx.set_line_width(1.0);

    for p in 0..state.planet_count {
        // Visibility check - is any part of orbit on screen?
        let orbit = &state.planet_orbits[p];
        let aphelion = orbit.a * (1.0 + orbit.e);

        if !state.view.is_visible(0.0, 0.0, aphelion) {
            continue;
        }

        // Orbit color (dimmed version of planet color)
        let color = format!("{}40", state.planet_colors[p]); // 25% alpha
        ctx.set_stroke_style(&JsValue::from_str(&color));

        ctx.begin_path();

        let path = &state.orbit_paths[p];
        let (sx, sy) = state.view.au_to_screen(path[0], path[1]);
        ctx.move_to(sx, sy);

        for i in 1..ORBIT_SEGMENTS {
            let x = path[i * 2];
            let y = path[i * 2 + 1];
            let (sx, sy) = state.view.au_to_screen(x, y);
            ctx.line_to(sx, sy);
        }

        ctx.close_path();
        ctx.stroke();
    }
}

// ============================================================================
// SUN
// ============================================================================

fn draw_sun(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let view = &state.view;
    let (cx, cy) = view.au_to_screen(0.0, 0.0);

    // Sun radius in pixels (with minimum size for visibility)
    let sun_radius_au = SOLAR_RADIUS_KM / AU_KM;
    let base_radius = (sun_radius_au / view.zoom).max(8.0);

    // Pulsing corona
    let pulse = 1.0 + (time * 0.5).sin() * 0.1;
    let corona_radius = base_radius * 3.0 * pulse;

    // Outer corona glow
    let gradient = ctx.create_radial_gradient(cx, cy, base_radius, cx, cy, corona_radius).unwrap();
    gradient.add_color_stop(0.0, "rgba(255, 200, 50, 0.8)").unwrap();
    gradient.add_color_stop(0.3, "rgba(255, 150, 50, 0.4)").unwrap();
    gradient.add_color_stop(0.6, "rgba(255, 100, 50, 0.1)").unwrap();
    gradient.add_color_stop(1.0, "rgba(255, 50, 0, 0)").unwrap();

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, corona_radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Sun body
    let body_gradient = ctx.create_radial_gradient(
        cx - base_radius * 0.3, cy - base_radius * 0.3, 0.0,
        cx, cy, base_radius
    ).unwrap();
    body_gradient.add_color_stop(0.0, "#FFF5E0").unwrap();
    body_gradient.add_color_stop(0.5, "#FFD700").unwrap();
    body_gradient.add_color_stop(0.8, "#FFA500").unwrap();
    body_gradient.add_color_stop(1.0, "#FF6B00").unwrap();

    ctx.set_fill_style(&body_gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, base_radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Sunspots (procedural, time-varying)
    if base_radius > 20.0 {
        ctx.set_fill_style(&JsValue::from_str("rgba(100, 50, 0, 0.3)"));
        for i in 0..3 {
            let seed = i as f64 * 2.71;
            let angle = (time * 0.02 + seed * 2.0) % (2.0 * PI);
            let dist = base_radius * (0.3 + (seed * 1.5).sin().abs() * 0.4);
            let spot_x = cx + angle.cos() * dist;
            let spot_y = cy + angle.sin() * dist;
            let spot_r = base_radius * (0.05 + (seed * 3.0).sin().abs() * 0.1);

            ctx.begin_path();
            ctx.arc(spot_x, spot_y, spot_r, 0.0, 2.0 * PI).unwrap_or(());
            ctx.fill();
        }
    }

    // Label
    if view.zoom < 0.05 {
        ctx.set_font("bold 14px sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#FFD700"));
        ctx.fill_text("Sun", cx + base_radius + 5.0, cy + 5.0).unwrap_or(());
    }
}

// ============================================================================
// PLANETS
// ============================================================================

fn draw_planets(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let view = &state.view;
    let lod = view.lod_level();

    for p in 0..state.planet_count {
        let x = state.planet_x[p];
        let y = state.planet_y[p];

        // Frustum culling
        if !view.is_visible(x, y, 1.0) {
            continue;
        }

        let (sx, sy) = view.au_to_screen(x, y);

        // Planet radius in pixels (with minimum for visibility)
        let radius_au = state.planet_radii_km[p] / AU_KM;
        let base_radius = (radius_au / view.zoom).max(4.0).min(50.0);

        let color = state.planet_colors[p];

        // Draw based on LOD
        if lod >= 2 && base_radius > 10.0 {
            // High detail - gradient sphere
            draw_planet_detailed(ctx, sx, sy, base_radius, color, state.planet_has_rings[p], time, p);
        } else {
            // Simple circle
            ctx.set_fill_style(&JsValue::from_str(color));
            ctx.begin_path();
            ctx.arc(sx, sy, base_radius, 0.0, 2.0 * PI).unwrap_or(());
            ctx.fill();
        }

        // Label
        if base_radius < 30.0 {
            ctx.set_font("11px sans-serif");
            ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.8)"));
            ctx.fill_text(state.planet_names[p], sx + base_radius + 4.0, sy + 4.0).unwrap_or(());
        }
    }
}

fn draw_planet_detailed(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64,
                         radius: f64, color: &str, has_rings: bool, _time: f64, idx: usize) {
    // Sphere gradient (3D effect)
    let gradient = ctx.create_radial_gradient(
        cx - radius * 0.3, cy - radius * 0.3, 0.0,
        cx, cy, radius
    ).unwrap();

    // Parse color and create lighter/darker versions
    gradient.add_color_stop(0.0, &lighten_color(color, 0.3)).unwrap();
    gradient.add_color_stop(0.5, color).unwrap();
    gradient.add_color_stop(1.0, &darken_color(color, 0.4)).unwrap();

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Rings (Saturn, Uranus)
    if has_rings && radius > 15.0 {
        ctx.save();
        ctx.translate(cx, cy).unwrap_or(());

        // Ring tilt varies by planet
        let tilt = if idx == 5 { 0.4 } else { 0.8 }; // Saturn more tilted

        ctx.set_stroke_style(&JsValue::from_str("rgba(200, 180, 150, 0.6)"));
        ctx.set_line_width(radius * 0.15);

        ctx.begin_path();
        ctx.ellipse(0.0, 0.0, radius * 2.0, radius * 0.3 * tilt, 0.0, 0.0, 2.0 * PI).unwrap_or(());
        ctx.stroke();

        // Inner ring
        ctx.set_stroke_style(&JsValue::from_str("rgba(180, 160, 130, 0.4)"));
        ctx.set_line_width(radius * 0.1);
        ctx.begin_path();
        ctx.ellipse(0.0, 0.0, radius * 1.5, radius * 0.22 * tilt, 0.0, 0.0, 2.0 * PI).unwrap_or(());
        ctx.stroke();

        ctx.restore();
    }

    // Subtle atmosphere glow for gas giants
    if radius > 25.0 {
        let glow = ctx.create_radial_gradient(cx, cy, radius * 0.9, cx, cy, radius * 1.2).unwrap();
        glow.add_color_stop(0.0, "rgba(255, 255, 255, 0)").unwrap();
        glow.add_color_stop(0.5, &format!("{}20", color)).unwrap();
        glow.add_color_stop(1.0, "rgba(255, 255, 255, 0)").unwrap();

        ctx.set_fill_style(&glow);
        ctx.begin_path();
        ctx.arc(cx, cy, radius * 1.2, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }
}

// ============================================================================
// MISSIONS
// ============================================================================

fn draw_missions(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let view = &state.view;

    for m in 0..state.mission_count {
        if !state.mission_active[m] { continue; }

        let x = state.mission_x[m];
        let y = state.mission_y[m];

        // Visibility check
        if !view.is_visible(x, y, 5.0) {
            continue;
        }

        let (sx, sy) = view.au_to_screen(x, y);
        let color = state.mission_colors[m];

        // Blinking beacon
        let blink = ((time * 3.0 + m as f64 * 0.5).sin() * 0.5 + 0.5).max(0.3);

        // Spacecraft icon (small triangle)
        ctx.save();
        ctx.translate(sx, sy).unwrap_or(());

        // Direction of travel (approximate)
        let angle = (y).atan2(x) + PI; // Away from sun

        ctx.rotate(angle).unwrap_or(());

        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.set_global_alpha(blink);

        // Triangle shape
        ctx.begin_path();
        ctx.move_to(6.0, 0.0);
        ctx.line_to(-4.0, -3.0);
        ctx.line_to(-4.0, 3.0);
        ctx.close_path();
        ctx.fill();

        // Glow
        ctx.set_global_alpha(blink * 0.3);
        ctx.begin_path();
        ctx.arc(0.0, 0.0, 8.0, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();

        ctx.restore();
        ctx.set_global_alpha(1.0);

        // Trail (simplified)
        draw_mission_trail(ctx, state, m);

        // Label
        ctx.set_font("10px monospace");
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.fill_text(state.mission_names[m], sx + 10.0, sy - 5.0).unwrap_or(());

        // Distance from sun
        let dist = (x * x + y * y).sqrt();
        ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.5)"));
        ctx.fill_text(&format!("{:.1} AU", dist), sx + 10.0, sy + 8.0).unwrap_or(());
    }
}

fn draw_mission_trail(ctx: &CanvasRenderingContext2d, state: &SimulationState, idx: usize) {
    let count = state.mission_waypoint_counts[idx];
    if count < 2 { return; }

    let wps = &state.mission_waypoints[idx];
    let color = state.mission_colors[idx];

    ctx.set_stroke_style(&JsValue::from_str(&format!("{}60", color)));
    ctx.set_line_width(1.0);
    ctx.begin_path();

    let (sx, sy) = state.view.au_to_screen(wps[0].1, wps[0].2);
    ctx.move_to(sx, sy);

    for i in 1..count {
        let (sx, sy) = state.view.au_to_screen(wps[i].1, wps[i].2);
        ctx.line_to(sx, sy);
    }

    ctx.stroke();
}

// ============================================================================
// UI OVERLAY
// ============================================================================

fn draw_ui_overlay(ctx: &CanvasRenderingContext2d, state: &SimulationState) {
    let w = state.view.width;
    let h = state.view.height;

    // Date display (top-left)
    let (year, month, day) = state.get_date();
    ctx.set_font("bold 16px monospace");
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.9)"));
    ctx.fill_text(&format!("{:04}-{:02}-{:02}", year, month, day), 20.0, 30.0).unwrap_or(());

    // Time scale indicator
    ctx.set_font("12px monospace");
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.6)"));

    let time_str = if state.paused {
        "PAUSED".to_string()
    } else if state.time_scale.abs() < 1.0 {
        format!("{:.2}x", state.time_scale)
    } else if state.time_scale.abs() < 365.25 {
        format!("{:.0} days/sec", state.time_scale)
    } else {
        format!("{:.1} years/sec", state.time_scale / 365.25)
    };
    ctx.fill_text(&time_str, 20.0, 50.0).unwrap_or(());

    // Zoom level (top-right)
    let zoom_str = if state.view.zoom < 0.01 {
        format!("Scale: {:.0} km/px", state.view.zoom * AU_KM)
    } else {
        format!("Scale: {:.3} AU/px", state.view.zoom)
    };
    ctx.set_text_align("right");
    ctx.fill_text(&zoom_str, w - 20.0, 30.0).unwrap_or(());
    ctx.set_text_align("start");

    // FPS (bottom-left, only if debugging)
    #[cfg(debug_assertions)]
    {
        ctx.fill_text(&format!("FPS: {:.0}", state.fps), 20.0, h - 20.0).unwrap_or(());
    }

    // Controls hint (bottom)
    ctx.set_font("11px sans-serif");
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.4)"));
    ctx.set_text_align("center");
    ctx.fill_text(
        "Scroll: zoom | Drag: pan | 1-8: planets | Space: pause | +/-: time scale",
        w / 2.0, h - 15.0
    ).unwrap_or(());
    ctx.set_text_align("start");
}

// ============================================================================
// COLOR UTILITIES
// ============================================================================

fn lighten_color(hex: &str, amount: f64) -> String {
    if let Some((r, g, b)) = parse_hex(hex) {
        let r = ((r as f64 + (255.0 - r as f64) * amount) as u8).min(255);
        let g = ((g as f64 + (255.0 - g as f64) * amount) as u8).min(255);
        let b = ((b as f64 + (255.0 - b as f64) * amount) as u8).min(255);
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        hex.to_string()
    }
}

fn darken_color(hex: &str, amount: f64) -> String {
    if let Some((r, g, b)) = parse_hex(hex) {
        let r = (r as f64 * (1.0 - amount)) as u8;
        let g = (g as f64 * (1.0 - amount)) as u8;
        let b = (b as f64 * (1.0 - amount)) as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        hex.to_string()
    }
}

fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}
