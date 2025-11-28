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

    // Solar activity based on cycle phase (more flares/prominences at solar max)
    let activity = (state.solar_cycle_phase * 2.0 * PI).sin() * 0.5 + 0.5;

    // Pulsing corona - stronger at solar maximum
    let pulse = 1.0 + (time * 0.5).sin() * 0.1 * (0.5 + activity * 0.5);
    let corona_radius = base_radius * (2.5 + activity * 1.0) * pulse;

    // Solar wind streamers (coronal streamers)
    if base_radius > 15.0 {
        draw_solar_wind(ctx, cx, cy, base_radius, time, activity);
    }

    // Outer corona glow
    let gradient = ctx.create_radial_gradient(cx, cy, base_radius, cx, cy, corona_radius).unwrap();
    gradient.add_color_stop(0.0, "rgba(255, 220, 100, 0.9)").unwrap();
    gradient.add_color_stop(0.2, "rgba(255, 180, 80, 0.6)").unwrap();
    gradient.add_color_stop(0.4, "rgba(255, 140, 60, 0.3)").unwrap();
    gradient.add_color_stop(0.7, "rgba(255, 100, 40, 0.1)").unwrap();
    gradient.add_color_stop(1.0, "rgba(255, 50, 0, 0)").unwrap();

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, corona_radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Solar prominences (arcs of plasma) - more during solar maximum
    if base_radius > 20.0 {
        let num_prominences = (2.0 + activity * 4.0) as i32;
        for i in 0..num_prominences {
            draw_solar_prominence(ctx, cx, cy, base_radius, time, i as f64, activity);
        }
    }

    // Sun body with limb darkening
    let body_gradient = ctx.create_radial_gradient(
        cx - base_radius * 0.2, cy - base_radius * 0.2, 0.0,
        cx, cy, base_radius
    ).unwrap();
    body_gradient.add_color_stop(0.0, "#FFFEF0").unwrap();
    body_gradient.add_color_stop(0.3, "#FFF8DC").unwrap();
    body_gradient.add_color_stop(0.6, "#FFE87C").unwrap();
    body_gradient.add_color_stop(0.85, "#FFD700").unwrap();
    body_gradient.add_color_stop(0.95, "#FFA500").unwrap();
    body_gradient.add_color_stop(1.0, "#FF6B00").unwrap();

    ctx.set_fill_style(&body_gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, base_radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Granulation (convection cells) - only when zoomed in
    if base_radius > 40.0 {
        draw_solar_granulation(ctx, cx, cy, base_radius, time);
    }

    // Sunspots - more during solar maximum
    if base_radius > 20.0 {
        let num_spots = (1.0 + activity * 6.0) as i32;
        draw_sunspots(ctx, cx, cy, base_radius, time, num_spots, activity);
    }

    // Active regions (bright faculae near sunspots)
    if base_radius > 30.0 && activity > 0.3 {
        draw_faculae(ctx, cx, cy, base_radius, time, activity);
    }

    // Label
    if view.zoom < 0.05 {
        ctx.set_font("bold 14px sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#FFD700"));
        ctx.fill_text("Sun", cx + base_radius + 5.0, cy + 5.0).unwrap_or(());
    }
}

/// Draw solar wind streamers emanating from the sun
fn draw_solar_wind(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64, activity: f64) {
    let num_streamers = 8;
    ctx.set_global_alpha(0.15);

    for i in 0..num_streamers {
        let base_angle = (i as f64 / num_streamers as f64) * 2.0 * PI;
        let wobble = (time * 0.3 + i as f64 * 0.7).sin() * 0.1;
        let angle = base_angle + wobble;

        let length = radius * (3.0 + activity * 2.0 + (time * 0.2 + i as f64).sin() * 0.5);

        let grad = ctx.create_linear_gradient(
            cx + angle.cos() * radius,
            cy + angle.sin() * radius,
            cx + angle.cos() * length,
            cy + angle.sin() * length
        );
        grad.add_color_stop(0.0, "rgba(255, 200, 100, 0.5)").unwrap();
        grad.add_color_stop(0.5, "rgba(255, 150, 50, 0.2)").unwrap();
        grad.add_color_stop(1.0, "rgba(255, 100, 0, 0)").unwrap();

        ctx.set_stroke_style(&grad);
        ctx.set_line_width(radius * 0.3);
        ctx.begin_path();
        ctx.move_to(cx + angle.cos() * radius, cy + angle.sin() * radius);
        ctx.line_to(cx + angle.cos() * length, cy + angle.sin() * length);
        ctx.stroke();
    }

    ctx.set_global_alpha(1.0);
}

/// Draw solar prominences (plasma arcs)
fn draw_solar_prominence(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64,
                          time: f64, idx: f64, activity: f64) {
    let seed = idx * 3.14159 + time * 0.01;
    let base_angle = (seed * 2.7) % (2.0 * PI);

    // Only draw if on visible portion
    let vis = (seed * 1.3).sin();
    if vis < 0.0 { return; }

    let height = radius * (0.3 + (seed * 1.7).sin().abs() * 0.4) * activity;
    let width = radius * 0.15;

    ctx.save();
    ctx.translate(cx, cy).unwrap_or(());
    ctx.rotate(base_angle).unwrap_or(());

    // Prominence arc
    let prom_grad = ctx.create_radial_gradient(0.0, -radius - height * 0.5, 0.0,
                                                0.0, -radius, height).unwrap();
    prom_grad.add_color_stop(0.0, "rgba(255, 100, 50, 0.8)").unwrap();
    prom_grad.add_color_stop(0.5, "rgba(255, 80, 30, 0.5)").unwrap();
    prom_grad.add_color_stop(1.0, "rgba(255, 50, 0, 0)").unwrap();

    ctx.set_fill_style(&prom_grad);
    ctx.begin_path();

    // Draw arc shape
    ctx.move_to(-width, -radius);
    ctx.quadratic_curve_to(-width * 0.5, -radius - height, 0.0, -radius - height * 0.8);
    ctx.quadratic_curve_to(width * 0.5, -radius - height, width, -radius);
    ctx.close_path();
    ctx.fill();

    ctx.restore();
}

/// Draw solar granulation pattern
fn draw_solar_granulation(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.95, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    // Granulation cells (bright centers, dark edges)
    let cell_count = 30;
    ctx.set_global_alpha(0.15);

    for i in 0..cell_count {
        let seed = i as f64 * 7.31;
        let angle = (seed * 2.1 + time * 0.001) % (2.0 * PI);
        let dist = (seed * 1.3).sin().abs() * radius * 0.85;

        let cell_x = cx + angle.cos() * dist;
        let cell_y = cy + angle.sin() * dist;
        let cell_r = radius * (0.04 + (seed * 0.9).sin().abs() * 0.03);

        // Bright granule center
        ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 230, 0.4)"));
        ctx.begin_path();
        ctx.arc(cell_x, cell_y, cell_r, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();

        // Dark intergranular lane
        ctx.set_stroke_style(&JsValue::from_str("rgba(200, 150, 50, 0.3)"));
        ctx.set_line_width(1.0);
        ctx.stroke();
    }

    ctx.set_global_alpha(1.0);
    ctx.restore();
}

/// Draw sunspots with umbra and penumbra
fn draw_sunspots(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64,
                  time: f64, num_spots: i32, activity: f64) {
    for i in 0..num_spots {
        let seed = i as f64 * 2.71;
        let angle = (time * 0.02 + seed * 2.0) % (2.0 * PI);
        let dist = radius * (0.2 + (seed * 1.5).sin().abs() * 0.5);

        // Only draw spots on visible hemisphere
        if angle.cos() < -0.3 { continue; }

        let spot_x = cx + angle.cos() * dist;
        let spot_y = cy + angle.sin() * dist * 0.8; // Foreshortening
        let spot_r = radius * (0.03 + (seed * 3.0).sin().abs() * 0.05) * (0.5 + activity);

        // Foreshorten near limb
        let limb_factor = (1.0 - (dist / radius).powi(2)).sqrt().max(0.3);
        let drawn_r = spot_r * limb_factor;

        // Penumbra (outer, lighter)
        ctx.set_fill_style(&JsValue::from_str("rgba(140, 80, 20, 0.5)"));
        ctx.begin_path();
        ctx.arc(spot_x, spot_y, drawn_r * 1.5, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();

        // Umbra (inner, darker)
        ctx.set_fill_style(&JsValue::from_str("rgba(60, 30, 10, 0.7)"));
        ctx.begin_path();
        ctx.arc(spot_x, spot_y, drawn_r, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }
}

/// Draw bright faculae (active regions near sunspots)
fn draw_faculae(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64, activity: f64) {
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 220, 0.2)"));

    for i in 0..4 {
        let seed = i as f64 * 4.17;
        let angle = (time * 0.02 + seed * 2.5) % (2.0 * PI);
        let dist = radius * (0.6 + (seed * 1.2).sin().abs() * 0.3);

        // Faculae are more visible near the limb
        if angle.cos().abs() > 0.5 { continue; }

        let fac_x = cx + angle.cos() * dist;
        let fac_y = cy + angle.sin() * dist * 0.9;
        let fac_r = radius * 0.08 * activity;

        ctx.begin_path();
        ctx.ellipse(fac_x, fac_y, fac_r, fac_r * 0.6, angle, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
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
                         radius: f64, color: &str, has_rings: bool, time: f64, idx: usize) {
    // Planet-specific rendering based on index
    // 0=Mercury, 1=Venus, 2=Earth, 3=Mars, 4=Jupiter, 5=Saturn, 6=Uranus, 7=Neptune

    match idx {
        2 => draw_earth(ctx, cx, cy, radius, time),      // Earth with continents
        4 => draw_jupiter(ctx, cx, cy, radius, time),    // Jupiter with bands and GRS
        5 => draw_saturn(ctx, cx, cy, radius, time),     // Saturn with detailed rings
        3 => draw_mars(ctx, cx, cy, radius, time),       // Mars with polar caps
        _ => draw_generic_planet(ctx, cx, cy, radius, color, has_rings, time, idx),
    }
}

/// Earth with blue oceans, green/brown continents, polar ice caps
fn draw_earth(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Ocean base
    let gradient = ctx.create_radial_gradient(
        cx - radius * 0.3, cy - radius * 0.3, 0.0,
        cx, cy, radius
    ).unwrap();
    gradient.add_color_stop(0.0, "#8BC4E8").unwrap();
    gradient.add_color_stop(0.5, "#4A90C2").unwrap();
    gradient.add_color_stop(1.0, "#1A4A6E").unwrap();

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Clipping for continent features
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    // Procedural continents (simplified shapes that rotate with time)
    let rotation = time * 0.03;
    ctx.set_fill_style(&JsValue::from_str("rgba(90, 130, 70, 0.7)"));

    // North America-ish blob
    draw_continent_blob(ctx, cx, cy, radius, rotation + 0.0, 0.3, 0.35, 0.25);
    // Europe/Africa-ish blob
    draw_continent_blob(ctx, cx, cy, radius, rotation + 1.8, 0.2, 0.4, 0.3);
    // Asia blob
    draw_continent_blob(ctx, cx, cy, radius, rotation + 3.5, 0.35, 0.3, 0.35);
    // South America
    draw_continent_blob(ctx, cx, cy, radius, rotation + 0.5, -0.25, 0.15, 0.2);
    // Australia
    draw_continent_blob(ctx, cx, cy, radius, rotation + 4.5, -0.35, 0.12, 0.1);

    // Polar ice caps
    ctx.set_fill_style(&JsValue::from_str("rgba(240, 250, 255, 0.85)"));
    ctx.begin_path();
    ctx.arc(cx, cy - radius * 0.85, radius * 0.25, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
    ctx.begin_path();
    ctx.arc(cx, cy + radius * 0.88, radius * 0.2, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Cloud layer
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.25)"));
    for i in 0..5 {
        let seed = i as f64 * 1.7;
        let angle = rotation * 1.2 + seed;
        let lat = (seed * 2.1).sin() * 0.6;
        let cloud_x = cx + angle.cos() * radius * 0.7 * (1.0 - lat.abs());
        let cloud_y = cy + lat * radius * 0.9;
        let cloud_r = radius * (0.15 + (seed * 1.3).sin().abs() * 0.1);
        ctx.begin_path();
        ctx.ellipse(cloud_x, cloud_y, cloud_r, cloud_r * 0.4, angle * 0.5, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }

    ctx.restore();

    // Atmosphere glow
    let atmo = ctx.create_radial_gradient(cx, cy, radius * 0.95, cx, cy, radius * 1.15).unwrap();
    atmo.add_color_stop(0.0, "rgba(100, 180, 255, 0)").unwrap();
    atmo.add_color_stop(0.5, "rgba(100, 180, 255, 0.15)").unwrap();
    atmo.add_color_stop(1.0, "rgba(100, 180, 255, 0)").unwrap();
    ctx.set_fill_style(&atmo);
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 1.15, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Draw a blob-shaped continent
fn draw_continent_blob(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64,
                        longitude: f64, latitude: f64, width: f64, height: f64) {
    // Only draw if on visible side (longitude between -PI/2 and PI/2 from view)
    let vis_angle = longitude % (2.0 * PI);
    if vis_angle > PI * 0.5 && vis_angle < PI * 1.5 {
        return;
    }

    let x = cx + longitude.cos() * radius * 0.7 * (1.0 - latitude.abs() * 0.3);
    let y = cy + latitude * radius * 0.9;
    let w = radius * width * longitude.cos().abs().max(0.3);
    let h = radius * height;

    ctx.begin_path();
    ctx.ellipse(x, y, w, h, longitude * 0.2, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Jupiter with cloud bands and Great Red Spot
fn draw_jupiter(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Base color
    let gradient = ctx.create_radial_gradient(
        cx - radius * 0.3, cy - radius * 0.3, 0.0,
        cx, cy, radius
    ).unwrap();
    gradient.add_color_stop(0.0, "#F5E6D3").unwrap();
    gradient.add_color_stop(0.5, "#D4A574").unwrap();
    gradient.add_color_stop(1.0, "#8B6914").unwrap();

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Clipping for bands
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    // Cloud bands (alternating light/dark)
    let band_colors = [
        "rgba(230, 200, 170, 0.5)",  // Light zone
        "rgba(160, 110, 70, 0.5)",   // Dark belt
        "rgba(220, 190, 160, 0.5)",
        "rgba(140, 90, 50, 0.6)",
        "rgba(210, 180, 150, 0.5)",
        "rgba(150, 100, 60, 0.5)",
        "rgba(200, 170, 140, 0.5)",
    ];

    let band_height = radius * 2.0 / band_colors.len() as f64;
    for (i, color) in band_colors.iter().enumerate() {
        let y_offset = cy - radius + band_height * i as f64;
        // Wavy bands
        let wave = (time * 0.1 + i as f64 * 0.5).sin() * radius * 0.02;
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.fill_rect(cx - radius * 1.1, y_offset + wave, radius * 2.2, band_height * 1.1);
    }

    // Great Red Spot
    let grs_rotation = time * 0.02;
    let grs_x = cx + grs_rotation.cos() * radius * 0.4;
    let grs_y = cy + radius * 0.2; // South of equator

    // Only draw if on visible side
    if grs_rotation.cos() > -0.3 {
        let grs_gradient = ctx.create_radial_gradient(
            grs_x, grs_y, 0.0,
            grs_x, grs_y, radius * 0.2
        ).unwrap();
        grs_gradient.add_color_stop(0.0, "rgba(200, 80, 60, 0.9)").unwrap();
        grs_gradient.add_color_stop(0.5, "rgba(180, 70, 50, 0.7)").unwrap();
        grs_gradient.add_color_stop(1.0, "rgba(160, 100, 80, 0)").unwrap();

        ctx.set_fill_style(&grs_gradient);
        ctx.begin_path();
        ctx.ellipse(grs_x, grs_y, radius * 0.18, radius * 0.1, 0.0, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }

    ctx.restore();

    // Subtle atmosphere
    let atmo = ctx.create_radial_gradient(cx, cy, radius * 0.95, cx, cy, radius * 1.08).unwrap();
    atmo.add_color_stop(0.0, "rgba(255, 220, 180, 0)").unwrap();
    atmo.add_color_stop(0.6, "rgba(255, 220, 180, 0.1)").unwrap();
    atmo.add_color_stop(1.0, "rgba(255, 200, 150, 0)").unwrap();
    ctx.set_fill_style(&atmo);
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 1.08, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Saturn with detailed ring system
fn draw_saturn(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, _time: f64) {
    // Ring system (behind planet)
    draw_saturn_rings(ctx, cx, cy, radius, true);

    // Planet body
    let gradient = ctx.create_radial_gradient(
        cx - radius * 0.3, cy - radius * 0.3, 0.0,
        cx, cy, radius
    ).unwrap();
    gradient.add_color_stop(0.0, "#F5E8C8").unwrap();
    gradient.add_color_stop(0.5, "#E3D4AD").unwrap();
    gradient.add_color_stop(1.0, "#A08050").unwrap();

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Subtle bands
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    let bands = ["rgba(200, 180, 140, 0.3)", "rgba(180, 160, 120, 0.2)", "rgba(190, 170, 130, 0.25)"];
    for (i, color) in bands.iter().enumerate() {
        let y = cy - radius * 0.6 + (i as f64 * radius * 0.4);
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.fill_rect(cx - radius, y, radius * 2.0, radius * 0.3);
    }
    ctx.restore();

    // Ring system (in front of planet)
    draw_saturn_rings(ctx, cx, cy, radius, false);
}

/// Draw Saturn's ring system
fn draw_saturn_rings(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, behind: bool) {
    if radius < 15.0 { return; }

    ctx.save();
    ctx.translate(cx, cy).unwrap_or(());

    // Ring tilt
    let tilt = 0.4;

    // Ring definitions: (inner_mult, outer_mult, color, opacity)
    let rings = [
        (1.25, 1.45, "#C4B896", 0.7),  // C Ring (innermost, faint)
        (1.50, 1.95, "#D4C8A6", 0.85), // B Ring (bright)
        (2.00, 2.05, "#000000", 0.0),  // Cassini Division (gap)
        (2.10, 2.30, "#E8DCC0", 0.75), // A Ring
        (2.35, 2.40, "#000000", 0.0),  // Encke Gap
        (2.45, 2.55, "#C8BC98", 0.5),  // F Ring (faint, outer)
    ];

    for (inner, outer, color, opacity) in rings.iter() {
        if *opacity < 0.1 { continue; }

        let inner_r = radius * inner;
        let outer_r = radius * outer;

        // Draw arc (either top half or bottom half)
        ctx.set_global_alpha(*opacity * if behind { 0.5 } else { 1.0 });

        // Create gradient for ring
        let ring_grad = ctx.create_linear_gradient(-outer_r, 0.0, outer_r, 0.0);
        ring_grad.add_color_stop(0.0, &format!("{}60", color)).unwrap();
        ring_grad.add_color_stop(0.3, color).unwrap();
        ring_grad.add_color_stop(0.5, &lighten_color(color, 0.2)).unwrap();
        ring_grad.add_color_stop(0.7, color).unwrap();
        ring_grad.add_color_stop(1.0, &format!("{}60", color)).unwrap();

        ctx.set_fill_style(&ring_grad);
        ctx.begin_path();

        if behind {
            // Draw top arc (behind planet)
            ctx.ellipse(0.0, 0.0, outer_r, outer_r * tilt, 0.0, PI, 2.0 * PI).unwrap_or(());
            ctx.ellipse(0.0, 0.0, inner_r, inner_r * tilt, 0.0, 2.0 * PI, PI).unwrap_or(());
        } else {
            // Draw bottom arc (in front of planet)
            ctx.ellipse(0.0, 0.0, outer_r, outer_r * tilt, 0.0, 0.0, PI).unwrap_or(());
            ctx.ellipse(0.0, 0.0, inner_r, inner_r * tilt, 0.0, PI, 0.0).unwrap_or(());
        }
        ctx.close_path();
        ctx.fill();
    }

    ctx.set_global_alpha(1.0);
    ctx.restore();
}

/// Mars with red surface and polar ice caps
fn draw_mars(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Red surface base
    let gradient = ctx.create_radial_gradient(
        cx - radius * 0.3, cy - radius * 0.3, 0.0,
        cx, cy, radius
    ).unwrap();
    gradient.add_color_stop(0.0, "#E8A080").unwrap();
    gradient.add_color_stop(0.5, "#C1440E").unwrap();
    gradient.add_color_stop(1.0, "#6E2800").unwrap();

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Surface features
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    // Dark regions (like Syrtis Major)
    let rotation = time * 0.02;
    ctx.set_fill_style(&JsValue::from_str("rgba(80, 30, 10, 0.4)"));

    let dark_x = cx + rotation.cos() * radius * 0.3;
    if rotation.cos() > 0.0 {
        ctx.begin_path();
        ctx.ellipse(dark_x, cy + radius * 0.1, radius * 0.25, radius * 0.4, 0.3, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }

    // Polar ice caps (white)
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 250, 245, 0.9)"));
    ctx.begin_path();
    ctx.arc(cx, cy - radius * 0.85, radius * 0.2, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Southern cap (smaller)
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 250, 245, 0.7)"));
    ctx.begin_path();
    ctx.arc(cx, cy + radius * 0.9, radius * 0.12, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    ctx.restore();

    // Thin atmosphere
    let atmo = ctx.create_radial_gradient(cx, cy, radius * 0.95, cx, cy, radius * 1.05).unwrap();
    atmo.add_color_stop(0.0, "rgba(255, 200, 180, 0)").unwrap();
    atmo.add_color_stop(0.7, "rgba(255, 180, 150, 0.08)").unwrap();
    atmo.add_color_stop(1.0, "rgba(255, 150, 120, 0)").unwrap();
    ctx.set_fill_style(&atmo);
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 1.05, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Generic planet rendering for Mercury, Venus, Uranus, Neptune
fn draw_generic_planet(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64,
                       radius: f64, color: &str, has_rings: bool, time: f64, idx: usize) {
    // Sphere gradient (3D effect)
    let gradient = ctx.create_radial_gradient(
        cx - radius * 0.3, cy - radius * 0.3, 0.0,
        cx, cy, radius
    ).unwrap();

    gradient.add_color_stop(0.0, &lighten_color(color, 0.3)).unwrap();
    gradient.add_color_stop(0.5, color).unwrap();
    gradient.add_color_stop(1.0, &darken_color(color, 0.4)).unwrap();

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Planet-specific features
    match idx {
        0 => { // Mercury - cratered surface
            ctx.set_fill_style(&JsValue::from_str("rgba(80, 80, 80, 0.3)"));
            for i in 0..5 {
                let seed = i as f64 * 2.3;
                let crater_x = cx + (seed * 1.7).cos() * radius * 0.5;
                let crater_y = cy + (seed * 2.1).sin() * radius * 0.5;
                let crater_r = radius * (0.05 + (seed * 0.7).sin().abs() * 0.08);
                ctx.begin_path();
                ctx.arc(crater_x, crater_y, crater_r, 0.0, 2.0 * PI).unwrap_or(());
                ctx.fill();
            }
        }
        1 => { // Venus - thick atmosphere swirls
            ctx.save();
            ctx.begin_path();
            ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
            ctx.clip();

            ctx.set_fill_style(&JsValue::from_str("rgba(255, 230, 180, 0.3)"));
            let rot = time * 0.005; // Very slow rotation
            for i in 0..4 {
                let y = cy - radius * 0.6 + (i as f64 * radius * 0.35);
                let wave = (rot + i as f64 * 0.8).sin() * radius * 0.1;
                ctx.fill_rect(cx - radius + wave, y, radius * 2.0, radius * 0.25);
            }
            ctx.restore();
        }
        6 => { // Uranus - tilted rings and blue-green color
            // Uranus rings (very faint, nearly vertical due to extreme tilt)
            if radius > 15.0 {
                ctx.save();
                ctx.translate(cx, cy).unwrap_or(());
                ctx.rotate(PI * 0.47).unwrap_or(()); // Nearly sideways

                ctx.set_stroke_style(&JsValue::from_str("rgba(150, 180, 180, 0.3)"));
                ctx.set_line_width(radius * 0.08);
                ctx.begin_path();
                ctx.ellipse(0.0, 0.0, radius * 1.8, radius * 0.15, 0.0, 0.0, 2.0 * PI).unwrap_or(());
                ctx.stroke();

                ctx.restore();
            }
        }
        7 => { // Neptune - dark spot and bands
            ctx.save();
            ctx.begin_path();
            ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
            ctx.clip();

            // Faint bands
            ctx.set_fill_style(&JsValue::from_str("rgba(40, 80, 180, 0.2)"));
            ctx.fill_rect(cx - radius, cy - radius * 0.2, radius * 2.0, radius * 0.3);
            ctx.fill_rect(cx - radius, cy + radius * 0.3, radius * 2.0, radius * 0.2);

            // Great Dark Spot
            let spot_rot = time * 0.015;
            if spot_rot.cos() > 0.0 {
                ctx.set_fill_style(&JsValue::from_str("rgba(30, 50, 120, 0.5)"));
                let spot_x = cx + spot_rot.cos() * radius * 0.3;
                ctx.begin_path();
                ctx.ellipse(spot_x, cy - radius * 0.15, radius * 0.15, radius * 0.1, 0.0, 0.0, 2.0 * PI).unwrap_or(());
                ctx.fill();
            }
            ctx.restore();
        }
        _ => {}
    }

    // Rings for Uranus (handled above) - skip here
    if has_rings && idx != 6 && radius > 15.0 {
        ctx.save();
        ctx.translate(cx, cy).unwrap_or(());

        let tilt = if idx == 5 { 0.4 } else { 0.8 };

        ctx.set_stroke_style(&JsValue::from_str("rgba(200, 180, 150, 0.6)"));
        ctx.set_line_width(radius * 0.15);
        ctx.begin_path();
        ctx.ellipse(0.0, 0.0, radius * 2.0, radius * 0.3 * tilt, 0.0, 0.0, 2.0 * PI).unwrap_or(());
        ctx.stroke();

        ctx.restore();
    }

    // Atmosphere glow for gas/ice giants
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
        let name = state.mission_names[m];

        // Blinking beacon
        let blink = ((time * 3.0 + m as f64 * 0.5).sin() * 0.5 + 0.5).max(0.3);

        // Draw spacecraft based on mission type
        ctx.save();
        ctx.translate(sx, sy).unwrap_or(());

        // Direction of travel (approximate)
        let angle = (y).atan2(x) + PI; // Away from sun
        ctx.rotate(angle).unwrap_or(());

        ctx.set_global_alpha(blink);

        // Draw mission-specific spacecraft shape
        match name {
            "Voyager 1" | "Voyager 2" => draw_voyager(ctx, color),
            "New Horizons" => draw_new_horizons(ctx, color),
            "Parker Solar" => draw_parker_probe(ctx, color),
            _ => draw_generic_spacecraft(ctx, color),
        }

        // Communication beam (pulsing)
        draw_comm_beam(ctx, color, time, m as f64);

        ctx.restore();
        ctx.set_global_alpha(1.0);

        // Trail
        draw_mission_trail(ctx, state, m);

        // Label with icon
        ctx.set_font("10px monospace");
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.fill_text(name, sx + 12.0, sy - 5.0).unwrap_or(());

        // Distance from sun
        let dist = (x * x + y * y).sqrt();
        ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.5)"));
        ctx.fill_text(&format!("{:.1} AU", dist), sx + 12.0, sy + 8.0).unwrap_or(());
    }
}

/// Voyager spacecraft with dish antenna and RTG boom
fn draw_voyager(ctx: &CanvasRenderingContext2d, color: &str) {
    // Main bus (rectangular body)
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.fill_rect(-3.0, -2.0, 6.0, 4.0);

    // High-gain antenna (large dish)
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(1.5);
    ctx.begin_path();
    ctx.arc(5.0, 0.0, 5.0, -0.8, 0.8).unwrap_or(());
    ctx.stroke();

    // Dish fill
    ctx.set_fill_style(&JsValue::from_str(&format!("{}80", color)));
    ctx.begin_path();
    ctx.move_to(5.0, 0.0);
    ctx.arc(5.0, 0.0, 5.0, -0.8, 0.8).unwrap_or(());
    ctx.close_path();
    ctx.fill();

    // RTG boom (nuclear power)
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.move_to(-3.0, 0.0);
    ctx.line_to(-10.0, 5.0);
    ctx.stroke();

    // RTG cylinders
    ctx.set_fill_style(&JsValue::from_str("rgba(180, 120, 80, 0.8)"));
    ctx.fill_rect(-11.0, 3.0, 3.0, 4.0);

    // Magnetometer boom
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.begin_path();
    ctx.move_to(-3.0, 0.0);
    ctx.line_to(-12.0, -4.0);
    ctx.stroke();

    // Golden record indicator
    ctx.set_fill_style(&JsValue::from_str("#FFD700"));
    ctx.begin_path();
    ctx.arc(0.0, 0.0, 1.5, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// New Horizons with triangular shape and dish
fn draw_new_horizons(ctx: &CanvasRenderingContext2d, color: &str) {
    // Triangular body
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.begin_path();
    ctx.move_to(6.0, 0.0);
    ctx.line_to(-4.0, -4.0);
    ctx.line_to(-4.0, 4.0);
    ctx.close_path();
    ctx.fill();

    // High-gain dish
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(1.5);
    ctx.begin_path();
    ctx.arc(3.0, 0.0, 4.0, -0.7, 0.7).unwrap_or(());
    ctx.stroke();

    // RTG (single unit)
    ctx.set_fill_style(&JsValue::from_str("rgba(200, 150, 100, 0.8)"));
    ctx.fill_rect(-8.0, -1.5, 4.0, 3.0);

    // LORRI telescope
    ctx.set_fill_style(&JsValue::from_str("rgba(100, 100, 120, 0.9)"));
    ctx.fill_rect(-2.0, -4.5, 3.0, 2.0);
}

/// Parker Solar Probe with heat shield
fn draw_parker_probe(ctx: &CanvasRenderingContext2d, color: &str) {
    // Heat shield (large white circle facing sun)
    ctx.set_fill_style(&JsValue::from_str("rgba(240, 240, 245, 0.9)"));
    ctx.begin_path();
    ctx.arc(-4.0, 0.0, 6.0, -1.2, 1.2).unwrap_or(());
    ctx.close_path();
    ctx.fill();

    // Shield edge glow (hot!)
    ctx.set_stroke_style(&JsValue::from_str("rgba(255, 150, 50, 0.6)"));
    ctx.set_line_width(2.0);
    ctx.begin_path();
    ctx.arc(-4.0, 0.0, 6.0, -1.2, 1.2).unwrap_or(());
    ctx.stroke();

    // Spacecraft body (behind shield)
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.fill_rect(0.0, -2.0, 5.0, 4.0);

    // Solar panels (retracted/angled for protection)
    ctx.set_fill_style(&JsValue::from_str("rgba(50, 80, 150, 0.8)"));
    ctx.begin_path();
    ctx.move_to(3.0, -2.0);
    ctx.line_to(6.0, -5.0);
    ctx.line_to(8.0, -4.0);
    ctx.line_to(5.0, -1.0);
    ctx.close_path();
    ctx.fill();

    ctx.begin_path();
    ctx.move_to(3.0, 2.0);
    ctx.line_to(6.0, 5.0);
    ctx.line_to(8.0, 4.0);
    ctx.line_to(5.0, 1.0);
    ctx.close_path();
    ctx.fill();
}

/// Generic spacecraft for other missions
fn draw_generic_spacecraft(ctx: &CanvasRenderingContext2d, color: &str) {
    ctx.set_fill_style(&JsValue::from_str(color));

    // Body
    ctx.begin_path();
    ctx.move_to(6.0, 0.0);
    ctx.line_to(-4.0, -3.0);
    ctx.line_to(-4.0, 3.0);
    ctx.close_path();
    ctx.fill();

    // Solar panels
    ctx.set_fill_style(&JsValue::from_str("rgba(50, 100, 180, 0.7)"));
    ctx.fill_rect(-2.0, -7.0, 4.0, 4.0);
    ctx.fill_rect(-2.0, 3.0, 4.0, 4.0);

    // Glow
    ctx.set_fill_style(&JsValue::from_str(&format!("{}30", color)));
    ctx.begin_path();
    ctx.arc(0.0, 0.0, 8.0, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Communication beam pulsing towards Earth direction
fn draw_comm_beam(ctx: &CanvasRenderingContext2d, color: &str, time: f64, idx: f64) {
    let pulse = ((time * 5.0 + idx * 2.0).sin() * 0.5 + 0.5).powi(3);
    if pulse < 0.1 { return; }

    ctx.set_global_alpha(pulse * 0.3);
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(0.5);

    // Beam towards "Earth" (roughly back towards sun direction)
    ctx.begin_path();
    ctx.move_to(5.0, 0.0);
    ctx.line_to(5.0 + 20.0 * pulse, 0.0);
    ctx.stroke();

    ctx.set_global_alpha(1.0);
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
