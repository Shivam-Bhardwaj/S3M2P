//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: render.rs | HELIOS/src/render.rs
//! PURPOSE: Canvas 2D rendering engine for solar system visualization with starfield and UI
//! MODIFIED: 2025-12-02
//! LAYER: HELIOS (simulation)
//! ═══════════════════════════════════════════════════════════════════════════════

// Canvas 2D Renderer - Following too.foo patterns
// No GPU required, efficient CPU rendering
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::approx_constant)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unnecessary_min_or_max)]

use crate::simulation::{SimulationState, AU_KM, ORBIT_SEGMENTS, SOLAR_RADIUS_KM};
use dna::world::cca::{Epoch, FrameId, TimeScale};
use std::f64::consts::PI;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

// ============================================================================
// DRAWING UTILITIES
// ============================================================================

/// Compute breathing factor for visual elements
/// Returns value centered at 1.0, oscillating by amplitude
#[inline]
fn breath_factor(time: f64, frequency: f64, amplitude: f64, phase: f64) -> f64 {
    1.0 + (time * frequency + phase).sin() * amplitude
}

/// Multi-layered breathing for organic feel
/// Combines slow, medium, and fast oscillations weighted by solar activity
#[inline]
fn layered_breath(time: f64, base_amp: f64, activity: f64) -> f64 {
    let slow = (time * 0.1).sin() * base_amp * 0.5;
    let medium = (time * 0.35).sin() * base_amp * 0.8;
    let fast = (time * 0.9).sin() * base_amp * 0.3;
    1.0 + (slow + medium + fast) * (0.5 + activity * 0.5)
}

/// Safely create a transparent version of a hex color
/// Handles malformed colors gracefully to prevent rendering artifacts
#[inline]
fn color_transparent(color: &str) -> String {
    if color.len() >= 7 && color.starts_with('#') {
        format!("{}00", &color[..7])
    } else {
        "transparent".to_string()
    }
}

/// Draw the entire scene
pub fn render(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let w = state.view.width;
    let h = state.view.height;

    // Clear with space background
    ctx.set_fill_style(&JsValue::from_str("#000008"));
    ctx.fill_rect(0.0, 0.0, w, h);

    // Draw layers back to front
    draw_starfield(ctx, state, time);
    draw_heliosphere_boundaries(ctx, state, time);
    draw_orbits(ctx, state, time);
    draw_missions(ctx, state, time);
    draw_sun(ctx, state, time);
    draw_planets(ctx, state, time);
    draw_ui_overlay(ctx, state);
}

// ============================================================================
// STARFIELD & CELESTIAL BACKGROUND
// ============================================================================

// NOTE: BRIGHT_STARS is deprecated - now using StarDatabase with 3D positions
// Kept for reference / potential fallback
#[allow(dead_code)]
const BRIGHT_STARS: &[(&str, f64, f64, f64, &str)] = &[
    // (name, ecliptic_lon, ecliptic_lat, magnitude, color)
    ("Sirius", 104.0, -39.6, -1.46, "#A3CFFF"), // Alpha CMa - brightest star
    ("Canopus", 96.0, -76.0, -0.72, "#FFFEF0"), // Alpha Car
    ("Arcturus", 214.0, 30.7, -0.05, "#FFB347"), // Alpha Boo - orange giant
    ("Vega", 285.0, 61.7, 0.03, "#A3CFFF"),     // Alpha Lyr - near solar apex
    ("Capella", 79.0, 23.0, 0.08, "#FFFBCC"),   // Alpha Aur
    ("Rigel", 78.0, -31.0, 0.13, "#B4CFFF"),    // Beta Ori
    ("Procyon", 116.0, -16.0, 0.34, "#FFEFD5"), // Alpha CMi
    ("Betelgeuse", 89.0, -16.0, 0.42, "#FF6B35"), // Alpha Ori - red supergiant
    ("Aldebaran", 69.0, -5.5, 0.85, "#FF8C42"), // Alpha Tau - orange giant
    ("Spica", 204.0, -2.0, 1.04, "#B4CFFF"),    // Alpha Vir
    ("Antares", 248.0, -4.5, 1.09, "#FF4500"),  // Alpha Sco - red supergiant
    ("Pollux", 113.0, 6.7, 1.14, "#FFD27F"),    // Beta Gem
    ("Fomalhaut", 334.0, -21.0, 1.16, "#A3CFFF"), // Alpha PsA
    ("Deneb", 310.0, 60.0, 1.25, "#FFFFFF"),    // Alpha Cyg
    ("Regulus", 150.0, 0.5, 1.35, "#B4CFFF"),   // Alpha Leo
    ("Castor", 113.5, 10.0, 1.58, "#A3CFFF"),   // Alpha Gem
    ("Polaris", 88.0, 66.4, 1.98, "#FFFBCC"),   // Alpha UMi - near north ecliptic pole
    ("Altair", 296.0, 29.3, 0.77, "#FFFFFF"),   // Alpha Aql
];

// ============================================================================
// CONSTELLATION DATA - Now from UniverseDataManager
// ============================================================================

// NOTE: ADDITIONAL_BRIGHT_STARS is deprecated - now using StarDatabase
#[allow(dead_code)]
const ADDITIONAL_BRIGHT_STARS: &[(&str, f64, f64, f64, &str)] = &[
    // Summer Triangle (complete with the ones already in BRIGHT_STARS)
    // Vega, Altair, Deneb already included

    // Winter Hexagon / Winter Circle
    ("Capella", 79.0, 23.0, 0.08, "#FFFBCC"), // Already included
    ("Rigel", 78.0, -31.0, 0.13, "#B4CFFF"),  // Already included
    ("Aldebaran", 69.0, -5.5, 0.85, "#FF8C42"), // Already included
    ("Procyon", 116.0, -16.0, 0.34, "#FFEFD5"), // Already included
    ("Sirius", 104.0, -39.6, -1.46, "#A3CFFF"), // Already included
    // Southern prominent stars
    ("Achernar", 335.0, -59.0, 0.46, "#B4CFFF"), // Alpha Eri - very south
    ("Hadar", 232.0, -44.0, 0.61, "#B4CFFF"),    // Beta Cen
    ("Acrux", 217.0, -53.0, 0.76, "#B4CFFF"),    // Alpha Cru (Southern Cross)
    ("Mimosa", 223.0, -50.0, 1.25, "#B4CFFF"),   // Beta Cru
    // More zodiac stars
    ("Zubenelgenubi", 225.0, 0.0, 2.75, "#FFFFFF"), // Alpha Lib (Libra)
    ("Zubeneschamali", 229.0, 9.0, 2.61, "#B4FFB4"), // Beta Lib (greenish)
    // More northern stars
    ("Mirfak", 64.0, 30.0, 1.79, "#FFFBCC"),    // Alpha Per
    ("Algol", 55.0, 22.0, 2.12, "#B4CFFF"),     // Beta Per (eclipsing binary)
    ("Hamal", 37.0, 10.0, 2.00, "#FFD27F"),     // Alpha Ari (Aries)
    ("Alpheratz", 14.0, 26.0, 2.06, "#B4CFFF"), // Alpha And
    ("Mirach", 24.0, 26.0, 2.05, "#FF8C42"),    // Beta And
    ("Almach", 36.0, 28.0, 2.26, "#FFD27F"),    // Gamma And
    // Pegasus Square
    ("Scheat", 9.0, 20.0, 2.42, "#FF8C42"), // Beta Peg
    ("Markab", 3.0, 12.0, 2.49, "#B4CFFF"), // Alpha Peg
    ("Algenib", 8.0, 7.0, 2.83, "#B4CFFF"), // Gamma Peg
    // Misc bright stars
    ("Enif", 341.0, 18.0, 2.39, "#FFD27F"), // Epsilon Peg (Pegasus nose)
    ("Rasalhague", 266.0, 36.0, 2.07, "#FFFFFF"), // Alpha Oph (Ophiuchus)
    ("Eltanin", 268.0, 75.0, 2.23, "#FFD27F"), // Gamma Dra (Draco)
    ("Thuban", 234.0, 66.0, 3.67, "#FFFFFF"), // Alpha Dra (ancient pole star)
    ("Menkent", 218.0, -25.0, 2.06, "#FFD27F"), // Theta Cen
];

// Galactic plane passes through these ecliptic longitudes with varying latitudes
// The Milky Way band crosses the ecliptic at roughly 90° and 270° longitude
const GALACTIC_PLANE: &[(f64, f64)] = &[
    // (ecliptic_lon, ecliptic_lat) - approximate galactic plane in ecliptic coords
    (0.0, 60.0),
    (30.0, 45.0),
    (60.0, 30.0),
    (90.0, 0.0), // Galactic plane dips to ecliptic
    (120.0, -30.0),
    (150.0, -50.0),
    (180.0, -60.0), // Below ecliptic
    (210.0, -50.0),
    (240.0, -30.0),
    (270.0, 0.0), // Rises back through ecliptic
    (300.0, 30.0),
    (330.0, 50.0),
    (360.0, 60.0), // Above ecliptic
];

// Solar apex direction (where the Sun is moving through the local interstellar medium)
// Near Vega, in the direction of Hercules constellation
// Ecliptic coordinates: ~270° longitude, ~53° latitude
const SOLAR_APEX_LON: f64 = 270.0;
const SOLAR_APEX_LAT: f64 = 53.0;

// Galactic center direction in ecliptic coordinates
// The galactic center (Sagittarius A*) in ecliptic coords
const GALACTIC_CENTER_LON: f64 = 266.4;
const GALACTIC_CENTER_LAT: f64 = -5.5;

fn draw_starfield(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    // DETERMINISTIC ONLY: All stars from database with true 3D positions
    // No random procedural stars - pure dataset-driven rendering
    draw_bright_stars(ctx, state, time);
}

/// Interpolate galactic plane latitude at given ecliptic longitude
fn interpolate_galactic_lat(lon: f64) -> f64 {
    let lon = lon % 360.0;
    // Find bracketing points
    for i in 0..GALACTIC_PLANE.len() - 1 {
        let (l1, lat1) = GALACTIC_PLANE[i];
        let (l2, lat2) = GALACTIC_PLANE[i + 1];
        if lon >= l1 && lon < l2 {
            let t = (lon - l1) / (l2 - l1);
            return lat1 + t * (lat2 - lat1);
        }
    }
    // Wrap around
    let (l1, lat1) = GALACTIC_PLANE[GALACTIC_PLANE.len() - 1];
    let (l2, lat2) = GALACTIC_PLANE[0];
    let t = (lon - l1) / ((l2 + 360.0) - l1);
    lat1 + t * (lat2 - lat1)
}

/// Draw faint background stars with natural random distribution
fn draw_background_stars(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let w = state.view.width;
    let h = state.view.height;

    // Subtle drift for NASA Eyes effect - stars slowly move to give sense of motion
    let drift_speed = if state.view.zoom < 0.001 {
        0.5 // Slow drift when zoomed in close
    } else {
        0.1 // Very subtle at normal zoom
    };

    let time_drift = time * drift_speed;

    // Star count based on screen area
    let star_count = 400;

    ctx.set_fill_style(&JsValue::from_str("white"));

    // Use a hash-like function for better random distribution
    // This avoids visible patterns from simple modulo operations
    for i in 0..star_count {
        // Better pseudo-random using multiple primes and trigonometric mixing
        let fi = i as f64;
        let hash1 = ((fi * 127.1 + 311.7).sin() * 43758.5453).fract();
        let hash2 = ((fi * 269.5 + 183.3).sin() * 43758.5453).fract();
        let hash3 = ((fi * 419.2 + 371.9).sin() * 43758.5453).fract();

        // Base position from hash (0 to 1 range)
        let base_x = hash1;
        let base_y = hash2;

        // Apply subtle time-based drift (wrapping)
        let drift_x = (time_drift * (0.3 + hash3 * 0.7)) % 1.0;
        let drift_y = (time_drift * 0.2 * hash1) % 1.0;

        // Final position on screen with wrapping
        let x = ((base_x + drift_x) % 1.0) * w;
        let y = ((base_y + drift_y) % 1.0) * h;

        // Brightness follows realistic distribution (more faint stars than bright)
        let brightness = hash3 * hash3 * 0.8 + 0.1;

        // Subtle twinkle
        let twinkle = 0.85 + ((time * (0.5 + hash1 * 1.5) + fi * 0.1).sin() * 0.15);
        let alpha = brightness * twinkle;

        // Size based on brightness
        let size = 0.3 + brightness * 1.2;

        // Color variation for brighter stars
        if brightness > 0.5 {
            let color_hash = (hash1 * 5.0) as usize % 5;
            let colors = ["#FFFFFF", "#FFE8D0", "#D0E8FF", "#FFFAD0", "#F0E8FF"];
            ctx.set_fill_style(&JsValue::from_str(colors[color_hash]));
        } else {
            ctx.set_fill_style(&JsValue::from_str("white"));
        }

        ctx.set_global_alpha(alpha);
        ctx.begin_path();
        ctx.arc(x, y, size, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }

    ctx.set_global_alpha(1.0);
}

/// Calculate star visibility with smooth LOD fading
/// Returns 0.0-1.0 where 1.0 = fully visible, 0.0 = not visible
#[inline]
fn star_visibility(magnitude: f64, mag_limit: f64) -> f64 {
    const FADE_RANGE: f64 = 0.5; // Fade over 0.5 magnitude range
    if magnitude >= mag_limit {
        0.0
    } else if magnitude > mag_limit - FADE_RANGE {
        // Smooth fade near the limit
        (mag_limit - magnitude) / FADE_RANGE
    } else {
        1.0
    }
}

/// Draw bright named stars with true 3D positions from StarDatabase,
/// filtered through the StarDataManager to enforce LOD and a hard cap
/// on the number of stars drawn per frame.
fn draw_bright_stars(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let width = state.view.width;
    let height = state.view.height;

    // Magnitude limit based on scale level (with extra margin for fading)
    let mag_limit = state.camera.scale_level.star_magnitude_limit();

    for inst in state.star_mgr.visible_instances() {
        // Calculate visibility with LOD fading
        let visibility = star_visibility(inst.magnitude as f64, mag_limit);
        if visibility <= 0.0 {
            continue;
        }

        // Project star's 3D position to screen
        let (sx, sy, depth) = state.project_3d(inst.position.x, inst.position.y, inst.position.z);

        // Skip if off screen (with margin for glows)
        if sx < -50.0 || sx > width + 50.0 || sy < -50.0 || sy > height + 50.0 {
            continue;
        }

        // Skip if behind camera
        if depth < 0.0 {
            continue;
        }

        // Get star color from instance
        let color = format!(
            "#{:02x}{:02x}{:02x}",
            inst.color_rgb[0], inst.color_rgb[1], inst.color_rgb[2]
        );

        // NASA Eyes style: Simple, clean, subtle
        // Size: small points with slight variation by brightness
        let core_size = (6.0 - inst.magnitude as f64).max(1.5).min(3.0);
        let glow_radius = core_size * 1.8;

        // Subtle twinkling (very gentle, like NASA Eyes)
        let twinkle = 0.95 + ((time * 1.0 + inst.id as f64 * 0.1).sin() * 0.05);

        // Apply visibility and twinkle
        let base_alpha = twinkle * visibility;

        // Subtle glow (NASA Eyes style - not overdone)
        if inst.magnitude < 4.0 {
            ctx.set_global_alpha(base_alpha * 0.3);

            if let Ok(gradient) = ctx.create_radial_gradient(sx, sy, 0.0, sx, sy, glow_radius) {
                gradient
                    .add_color_stop(0.0, &format!("rgba({},{},{},0.8)", inst.color_rgb[0], inst.color_rgb[1], inst.color_rgb[2]))
                    .ok();
                gradient
                    .add_color_stop(0.6, &format!("rgba({},{},{},0.2)", inst.color_rgb[0], inst.color_rgb[1], inst.color_rgb[2]))
                    .ok();
                gradient
                    .add_color_stop(1.0, "rgba(255,255,255,0.0)")
                    .ok();

                ctx.set_fill_style(&gradient);
                ctx.begin_path();
                ctx.arc(sx, sy, glow_radius, 0.0, 2.0 * PI).unwrap_or(());
                ctx.fill();
            }
        }

        // Core point (clean and simple like NASA Eyes)
        ctx.set_global_alpha(base_alpha);
        ctx.set_fill_style(&JsValue::from_str(&color));
        ctx.begin_path();
        ctx.arc(sx, sy, core_size * 0.5, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();

        // Draw star name at heliosphere scale (only for fully visible stars)
        if visibility > 0.9 && state.view.zoom > 0.8 && inst.magnitude < 0.5 && inst.name.is_some()
        {
            ctx.set_font("500 10px 'Just Sans', sans-serif");
            ctx.set_fill_style(&JsValue::from_str("rgba(200, 200, 255, 0.6)"));
            ctx.fill_text(&inst.name.as_ref().unwrap(), sx + core_size + 3.0, sy + 3.0)
                .unwrap_or(());
        }
    }

    ctx.set_global_alpha(1.0);
}

// Removed: draw_additional_stars - used celestial_to_screen

/// Draw constellation stick figures connecting bright stars
fn draw_constellations(ctx: &CanvasRenderingContext2d, state: &SimulationState) {
    // Create lookup map from star ID to position
    let mut star_positions = std::collections::HashMap::new();
    for inst in state.star_mgr.visible_instances() {
        star_positions.insert(inst.id, inst.position);
    }

    // Draw constellation lines from the manager's data
    for edge in state.star_mgr.constellation_edges() {
        // Look up star positions from visible instances
        let star_a_pos = star_positions.get(&edge.star_a);
        let star_b_pos = star_positions.get(&edge.star_b);

        if let (Some(pos_a), Some(pos_b)) = (star_a_pos, star_b_pos) {
            // Project to screen using 3D projection
            let (sx1, sy1, _) = state.project_3d(pos_a.x, pos_a.y, pos_a.z);
            let (sx2, sy2, _) = state.project_3d(pos_b.x, pos_b.y, pos_b.z);

            // Draw line
            ctx.set_stroke_style(&JsValue::from_str("rgba(100, 150, 255, 0.35)"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(sx1, sy1);
            ctx.line_to(sx2, sy2);
            ctx.stroke();
        }
    }

    // Draw constellation labels (simplified - could be enhanced later)
    if state.view.zoom > 0.6 {
        // For now, skip constellation labels since we removed the metadata
        // Could add back if needed from manager
    }

    // Asterisms removed for now - used celestial_to_screen
}

// Removed: draw_constellation_labels - used celestial_to_screen

// Removed: draw_asterisms - used celestial_to_screen

// Removed: draw_asterism_triangle - used celestial_to_screen

/// Draw a dashed line manually
fn draw_dashed_line(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    dash_len: f64,
    gap_len: f64,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1.0 {
        return;
    }

    let ux = dx / dist;
    let uy = dy / dist;

    let mut pos = 0.0;
    let mut drawing = true;

    ctx.begin_path();
    ctx.move_to(x1, y1);

    while pos < dist {
        let seg_len = if drawing { dash_len } else { gap_len };
        pos += seg_len;
        let end_pos = pos.min(dist);

        let x = x1 + ux * end_pos;
        let y = y1 + uy * end_pos;

        if drawing {
            ctx.line_to(x, y);
        } else {
            ctx.move_to(x, y);
        }
        drawing = !drawing;
    }
    ctx.stroke();
}

// Removed: draw_winter_hexagon - used celestial_to_screen

// Removed: draw_celestial_directions - used celestial_to_screen

// ============================================================================
// HELIOSPHERE BOUNDARIES
// ============================================================================

// The Sun moves through the Local Interstellar Medium (LISM) at about 26 km/s
// The heliosphere has a comet-like shape:
// - "Nose" faces the direction of motion (toward solar apex, ~255° ecliptic longitude)
// - "Tail" (heliotail) extends in the opposite direction, possibly 1000+ AU
// The heliosphere direction in our coordinate system: Sun moves toward -X direction
// (We orient the view so the interstellar "wind" comes from the right, tail goes left)

const HELIO_NOSE_DIRECTION: f64 = std::f64::consts::PI; // Nose points in -X direction

fn draw_heliosphere_boundaries(ctx: &CanvasRenderingContext2d, state: &SimulationState, _time: f64) {
    let view = &state.view;

    // Only draw if zoomed out enough
    if view.zoom < 0.5 {
        return;
    }

    // Draw interstellar wind streamlines first (behind boundaries)
    if view.zoom > 0.8 {
        draw_interstellar_wind(ctx, state);
    }

    // Breathing effect linked to solar activity
    let _activity = (state.solar_cycle_phase * 2.0 * PI).sin() * 0.5 + 0.5;
    let boundary_breath = 1.0;

    // Draw boundaries from outermost to innermost with breathing
    // Bow shock - may not exist (debated), drawn faintly
    draw_comet_boundary(
        ctx,
        state,
        state.bow_shock_au * boundary_breath,
        0.5,
        3.0,
        "rgba(231, 76, 60, 0.08)",
        "rgba(231, 76, 60, 0.15)",
        1.0,
    );

    // Heliopause - the boundary between solar wind and interstellar medium
    draw_comet_boundary(
        ctx,
        state,
        state.heliopause_au * boundary_breath,
        0.6,
        2.5,
        "rgba(155, 89, 182, 0.1)",
        "rgba(155, 89, 182, 0.2)",
        1.5,
    );

    // Termination shock - where solar wind slows to subsonic speeds
    draw_comet_boundary(
        ctx,
        state,
        state.termination_shock_au * boundary_breath,
        0.7,
        2.0,
        "rgba(52, 152, 219, 0.12)",
        "rgba(52, 152, 219, 0.25)",
        2.0,
    );

    // Labels with better positioning
    if view.zoom > 0.8 {
        let lod = view.lod_level();
        if lod == 0 {
            // Label positions - on the nose side
            draw_helio_label(
                ctx,
                state,
                state.termination_shock_au * 0.75,
                -0.3,
                "Termination Shock",
                "rgba(52, 152, 219, 0.8)",
            );
            draw_helio_label(
                ctx,
                state,
                state.heliopause_au * 0.75,
                -0.15,
                "Heliopause",
                "rgba(155, 89, 182, 0.8)",
            );
            draw_helio_label(
                ctx,
                state,
                state.bow_shock_au * 0.6,
                0.0,
                "Bow Shock (?)",
                "rgba(231, 76, 60, 0.7)",
            );

            // Tail label
            draw_helio_label(
                ctx,
                state,
                -state.heliopause_au * 1.5,
                0.0,
                "Heliotail",
                "rgba(100, 150, 200, 0.6)",
            );
        }
    }

    // Draw Voyager positions relative to boundaries
    draw_voyager_boundary_context(ctx, state);
}

/// Draw interstellar wind/medium flowing around the heliosphere
fn draw_interstellar_wind(ctx: &CanvasRenderingContext2d, state: &SimulationState) {
    let view = &state.view;
    // Use unified projection for Sun position (object-centric camera)
    let (sun_x, sun_y, _) = state.project_3d(0.0, 0.0, 0.0);

    // The Local Interstellar Medium (LISM) flows at ~26 km/s
    // from the direction of the constellation Ophiuchus
    // In our view, it comes from the right (positive X) toward the Sun

    let heliopause_r = state.heliopause_au / view.zoom;
    let bow_shock_r = state.bow_shock_au / view.zoom;

    // Layer 1: Distant interstellar medium - subtle background flow
    ctx.set_stroke_style(&JsValue::from_str("rgba(100, 150, 200, 0.06)"));
    ctx.set_line_width(1.0);

    for i in 0..25 {
        let y_offset = (i as f64 - 12.0) * 40.0;
        let start_x = view.width + 100.0;
        let start_y = sun_y + y_offset;

        ctx.begin_path();
        ctx.move_to(start_x, start_y);

        for step in 0..60 {
            let t = step as f64 / 60.0;
            let x = start_x - t * (view.width + 200.0);

            let dx = x - sun_x;
            let dy = start_y - sun_y;
            let dist = (dx * dx + dy * dy).sqrt();

            // More realistic deflection around the heliosphere
            let deflection = if dist < bow_shock_r * 1.2 {
                let factor = (1.0 - dist / (bow_shock_r * 1.2)).max(0.0);
                let angle = dy.atan2(dx);
                factor * factor * dy.signum() * 80.0 * (1.0 + angle.abs() * 0.3)
            } else {
                0.0
            };

            let y = start_y + deflection;
            ctx.line_to(x, y);
        }
        ctx.stroke();
    }

    // Layer 2: Denser LISM particles approaching heliosphere
    ctx.set_global_alpha(0.15);

    let particle_count = 40;
    for i in 0..particle_count {
        let seed = i as f64 * 1.618;
        let y_base = sun_y + ((seed * 3.7) % 1.0 - 0.5) * view.height * 0.8;
        let x_phase = (seed * 2.3) % 1.0;

        // Particles moving from right to left
        let x = view.width * (1.0 - x_phase * 0.3) + 50.0;
        let y = y_base;

        let dist_to_sun = ((x - sun_x).powi(2) + (y - sun_y).powi(2)).sqrt();

        // Skip particles inside heliosphere
        if dist_to_sun < heliopause_r {
            continue;
        }

        let size = 1.5 + (seed * 1.1).sin().abs();
        let alpha = 0.4 - (x_phase * 0.3);

        ctx.set_fill_style(&JsValue::from_str(&format!(
            "rgba(150, 180, 220, {})",
            alpha
        )));
        ctx.begin_path();
        ctx.arc(x, y, size, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }

    ctx.set_global_alpha(1.0);

    // Layer 3: Direction indicator with arrow and label
    draw_interstellar_wind_indicator(ctx, state);
}

/// Draw direction indicator showing where interstellar wind comes from
fn draw_interstellar_wind_indicator(ctx: &CanvasRenderingContext2d, state: &SimulationState) {
    let view = &state.view;

    // Position indicator at right edge of screen
    let indicator_x = view.width - 120.0;
    let indicator_y = view.height * 0.15;

    // Arrow pointing left (direction of LISM flow)
    let arrow_len = 60.0;
    let arrow_head = 12.0;

    ctx.set_stroke_style(&JsValue::from_str("rgba(150, 200, 255, 0.6)"));
    ctx.set_fill_style(&JsValue::from_str("rgba(150, 200, 255, 0.6)"));
    ctx.set_line_width(2.0);

    // Arrow shaft
    ctx.begin_path();
    ctx.move_to(indicator_x, indicator_y);
    ctx.line_to(indicator_x - arrow_len, indicator_y);
    ctx.stroke();

    // Arrow head
    ctx.begin_path();
    ctx.move_to(indicator_x - arrow_len, indicator_y);
    ctx.line_to(
        indicator_x - arrow_len + arrow_head,
        indicator_y - arrow_head * 0.5,
    );
    ctx.line_to(
        indicator_x - arrow_len + arrow_head,
        indicator_y + arrow_head * 0.5,
    );
    ctx.close_path();
    ctx.fill();

    // Labels
    ctx.set_font("600 11px 'Just Sans', sans-serif");
    ctx.set_fill_style(&JsValue::from_str("rgba(150, 200, 255, 0.8)"));
    ctx.fill_text(
        "Interstellar Wind",
        indicator_x - arrow_len - 5.0,
        indicator_y - 15.0,
    )
    .unwrap_or(());

    ctx.set_font("400 9px 'Just Sans', sans-serif");
    ctx.set_fill_style(&JsValue::from_str("rgba(150, 200, 255, 0.6)"));
    ctx.fill_text(
        "~26 km/s from Ophiuchus",
        indicator_x - arrow_len - 5.0,
        indicator_y + 18.0,
    )
    .unwrap_or(());

    // Small info about hydrogen wall
    ctx.set_font("300 8px 'Just Sans', sans-serif");
    ctx.fill_text(
        "(Local Interstellar Cloud)",
        indicator_x - arrow_len - 5.0,
        indicator_y + 30.0,
    )
    .unwrap_or(());
}

/// Draw a 3D comet-shaped heliosphere boundary as a wireframe sphere
/// This creates true 3D depth perception by drawing latitude/longitude lines
/// that respond properly to camera rotation and tilt
/// nose_factor: how close to Sun the nose is (0.5 = 50% of radius)
/// tail_factor: how far the tail extends (2.0 = 2x radius)
fn draw_comet_boundary(
    ctx: &CanvasRenderingContext2d,
    state: &SimulationState,
    radius_au: f64,
    nose_factor: f64,
    tail_factor: f64,
    fill_color: &str,
    stroke_color: &str,
    line_width: f64,
) {
    let view = &state.view;
    let r_pixels = radius_au / view.zoom;

    // Use unified projection function for object-centric camera
    let project = |x: f64, y: f64, z: f64| -> (f64, f64, f64) { state.project_3d(x, y, z) };

    // Don't draw if too small or too large
    if r_pixels < 10.0 || r_pixels > view.width * 3.0 {
        return;
    }

    // Parse colors for alpha manipulation
    let (base_r, base_g, base_b, base_a) = parse_rgba(stroke_color);
    let (fill_r, fill_g, fill_b, fill_a) = parse_rgba(fill_color);

    // Number of segments for smooth curves
    let lat_segments = 8; // Latitude lines (horizontal rings)
    let lon_segments = 12; // Longitude lines (vertical arcs)
    let points_per_line = 48; // Points per line for smooth curves

    // Draw filled background (subtle 3D shell effect)
    // Draw multiple latitude bands with depth-based opacity
    for lat_idx in 0..lat_segments {
        let lat1 = PI * (lat_idx as f64 / lat_segments as f64) - PI / 2.0;
        let lat2 = PI * ((lat_idx + 1) as f64 / lat_segments as f64) - PI / 2.0;

        ctx.begin_path();

        // First half of the band (going around)
        for i in 0..=points_per_line {
            let lon = 2.0 * PI * (i as f64 / points_per_line as f64);
            let (x, y, z) = heliosphere_point(radius_au, lat1, lon, nose_factor, tail_factor);
            let (sx, sy, _) = project(x, y, z);

            if i == 0 {
                ctx.move_to(sx, sy);
            } else {
                ctx.line_to(sx, sy);
            }
        }

        // Second half (coming back at lat2)
        for i in (0..=points_per_line).rev() {
            let lon = 2.0 * PI * (i as f64 / points_per_line as f64);
            let (x, y, z) = heliosphere_point(radius_au, lat2, lon, nose_factor, tail_factor);
            let (sx, sy, _) = project(x, y, z);
            ctx.line_to(sx, sy);
        }

        ctx.close_path();

        // Calculate average depth for this band
        let mid_lat = (lat1 + lat2) / 2.0;
        let (_, _, depth) = project(0.0, radius_au * mid_lat.cos(), radius_au * mid_lat.sin());

        // Depth-based alpha (back faces are dimmer)
        let depth_factor = (depth / (radius_au * 2.0) + 0.5).clamp(0.2, 1.0);
        let band_alpha = fill_a * depth_factor * 0.5;

        ctx.set_fill_style(&JsValue::from_str(&format!(
            "rgba({}, {}, {}, {})",
            fill_r, fill_g, fill_b, band_alpha
        )));
        ctx.fill();
    }

    // Draw latitude lines (horizontal rings at different elevations)
    for lat_idx in 0..=lat_segments {
        let lat = PI * (lat_idx as f64 / lat_segments as f64) - PI / 2.0;

        // Skip poles (they collapse to points)
        if lat.abs() > PI / 2.0 - 0.1 {
            continue;
        }

        ctx.begin_path();
        let mut first = true;
        let mut prev_depth = 0.0;

        for i in 0..=points_per_line {
            let lon = 2.0 * PI * (i as f64 / points_per_line as f64);
            let (x, y, z) = heliosphere_point(radius_au, lat, lon, nose_factor, tail_factor);
            let (sx, sy, depth) = project(x, y, z);

            if first {
                ctx.move_to(sx, sy);
                first = false;
                prev_depth = depth;
            } else {
                // Depth-based opacity for this segment
                let avg_depth = (depth + prev_depth) / 2.0;
                let depth_factor = (avg_depth / (radius_au * 2.0) + 0.5).clamp(0.3, 1.0);

                // Finish previous segment
                ctx.line_to(sx, sy);

                // Set line style based on depth
                let segment_alpha = base_a * depth_factor;
                ctx.set_stroke_style(&JsValue::from_str(&format!(
                    "rgba({}, {}, {}, {})",
                    base_r, base_g, base_b, segment_alpha
                )));
                ctx.set_line_width(line_width * depth_factor);
                ctx.stroke();

                // Start new segment
                ctx.begin_path();
                ctx.move_to(sx, sy);
                prev_depth = depth;
            }
        }
    }

    // Draw longitude lines (vertical arcs from pole to pole)
    for lon_idx in 0..lon_segments {
        let lon = 2.0 * PI * (lon_idx as f64 / lon_segments as f64);

        ctx.begin_path();
        let mut first = true;
        let mut prev_depth = 0.0;

        for i in 0..=points_per_line {
            let lat = PI * (i as f64 / points_per_line as f64) - PI / 2.0;
            let (x, y, z) = heliosphere_point(radius_au, lat, lon, nose_factor, tail_factor);
            let (sx, sy, depth) = project(x, y, z);

            if first {
                ctx.move_to(sx, sy);
                first = false;
                prev_depth = depth;
            } else {
                let avg_depth = (depth + prev_depth) / 2.0;
                let depth_factor = (avg_depth / (radius_au * 2.0) + 0.5).clamp(0.3, 1.0);

                ctx.line_to(sx, sy);

                let segment_alpha = base_a * depth_factor;
                ctx.set_stroke_style(&JsValue::from_str(&format!(
                    "rgba({}, {}, {}, {})",
                    base_r, base_g, base_b, segment_alpha
                )));
                ctx.set_line_width(line_width * depth_factor);
                ctx.stroke();

                ctx.begin_path();
                ctx.move_to(sx, sy);
                prev_depth = depth;
            }
        }
    }

    // Draw equatorial ring (more prominent)
    ctx.begin_path();
    for i in 0..=points_per_line {
        let lon = 2.0 * PI * (i as f64 / points_per_line as f64);
        let (x, y, z) = heliosphere_point(radius_au, 0.0, lon, nose_factor, tail_factor);
        let (sx, sy, _) = project(x, y, z);

        if i == 0 {
            ctx.move_to(sx, sy);
        } else {
            ctx.line_to(sx, sy);
        }
    }
    ctx.close_path();
    ctx.set_stroke_style(&JsValue::from_str(&format!(
        "rgba({}, {}, {}, {})",
        base_r,
        base_g,
        base_b,
        base_a * 1.5
    )));
    ctx.set_line_width(line_width * 1.5);
    ctx.stroke();
}

/// Calculate a point on the heliosphere surface
/// Uses spherical coordinates with asymmetric scaling for comet shape
/// lat: latitude (-PI/2 to PI/2, 0 = equator)
/// lon: longitude (0 to 2*PI)
#[inline]
fn heliosphere_point(
    radius: f64,
    lat: f64,
    lon: f64,
    nose_factor: f64,
    tail_factor: f64,
) -> (f64, f64, f64) {
    // Basic spherical to Cartesian
    let cos_lat = lat.cos();
    let sin_lat = lat.sin();
    let cos_lon = lon.cos();
    let sin_lon = lon.sin();

    // Base sphere coordinates (before asymmetric scaling)
    let x_sphere = cos_lat * cos_lon;
    let y_sphere = cos_lat * sin_lon;
    let z_sphere = sin_lat;

    // Apply comet-like asymmetric scaling
    // The heliosphere is compressed on the nose (positive X) and extended on the tail (negative X)
    // nose_factor < 1 compresses the nose, tail_factor > 1 extends the tail
    let x_scale = if x_sphere > 0.0 {
        // Nose side - compress
        nose_factor + (1.0 - nose_factor) * (1.0 - x_sphere)
    } else {
        // Tail side - extend
        1.0 + (tail_factor - 1.0) * (-x_sphere)
    };

    // Apply the nose direction rotation (heliosphere points in specific direction)
    let nose_dir = HELIO_NOSE_DIRECTION;
    let cos_nose = nose_dir.cos();
    let sin_nose = nose_dir.sin();

    // Unrotated position with asymmetric scaling
    let x_scaled = x_sphere * x_scale * radius;
    let y_scaled = y_sphere * radius;
    let z_scaled = z_sphere * radius;

    // Rotate to align with heliosphere nose direction
    let x = x_scaled * cos_nose - y_scaled * sin_nose;
    let y = x_scaled * sin_nose + y_scaled * cos_nose;
    let z = z_scaled;

    (x, y, z)
}

/// Parse rgba color string to components
#[inline]
fn parse_rgba(color: &str) -> (u8, u8, u8, f64) {
    // Handle "rgba(r, g, b, a)" format
    if color.starts_with("rgba(") {
        let inner = color.trim_start_matches("rgba(").trim_end_matches(')');
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4 {
            let r = parts[0].trim().parse().unwrap_or(255);
            let g = parts[1].trim().parse().unwrap_or(255);
            let b = parts[2].trim().parse().unwrap_or(255);
            let a = parts[3].trim().parse().unwrap_or(1.0);
            return (r, g, b, a);
        }
    }
    // Default fallback
    (255, 255, 255, 0.5)
}

/// Draw label for heliosphere boundaries
fn draw_helio_label(
    ctx: &CanvasRenderingContext2d,
    state: &SimulationState,
    x_au: f64,
    y_au: f64,
    label: &str,
    color: &str,
) {
    // Use unified projection for object-centric camera
    let (sx, sy, _) = state.project_3d(x_au, y_au, 0.0);

    ctx.set_font("500 11px 'Just Sans', monospace");
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.fill_text(label, sx, sy).unwrap_or(());
}

/// Draw context for Voyager spacecraft relative to heliosphere boundaries
fn draw_voyager_boundary_context(ctx: &CanvasRenderingContext2d, state: &SimulationState) {
    let view = &state.view;
    if view.zoom < 0.8 {
        return;
    }

    // Check if Voyagers are past the termination shock or heliopause
    for m in 0..state.mission_count {
        let name = state.mission_names[m];
        if !name.contains("Voyager") {
            continue;
        }

        let x = state.mission_x[m];
        let y = state.mission_y[m];
        let dist = (x * x + y * y).sqrt();

        // Draw status indicator (use unified projection)
        let (sx, sy, _) = state.project_3d(x, y, 0.0);

        let status = if dist > state.heliopause_au {
            ("INTERSTELLAR", "rgba(200, 100, 255, 0.8)")
        } else if dist > state.termination_shock_au {
            ("HELIOSHEATH", "rgba(100, 200, 255, 0.7)")
        } else {
            continue; // Don't label if still in inner heliosphere
        };

        ctx.set_font("300 8px 'Just Sans', monospace");
        ctx.set_fill_style(&JsValue::from_str(status.1));
        ctx.fill_text(status.0, sx + 12.0, sy + 20.0).unwrap_or(());
    }
}

// ============================================================================
// ORBIT PATHS
// ============================================================================

fn draw_orbits(ctx: &CanvasRenderingContext2d, state: &SimulationState, _time: f64) {
    let view = &state.view;

    for p in 0..state.planet_count {
        // Visibility check - is any part of orbit on screen?
        let orbit = &state.planet_orbits[p];
        let aphelion = orbit.a * (1.0 + orbit.e);

        if !view.is_visible(0.0, 0.0, aphelion) {
            continue;
        }

        let path = &state.orbit_paths[p];
        let hex_color = state.planet_colors[p].trim_start_matches('#');

        // Calculate depth range for this orbit to enable depth-based opacity
        let mut min_depth = f64::INFINITY;
        let mut max_depth = f64::NEG_INFINITY;
        for i in 0..ORBIT_SEGMENTS {
            let (_, _, depth) = state.project_3d(path[i * 3], path[i * 3 + 1], path[i * 3 + 2]);
            min_depth = min_depth.min(depth);
            max_depth = max_depth.max(depth);
        }
        let depth_range = (max_depth - min_depth).max(0.01);

        // Breathing glow - phase offset per orbit for cascading effect
        let orbit_breath = 1.0;
        let base_alpha = 0.25;

        // Draw orbit segments with depth-based opacity variation
        // This creates the illusion of 3D by making far parts of orbit dimmer
        let base_line_width = 1.0;

        // Draw segments individually with varying opacity based on depth
        for i in 0..ORBIT_SEGMENTS {
            let next_i = (i + 1) % ORBIT_SEGMENTS;

            let x1 = path[i * 3];
            let y1 = path[i * 3 + 1];
            let z1 = path[i * 3 + 2];
            let x2 = path[next_i * 3];
            let y2 = path[next_i * 3 + 1];
            let z2 = path[next_i * 3 + 2];

            let (sx1, sy1, depth1) = state.project_3d(x1, y1, z1);
            let (sx2, sy2, depth2) = state.project_3d(x2, y2, z2);

            // Calculate average depth for this segment
            let avg_depth = (depth1 + depth2) / 2.0;

            // Depth-based opacity: far segments are dimmer (0.5x to 1.0x)
            let normalized_depth = (avg_depth - min_depth) / depth_range;
            let depth_alpha_factor = 0.5 + normalized_depth * 0.5; // Far: 0.5x, Near: 1.0x

            // Depth-based line width: far segments slightly thinner
            let depth_width_factor = 0.7 + normalized_depth * 0.3; // Far: 0.7x, Near: 1.0x

            let segment_alpha = (base_alpha + orbit_breath * 0.15) * depth_alpha_factor;
            let segment_width = base_line_width * depth_width_factor;

            let alpha_hex = format!("{:02X}", (segment_alpha * 255.0).min(255.0) as u8);
            let color = format!("#{}{}", hex_color, alpha_hex);

            ctx.set_stroke_style(&JsValue::from_str(&color));
            ctx.set_line_width(segment_width);

            ctx.begin_path();
            ctx.move_to(sx1, sy1);
            ctx.line_to(sx2, sy2);
            ctx.stroke();
        }
    }
}

// ============================================================================
// SUN
// ============================================================================

fn draw_sun(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let view = &state.view;
    // Use CCA projection for sun position
    let (cx, cy, _) = state.project_3d(0.0, 0.0, 0.0);

    // Sun radius in pixels (with minimum size for visibility)
    let sun_radius_au = SOLAR_RADIUS_KM / AU_KM;
    let base_radius = (sun_radius_au / view.zoom).max(8.0);

    // Solar activity based on cycle phase (more flares/prominences at solar max)
    let activity = (state.solar_cycle_phase * 2.0 * PI).sin() * 0.5 + 0.5;

    // Multi-frequency breathing - organic, living star
    let breath = 1.0;
    let _corona_radius = base_radius * (2.5 + activity * 1.0) * breath;

    // Solar wind streamers (coronal streamers)
    if base_radius > 15.0 {
        draw_solar_wind(ctx, cx, cy, base_radius, time, activity);
    }

    // Solar prominences (arcs of plasma) - more during solar maximum
    if base_radius > 20.0 {
        let num_prominences = (2.0 + activity * 4.0) as i32;
        for i in 0..num_prominences {
            draw_solar_prominence(ctx, cx, cy, base_radius, time, i as f64, activity);
        }
    }

    // Sun body with limb darkening and core breathing
    let sun_breath = 1.0;
    let breathing_radius = base_radius * sun_breath;

    let body_gradient = ctx
        .create_radial_gradient(
            cx - breathing_radius * 0.2,
            cy - breathing_radius * 0.2,
            0.0,
            cx,
            cy,
            breathing_radius,
        )
        .unwrap();
    body_gradient.add_color_stop(0.0, "#FFFEF0").unwrap();
    body_gradient.add_color_stop(0.3, "#FFF8DC").unwrap();
    body_gradient.add_color_stop(0.6, "#FFE87C").unwrap();
    body_gradient.add_color_stop(0.85, "#FFD700").unwrap();
    body_gradient.add_color_stop(0.95, "#FFA500").unwrap();
    body_gradient.add_color_stop(1.0, "#FF6B00").unwrap();

    ctx.set_fill_style(&body_gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, breathing_radius, 0.0, 2.0 * PI)
        .unwrap_or(());
    ctx.fill();

    // Granulation (convection cells) - only when zoomed in
    if breathing_radius > 40.0 {
        draw_solar_granulation(ctx, cx, cy, breathing_radius, time);
    }

    // Sunspots - more during solar maximum
    if breathing_radius > 20.0 {
        let num_spots = (1.0 + activity * 6.0) as i32;
        draw_sunspots(ctx, cx, cy, breathing_radius, time, num_spots, activity);
    }

    // Active regions (bright faculae near sunspots)
    if breathing_radius > 30.0 && activity > 0.3 {
        draw_faculae(ctx, cx, cy, breathing_radius, time, activity);
    }

    // Label
    if view.zoom < 0.05 {
        ctx.set_font("700 14px 'Just Sans', sans-serif");
        ctx.set_fill_style(&JsValue::from_str("#FFD700"));
        ctx.fill_text("Sun", cx + breathing_radius + 5.0, cy + 5.0)
            .unwrap_or(());
    }
}

/// Draw solar wind streamers emanating from the sun
fn draw_solar_wind(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    radius: f64,
    time: f64,
    activity: f64,
) {
    // Draw multiple layers of solar wind for depth

    // Layer 1: Wide coronal streamers (equatorial)
    let num_streamers = 12;
    ctx.set_global_alpha(0.12);

    for i in 0..num_streamers {
        let base_angle = (i as f64 / num_streamers as f64) * 2.0 * PI;
        let wobble = (time * 0.2 + i as f64 * 0.5).sin() * 0.08;
        let angle = base_angle + wobble;

        // Streamers longer during solar minimum (equatorial), shorter during maximum
        let length_factor =
            4.0 + (1.0 - activity) * 2.0 + (time * 0.15 + i as f64 * 0.3).sin() * 0.8;
        let length = radius * length_factor;

        // Create tapered streamer
        let start_width = radius * 0.25;
        let end_width = radius * 0.05;

        let grad = ctx.create_linear_gradient(
            cx + angle.cos() * radius,
            cy + angle.sin() * radius,
            cx + angle.cos() * length,
            cy + angle.sin() * length,
        );
        grad.add_color_stop(0.0, "rgba(255, 220, 150, 0.6)")
            .unwrap();
        grad.add_color_stop(0.3, "rgba(255, 180, 100, 0.3)")
            .unwrap();
        grad.add_color_stop(0.6, "rgba(255, 140, 60, 0.15)")
            .unwrap();
        grad.add_color_stop(1.0, "rgba(255, 100, 30, 0)").unwrap();

        ctx.set_stroke_style(&grad);

        // Draw tapered streamer using multiple segments
        let segments = 8;
        for s in 0..segments {
            let t1 = s as f64 / segments as f64;
            let t2 = (s + 1) as f64 / segments as f64;
            let r1 = radius + (length - radius) * t1;
            let r2 = radius + (length - radius) * t2;
            let w = start_width * (1.0 - t1) + end_width * t1;

            ctx.set_line_width(w);
            ctx.begin_path();
            ctx.move_to(cx + angle.cos() * r1, cy + angle.sin() * r1);
            ctx.line_to(cx + angle.cos() * r2, cy + angle.sin() * r2);
            ctx.stroke();
        }
    }

    // Layer 2: Fast solar wind particles streaming outward
    let particle_count = (30.0 + activity * 40.0) as i32;
    ctx.set_global_alpha(0.25);

    for i in 0..particle_count {
        let seed = i as f64 * 2.718;
        let base_angle = (seed * 1.7) % (2.0 * PI);

        // Particles move outward over time
        let speed = 0.8 + (seed * 0.3).sin().abs() * 0.6;
        let time_offset = (time * speed + seed * 3.0) % 4.0;

        // Particle position along radial
        let dist = radius * (1.2 + time_offset * 1.5);
        let max_dist = radius * 5.5;

        if dist > max_dist {
            continue;
        }

        let x = cx + base_angle.cos() * dist;
        let y = cy + base_angle.sin() * dist;

        // Particle fades as it moves outward
        let fade = 1.0 - (dist - radius) / (max_dist - radius);
        let particle_alpha = fade * fade * 0.6;

        // Particle color varies with solar wind speed
        let is_fast = (seed * 2.1).sin() > 0.3;
        let color = if is_fast {
            format!("rgba(255, 200, 100, {})", particle_alpha)
        } else {
            format!("rgba(255, 150, 80, {})", particle_alpha * 0.7)
        };

        let size = 1.0 + fade * 2.0;

        ctx.set_fill_style(&JsValue::from_str(&color));
        ctx.begin_path();
        ctx.arc(x, y, size, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();

        // Draw small tail for faster particles
        if is_fast && fade > 0.5 {
            let tail_len = size * 3.0;
            ctx.set_stroke_style(&JsValue::from_str(&format!(
                "rgba(255, 180, 80, {})",
                particle_alpha * 0.3
            )));
            ctx.set_line_width(size * 0.5);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(
                x - base_angle.cos() * tail_len,
                y - base_angle.sin() * tail_len,
            );
            ctx.stroke();
        }
    }

    // Layer 3: CME (Coronal Mass Ejection) during high activity
    if activity > 0.6 {
        draw_cme(ctx, cx, cy, radius, time, activity);
    }

    ctx.set_global_alpha(1.0);
}

/// Draw a Coronal Mass Ejection (CME) - bubble of plasma erupting from Sun
fn draw_cme(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    radius: f64,
    time: f64,
    activity: f64,
) {
    // CME direction and timing
    let cme_cycle = 8.0; // seconds per CME cycle
    let cme_phase = (time % cme_cycle) / cme_cycle;

    // Only show CME during expansion phase
    if cme_phase > 0.7 {
        return;
    }

    let cme_angle = ((time * 0.05).floor() * 2.7) % (2.0 * PI); // Different angle each cycle
    let cme_size = radius * (0.3 + cme_phase * 2.5);
    let cme_dist = radius * (1.0 + cme_phase * 4.0);

    let cme_x = cx + cme_angle.cos() * cme_dist;
    let cme_y = cy + cme_angle.sin() * cme_dist;

    let alpha = (1.0 - cme_phase) * activity * 0.4;

    // CME bubble with gradient
    let cme_grad = ctx
        .create_radial_gradient(cme_x, cme_y, 0.0, cme_x, cme_y, cme_size)
        .ok();
    if let Some(grad) = cme_grad {
        grad.add_color_stop(0.0, &format!("rgba(255, 100, 50, {})", alpha * 0.8))
            .unwrap();
        grad.add_color_stop(0.4, &format!("rgba(255, 150, 80, {})", alpha * 0.4))
            .unwrap();
        grad.add_color_stop(0.7, &format!("rgba(255, 180, 100, {})", alpha * 0.2))
            .unwrap();
        grad.add_color_stop(1.0, "rgba(255, 200, 150, 0)").unwrap();

        ctx.set_fill_style(&grad);
        ctx.begin_path();
        ctx.arc(cme_x, cme_y, cme_size, 0.0, 2.0 * PI).unwrap_or(());
        ctx.fill();
    }

    // CME shock front (leading edge)
    ctx.set_stroke_style(&JsValue::from_str(&format!(
        "rgba(255, 220, 180, {})",
        alpha * 0.5
    )));
    ctx.set_line_width(2.0);
    ctx.begin_path();
    ctx.arc(cme_x, cme_y, cme_size, cme_angle - 0.8, cme_angle + 0.8)
        .unwrap_or(());
    ctx.stroke();
}

/// Draw solar prominences (plasma arcs)
fn draw_solar_prominence(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    radius: f64,
    time: f64,
    idx: f64,
    activity: f64,
) {
    let seed = idx * 3.14159 + time * 0.01;
    let base_angle = (seed * 2.7) % (2.0 * PI);

    // Only draw if on visible portion
    let vis = (seed * 1.3).sin();
    if vis < 0.0 {
        return;
    }

    let height = radius * (0.3 + (seed * 1.7).sin().abs() * 0.4) * activity;
    let width = radius * 0.15;

    ctx.save();
    ctx.translate(cx, cy).unwrap_or(());
    ctx.rotate(base_angle).unwrap_or(());

    // Prominence arc
    let prom_grad = ctx
        .create_radial_gradient(0.0, -radius - height * 0.5, 0.0, 0.0, -radius, height)
        .unwrap();
    prom_grad
        .add_color_stop(0.0, "rgba(255, 100, 50, 0.8)")
        .unwrap();
    prom_grad
        .add_color_stop(0.5, "rgba(255, 80, 30, 0.5)")
        .unwrap();
    prom_grad
        .add_color_stop(1.0, "rgba(255, 50, 0, 0)")
        .unwrap();

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
fn draw_solar_granulation(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    radius: f64,
    time: f64,
) {
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
fn draw_sunspots(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    radius: f64,
    time: f64,
    num_spots: i32,
    activity: f64,
) {
    for i in 0..num_spots {
        let seed = i as f64 * 2.71;
        let angle = (time * 0.02 + seed * 2.0) % (2.0 * PI);
        let dist = radius * (0.2 + (seed * 1.5).sin().abs() * 0.5);

        // Only draw spots on visible hemisphere
        if angle.cos() < -0.3 {
            continue;
        }

        let spot_x = cx + angle.cos() * dist;
        let spot_y = cy + angle.sin() * dist * 0.8; // Foreshortening
        let spot_r = radius * (0.03 + (seed * 3.0).sin().abs() * 0.05) * (0.5 + activity);

        // Foreshorten near limb
        let limb_factor = (1.0 - (dist / radius).powi(2)).sqrt().max(0.3);
        let drawn_r = spot_r * limb_factor;

        // Penumbra (outer, lighter)
        ctx.set_fill_style(&JsValue::from_str("rgba(140, 80, 20, 0.5)"));
        ctx.begin_path();
        ctx.arc(spot_x, spot_y, drawn_r * 1.5, 0.0, 2.0 * PI)
            .unwrap_or(());
        ctx.fill();

        // Umbra (inner, darker)
        ctx.set_fill_style(&JsValue::from_str("rgba(60, 30, 10, 0.7)"));
        ctx.begin_path();
        ctx.arc(spot_x, spot_y, drawn_r, 0.0, 2.0 * PI)
            .unwrap_or(());
        ctx.fill();
    }
}

/// Draw bright faculae (active regions near sunspots)
fn draw_faculae(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    radius: f64,
    time: f64,
    activity: f64,
) {
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 220, 0.2)"));

    for i in 0..4 {
        let seed = i as f64 * 4.17;
        let angle = (time * 0.02 + seed * 2.5) % (2.0 * PI);
        let dist = radius * (0.6 + (seed * 1.2).sin().abs() * 0.3);

        // Faculae are more visible near the limb
        if angle.cos().abs() > 0.5 {
            continue;
        }

        let fac_x = cx + angle.cos() * dist;
        let fac_y = cy + angle.sin() * dist * 0.9;
        let fac_r = radius * 0.08 * activity;

        ctx.begin_path();
        ctx.ellipse(fac_x, fac_y, fac_r, fac_r * 0.6, angle, 0.0, 2.0 * PI)
            .unwrap_or(());
        ctx.fill();
    }
}

// ============================================================================
// PLANETS
// ============================================================================

/// Apply planet-specific breathing based on physical character
/// Each planet has unique frequency and amplitude for organic variety
fn apply_planet_breathing(base_radius: f64, time: f64, planet_idx: usize) -> f64 {
    // Planet-specific breathing parameters (frequency, amplitude, phase offset)
    // Gas giants breathe slower/stronger, rocky planets faster/subtler
    let (frequency, amplitude, phase) = match planet_idx {
        0 => (1.2, 0.02, 0.0),   // Mercury - small, active
        1 => (0.6, 0.03, 0.5),   // Venus - thick atmosphere
        2 => (0.5, 0.025, 1.0),  // Earth - baseline
        3 => (0.7, 0.02, 1.5),   // Mars - thin atmosphere
        4 => (0.2, 0.04, 2.0),   // Jupiter - massive, slow rhythm
        5 => (0.25, 0.035, 2.5), // Saturn - large gas giant
        6 => (0.3, 0.03, 3.0),   // Uranus - ice giant
        7 => (0.35, 0.03, 3.5),  // Neptune - ice giant
        _ => (0.5, 0.025, 0.0),  // Default
    };

    base_radius * breath_factor(time, frequency, amplitude, phase)
}

fn draw_planets(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let view = &state.view;
    let _lod = view.lod_level();

    // Collect planet data with depth for sorting (back to front rendering)
    // This provides proper depth ordering for 3D perception
    let mut planets_with_depth: Vec<(usize, f64, f64, f64, f64)> =
        Vec::with_capacity(state.planet_count);

    for p in 0..state.planet_count {
        let x = state.planet_x[p];
        let y = state.planet_y[p];
        let z = state.planet_z[p];

        // Use CCA projection for proper 3D view
        let (sx, sy, depth) = state.project_3d(x, y, z);

        // Planet radius in pixels (with minimum for visibility)
        // NASA Eyes style: allow very large planets when zoomed in close
        let radius_au = state.planet_radii_km[p] / AU_KM;
        let base_radius = (radius_au / view.zoom).max(4.0).min(500.0); // Allow large planets

        // Screen-based visibility check (more reliable at extreme zoom)
        let margin = base_radius + 100.0;
        if sx < -margin || sx > view.width + margin || sy < -margin || sy > view.height + margin {
            continue;
        }

        planets_with_depth.push((p, sx, sy, depth, base_radius));
    }

    // Sort by depth (furthest first = smallest depth first for back-to-front rendering)
    planets_with_depth.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate depth range for perspective effects
    let (min_depth, max_depth) = if !planets_with_depth.is_empty() {
        let min = planets_with_depth
            .iter()
            .map(|p| p.3)
            .fold(f64::INFINITY, f64::min);
        let max = planets_with_depth
            .iter()
            .map(|p| p.3)
            .fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    } else {
        (0.0, 1.0)
    };
    let depth_range = (max_depth - min_depth).max(0.1); // Avoid division by zero

    // Draw planets back-to-front with depth-based effects
    for (p, sx, sy, depth, base_radius) in planets_with_depth {
        // Depth-based size scaling (perspective effect)
        // Objects further away appear slightly smaller, closer ones larger
        // Use subtle effect: 0.85x to 1.15x based on depth position
        let depth_scale_factor = if depth_range > 0.01 {
            let normalized_depth = (depth - min_depth) / depth_range; // 0 = far, 1 = near
            0.85 + normalized_depth * 0.30 // Far: 0.85x, Near: 1.15x
        } else {
            1.0
        };

        // Atmospheric perspective: distant objects slightly faded
        // Creates depth cues by making far objects appear hazier
        let alpha_factor = if depth_range > 0.01 {
            let normalized_depth = (depth - min_depth) / depth_range;
            0.75 + normalized_depth * 0.25 // Far: 0.75 alpha, Near: 1.0 alpha
        } else {
            1.0
        };

        // Apply planet-specific breathing and depth scaling
        let breathing_radius = apply_planet_breathing(base_radius, time, p);
        let scaled_radius = breathing_radius * depth_scale_factor;

        ctx.set_global_alpha(alpha_factor);

        let color = state.planet_colors[p];

        // Always draw detailed 3D planets (no simple circles)
        draw_planet_detailed(
            ctx,
            sx,
            sy,
            scaled_radius,
            color,
            state.planet_has_rings[p],
            time,
            p,
        );

        // Draw depth indicator line (shows height above/below ecliptic plane)
        // Only at intermediate zoom levels where orbital structure is visible
        if state.planet_z[p].abs() > 0.01 && view.zoom > 0.005 && view.zoom < 0.5 {
            draw_ecliptic_height_indicator(ctx, state, p, sx, sy, scaled_radius);
        }
    }

    ctx.set_global_alpha(1.0);
}

/// Draw vertical line showing planet's height above/below the ecliptic plane
/// This provides crucial depth perception by showing the Z-axis visually
fn draw_ecliptic_height_indicator(
    ctx: &CanvasRenderingContext2d,
    state: &SimulationState,
    planet_idx: usize,
    _screen_x: f64,
    screen_y: f64,
    planet_radius: f64,
) {
    let z = state.planet_z[planet_idx];

    // Calculate where the planet would be if it were in the ecliptic plane (z=0)
    let x = state.planet_x[planet_idx];
    let y = state.planet_y[planet_idx];
    let (ecliptic_x, ecliptic_y, _) = state.project_3d(x, y, 0.0);

    // Draw a vertical line from the ecliptic plane position to the planet
    let color = state.planet_colors[planet_idx];

    // Line from ecliptic plane to planet
    ctx.set_stroke_style(&JsValue::from_str(&format!("{}60", color))); // 38% opacity
    ctx.set_line_width(1.0);

    // Dashed line style
    ctx.begin_path();
    let start_y = ecliptic_y;
    let end_y = screen_y
        - planet_radius.signum() * (planet_radius + 2.0) * if z > 0.0 { 1.0 } else { -1.0 };

    // Draw dashed line manually
    let dash_len = 4.0;
    let gap_len = 3.0;
    let dy = end_y - start_y;
    let dist = dy.abs();
    let dir = if dy > 0.0 { 1.0 } else { -1.0 };

    ctx.move_to(ecliptic_x, start_y);
    let mut pos = 0.0;
    let mut drawing = true;

    while pos < dist {
        let seg = if drawing { dash_len } else { gap_len };
        pos += seg;
        let curr_y = start_y + dir * pos.min(dist);

        if drawing {
            ctx.line_to(ecliptic_x, curr_y);
        } else {
            ctx.move_to(ecliptic_x, curr_y);
        }
        drawing = !drawing;
    }
    ctx.stroke();

    // Small circle at ecliptic intersection point
    ctx.set_fill_style(&JsValue::from_str(&format!("{}40", color)));
    ctx.begin_path();
    ctx.arc(ecliptic_x, ecliptic_y, 3.0, 0.0, 2.0 * PI)
        .unwrap_or(());
    ctx.fill();

    // Optional: show Z value label for significant inclinations
    if z.abs() > 0.1 {
        ctx.set_font("400 9px 'Just Sans', sans-serif");
        ctx.set_fill_style(&JsValue::from_str(&format!("{}80", color)));
        let z_label = if z > 0.0 {
            format!("+{:.2} AU", z)
        } else {
            format!("{:.2} AU", z)
        };
        let label_y = (start_y + screen_y) / 2.0;
        ctx.fill_text(&z_label, ecliptic_x + 5.0, label_y)
            .unwrap_or(());
    }
}

fn draw_planet_detailed(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    radius: f64,
    _color: &str,
    _has_rings: bool,
    time: f64,
    idx: usize,
) {
    // Planet-specific rendering based on index
    // 0=Mercury, 1=Venus, 2=Earth, 3=Mars, 4=Jupiter, 5=Saturn, 6=Uranus, 7=Neptune

    match idx {
        0 => draw_mercury(ctx, cx, cy, radius, time), // Mercury with craters
        1 => draw_venus(ctx, cx, cy, radius, time),   // Venus with thick atmosphere
        2 => draw_earth(ctx, cx, cy, radius, time),   // Earth with continents
        3 => draw_mars(ctx, cx, cy, radius, time),    // Mars with polar caps
        4 => draw_jupiter(ctx, cx, cy, radius, time), // Jupiter with bands and GRS
        5 => draw_saturn(ctx, cx, cy, radius, time),  // Saturn with detailed rings
        6 => draw_uranus(ctx, cx, cy, radius, time),  // Uranus with tilted rings
        7 => draw_neptune(ctx, cx, cy, radius, time), // Neptune with dark spot
        _ => {}
    }
}

/// Earth with blue oceans, green/brown continents, polar ice caps
fn draw_earth(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Ocean base
    let gradient = ctx
        .create_radial_gradient(cx - radius * 0.3, cy - radius * 0.3, 0.0, cx, cy, radius)
        .unwrap();
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
    ctx.arc(cx, cy - radius * 0.85, radius * 0.25, 0.0, 2.0 * PI)
        .unwrap_or(());
    ctx.fill();
    ctx.begin_path();
    ctx.arc(cx, cy + radius * 0.88, radius * 0.2, 0.0, 2.0 * PI)
        .unwrap_or(());
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
        ctx.ellipse(
            cloud_x,
            cloud_y,
            cloud_r,
            cloud_r * 0.4,
            angle * 0.5,
            0.0,
            2.0 * PI,
        )
        .unwrap_or(());
        ctx.fill();
    }

    ctx.restore();

    // Atmosphere glow
    let atmo = ctx
        .create_radial_gradient(cx, cy, radius * 0.95, cx, cy, radius * 1.15)
        .unwrap();
    atmo.add_color_stop(0.0, "rgba(100, 180, 255, 0)").unwrap();
    atmo.add_color_stop(0.5, "rgba(100, 180, 255, 0.15)")
        .unwrap();
    atmo.add_color_stop(1.0, "rgba(100, 180, 255, 0)").unwrap();
    ctx.set_fill_style(&atmo);
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 1.15, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Draw a blob-shaped continent
fn draw_continent_blob(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    radius: f64,
    longitude: f64,
    latitude: f64,
    width: f64,
    height: f64,
) {
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
    ctx.ellipse(x, y, w, h, longitude * 0.2, 0.0, 2.0 * PI)
        .unwrap_or(());
    ctx.fill();
}

/// Jupiter with cloud bands and Great Red Spot
fn draw_jupiter(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Base color
    let gradient = ctx
        .create_radial_gradient(cx - radius * 0.3, cy - radius * 0.3, 0.0, cx, cy, radius)
        .unwrap();
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
        "rgba(230, 200, 170, 0.5)", // Light zone
        "rgba(160, 110, 70, 0.5)",  // Dark belt
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
        ctx.fill_rect(
            cx - radius * 1.1,
            y_offset + wave,
            radius * 2.2,
            band_height * 1.1,
        );
    }

    // Great Red Spot
    let grs_rotation = time * 0.02;
    let grs_x = cx + grs_rotation.cos() * radius * 0.4;
    let grs_y = cy + radius * 0.2; // South of equator

    // Only draw if on visible side
    if grs_rotation.cos() > -0.3 {
        let grs_gradient = ctx
            .create_radial_gradient(grs_x, grs_y, 0.0, grs_x, grs_y, radius * 0.2)
            .unwrap();
        grs_gradient
            .add_color_stop(0.0, "rgba(200, 80, 60, 0.9)")
            .unwrap();
        grs_gradient
            .add_color_stop(0.5, "rgba(180, 70, 50, 0.7)")
            .unwrap();
        grs_gradient
            .add_color_stop(1.0, "rgba(160, 100, 80, 0)")
            .unwrap();

        ctx.set_fill_style(&grs_gradient);
        ctx.begin_path();
        ctx.ellipse(
            grs_x,
            grs_y,
            radius * 0.18,
            radius * 0.1,
            0.0,
            0.0,
            2.0 * PI,
        )
        .unwrap_or(());
        ctx.fill();
    }

    ctx.restore();

    // Subtle atmosphere
    let atmo = ctx
        .create_radial_gradient(cx, cy, radius * 0.95, cx, cy, radius * 1.08)
        .unwrap();
    atmo.add_color_stop(0.0, "rgba(255, 220, 180, 0)").unwrap();
    atmo.add_color_stop(0.6, "rgba(255, 220, 180, 0.1)")
        .unwrap();
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
    let gradient = ctx
        .create_radial_gradient(cx - radius * 0.3, cy - radius * 0.3, 0.0, cx, cy, radius)
        .unwrap();
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

    let bands = [
        "rgba(200, 180, 140, 0.3)",
        "rgba(180, 160, 120, 0.2)",
        "rgba(190, 170, 130, 0.25)",
    ];
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
    if radius < 15.0 {
        return;
    }

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
        if *opacity < 0.1 {
            continue;
        }

        let inner_r = radius * inner;
        let outer_r = radius * outer;

        // Draw arc (either top half or bottom half)
        ctx.set_global_alpha(*opacity * if behind { 0.5 } else { 1.0 });

        // Create gradient for ring
        let ring_grad = ctx.create_linear_gradient(-outer_r, 0.0, outer_r, 0.0);
        ring_grad
            .add_color_stop(0.0, &format!("{}60", color))
            .unwrap();
        ring_grad.add_color_stop(0.3, color).unwrap();
        ring_grad
            .add_color_stop(0.5, &lighten_color(color, 0.2))
            .unwrap();
        ring_grad.add_color_stop(0.7, color).unwrap();
        ring_grad
            .add_color_stop(1.0, &format!("{}60", color))
            .unwrap();

        ctx.set_fill_style(&ring_grad);
        ctx.begin_path();

        if behind {
            // Draw top arc (behind planet)
            ctx.ellipse(0.0, 0.0, outer_r, outer_r * tilt, 0.0, PI, 2.0 * PI)
                .unwrap_or(());
            ctx.ellipse(0.0, 0.0, inner_r, inner_r * tilt, 0.0, 2.0 * PI, PI)
                .unwrap_or(());
        } else {
            // Draw bottom arc (in front of planet)
            ctx.ellipse(0.0, 0.0, outer_r, outer_r * tilt, 0.0, 0.0, PI)
                .unwrap_or(());
            ctx.ellipse(0.0, 0.0, inner_r, inner_r * tilt, 0.0, PI, 0.0)
                .unwrap_or(());
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
    let gradient = ctx
        .create_radial_gradient(cx - radius * 0.3, cy - radius * 0.3, 0.0, cx, cy, radius)
        .unwrap();
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
        ctx.ellipse(
            dark_x,
            cy + radius * 0.1,
            radius * 0.25,
            radius * 0.4,
            0.3,
            0.0,
            2.0 * PI,
        )
        .unwrap_or(());
        ctx.fill();
    }

    // Polar ice caps (white)
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 250, 245, 0.9)"));
    ctx.begin_path();
    ctx.arc(cx, cy - radius * 0.85, radius * 0.2, 0.0, 2.0 * PI)
        .unwrap_or(());
    ctx.fill();

    // Southern cap (smaller)
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 250, 245, 0.7)"));
    ctx.begin_path();
    ctx.arc(cx, cy + radius * 0.9, radius * 0.12, 0.0, 2.0 * PI)
        .unwrap_or(());
    ctx.fill();

    ctx.restore();

    // Thin atmosphere
    let atmo = ctx
        .create_radial_gradient(cx, cy, radius * 0.95, cx, cy, radius * 1.05)
        .unwrap();
    atmo.add_color_stop(0.0, "rgba(255, 200, 180, 0)").unwrap();
    atmo.add_color_stop(0.7, "rgba(255, 180, 150, 0.08)")
        .unwrap();
    atmo.add_color_stop(1.0, "rgba(255, 150, 120, 0)").unwrap();
    ctx.set_fill_style(&atmo);
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 1.05, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Mercury - heavily cratered gray surface like our Moon
fn draw_mercury(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Base gray surface with 3D sphere shading
    let gradient = ctx
        .create_radial_gradient(cx - radius * 0.3, cy - radius * 0.3, 0.0, cx, cy, radius)
        .unwrap();
    gradient.add_color_stop(0.0, "#E0E0E0").unwrap(); // Light gray highlight
    gradient.add_color_stop(0.4, "#A0A0A0").unwrap(); // Medium gray
    gradient.add_color_stop(0.8, "#606060").unwrap(); // Dark gray terminator
    gradient.add_color_stop(1.0, "#303030").unwrap(); // Very dark limb

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Clipping for surface features
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    // Craters - Mercury is heavily cratered
    let rotation = time * 0.01; // Very slow rotation (59 Earth days)

    // Large impact basins (like Caloris)
    ctx.set_fill_style(&JsValue::from_str("rgba(60, 60, 60, 0.4)"));
    let basin_x = cx + (rotation + 0.5).cos() * radius * 0.3;
    if (rotation + 0.5).cos() > 0.0 {
        ctx.begin_path();
        ctx.arc(basin_x, cy - radius * 0.2, radius * 0.25, 0.0, 2.0 * PI)
            .unwrap_or(());
        ctx.fill();
        // Bright ray pattern
        ctx.set_fill_style(&JsValue::from_str("rgba(180, 180, 180, 0.2)"));
        for i in 0..6 {
            let angle = i as f64 * PI / 3.0;
            let ray_x = basin_x + angle.cos() * radius * 0.35;
            let ray_y = cy - radius * 0.2 + angle.sin() * radius * 0.35;
            ctx.begin_path();
            ctx.ellipse(
                ray_x,
                ray_y,
                radius * 0.08,
                radius * 0.02,
                angle,
                0.0,
                2.0 * PI,
            )
            .unwrap_or(());
            ctx.fill();
        }
    }

    // Smaller craters
    ctx.set_fill_style(&JsValue::from_str("rgba(50, 50, 50, 0.35)"));
    for i in 0..12 {
        let seed = i as f64 * 2.7 + 1.5;
        let lon = rotation + seed;
        if lon.cos() > -0.2 {
            let crater_x = cx + lon.cos() * radius * (0.3 + (seed * 0.7).sin().abs() * 0.5);
            let crater_y = cy + (seed * 2.1).sin() * radius * 0.7;
            let crater_r = radius * (0.03 + (seed * 0.5).sin().abs() * 0.06);
            ctx.begin_path();
            ctx.arc(crater_x, crater_y, crater_r, 0.0, 2.0 * PI)
                .unwrap_or(());
            ctx.fill();
        }
    }

    ctx.restore();
}

/// Venus - thick yellowish atmosphere with swirling clouds
fn draw_venus(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Base yellowish-white atmosphere
    let gradient = ctx
        .create_radial_gradient(cx - radius * 0.25, cy - radius * 0.25, 0.0, cx, cy, radius)
        .unwrap();
    gradient.add_color_stop(0.0, "#FFFDE8").unwrap(); // Bright cream
    gradient.add_color_stop(0.3, "#F5E6C8").unwrap(); // Light tan
    gradient.add_color_stop(0.6, "#E6C87A").unwrap(); // Yellow-tan
    gradient.add_color_stop(1.0, "#8B7355").unwrap(); // Darker limb

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Clipping for cloud features
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    // Venus retrograde rotation - very slow (243 Earth days, backwards)
    let rot = -time * 0.003;

    // Cloud bands - horizontal streaks in sulfuric acid atmosphere
    for i in 0..6 {
        let band_y = cy - radius * 0.8 + (i as f64 * radius * 0.32);
        let wave = (rot * 2.0 + i as f64 * 1.2).sin() * radius * 0.08;
        let alpha = 0.15 + (i as f64 * 0.5).sin().abs() * 0.1;

        ctx.set_fill_style(&JsValue::from_str(&format!(
            "rgba(200, 180, 140, {})",
            alpha
        )));
        ctx.fill_rect(cx - radius + wave, band_y, radius * 2.0, radius * 0.2);
    }

    // Y-shaped cloud pattern (characteristic of Venus)
    ctx.set_stroke_style(&JsValue::from_str("rgba(180, 160, 120, 0.2)"));
    ctx.set_line_width(radius * 0.1);
    let y_rot = rot * 0.5;
    ctx.begin_path();
    ctx.move_to(cx + y_rot.cos() * radius * 0.5, cy - radius * 0.6);
    ctx.quadratic_curve_to(cx, cy, cx - y_rot.sin() * radius * 0.4, cy + radius * 0.6);
    ctx.stroke();

    ctx.restore();

    // Thick hazy atmosphere glow
    let atmo = ctx
        .create_radial_gradient(cx, cy, radius * 0.9, cx, cy, radius * 1.15)
        .unwrap();
    atmo.add_color_stop(0.0, "rgba(255, 240, 200, 0)").unwrap();
    atmo.add_color_stop(0.5, "rgba(255, 240, 200, 0.15)")
        .unwrap();
    atmo.add_color_stop(0.8, "rgba(255, 220, 180, 0.1)")
        .unwrap();
    atmo.add_color_stop(1.0, "rgba(255, 200, 150, 0)").unwrap();
    ctx.set_fill_style(&atmo);
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 1.15, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Uranus - pale cyan ice giant with extreme axial tilt and faint rings
fn draw_uranus(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Uranus rings (behind planet - very faint, nearly vertical)
    if radius > 8.0 {
        draw_uranus_rings(ctx, cx, cy, radius, true);
    }

    // Base pale cyan-green color (methane atmosphere)
    let gradient = ctx
        .create_radial_gradient(cx - radius * 0.3, cy - radius * 0.3, 0.0, cx, cy, radius)
        .unwrap();
    gradient.add_color_stop(0.0, "#E8FFFF").unwrap(); // Bright cyan-white
    gradient.add_color_stop(0.3, "#C5E8E8").unwrap(); // Light cyan
    gradient.add_color_stop(0.6, "#80C8C8").unwrap(); // Medium cyan-green
    gradient.add_color_stop(1.0, "#4A8080").unwrap(); // Dark teal limb

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Clipping for surface features
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    // Very subtle banding (Uranus appears nearly featureless)
    let rot = time * 0.008;
    ctx.set_fill_style(&JsValue::from_str("rgba(150, 200, 200, 0.08)"));
    for i in 0..3 {
        let y = cy - radius * 0.4 + (i as f64 * radius * 0.35);
        let wave = (rot + i as f64).sin() * radius * 0.03;
        ctx.fill_rect(cx - radius + wave, y, radius * 2.0, radius * 0.2);
    }

    // Polar region (slightly brighter, as Uranus is tilted)
    ctx.set_fill_style(&JsValue::from_str("rgba(200, 230, 230, 0.15)"));
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.3, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    ctx.restore();

    // Uranus rings (in front of planet)
    if radius > 8.0 {
        draw_uranus_rings(ctx, cx, cy, radius, false);
    }

    // Faint methane atmosphere haze
    let atmo = ctx
        .create_radial_gradient(cx, cy, radius * 0.95, cx, cy, radius * 1.08)
        .unwrap();
    atmo.add_color_stop(0.0, "rgba(200, 255, 255, 0)").unwrap();
    atmo.add_color_stop(0.6, "rgba(180, 230, 230, 0.08)")
        .unwrap();
    atmo.add_color_stop(1.0, "rgba(150, 200, 200, 0)").unwrap();
    ctx.set_fill_style(&atmo);
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 1.08, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

/// Draw Uranus's faint ring system (nearly vertical due to 98° axial tilt)
fn draw_uranus_rings(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, behind: bool) {
    ctx.save();
    ctx.translate(cx, cy).unwrap_or(());

    // Uranus's extreme tilt - rings appear nearly vertical
    let tilt = 0.15; // Very edge-on

    // Ring definitions (inner_mult, outer_mult, opacity)
    let rings = [
        (1.6, 1.65, 0.15), // Zeta ring
        (1.7, 1.75, 0.2),  // 6, 5, 4 rings
        (1.8, 1.85, 0.25), // Alpha, Beta rings
        (1.9, 1.95, 0.3),  // Eta, Gamma, Delta rings
        (2.0, 2.05, 0.35), // Epsilon ring (brightest)
    ];

    for (inner, outer, opacity) in rings.iter() {
        let inner_r = radius * inner;
        let outer_r = radius * outer;

        ctx.set_global_alpha(*opacity * if behind { 0.4 } else { 0.8 });
        ctx.set_fill_style(&JsValue::from_str("#9CBCBC"));

        ctx.begin_path();
        if behind {
            ctx.ellipse(0.0, 0.0, outer_r, outer_r * tilt, 0.0, PI, 2.0 * PI)
                .unwrap_or(());
            ctx.ellipse(0.0, 0.0, inner_r, inner_r * tilt, 0.0, 2.0 * PI, PI)
                .unwrap_or(());
        } else {
            ctx.ellipse(0.0, 0.0, outer_r, outer_r * tilt, 0.0, 0.0, PI)
                .unwrap_or(());
            ctx.ellipse(0.0, 0.0, inner_r, inner_r * tilt, 0.0, PI, 0.0)
                .unwrap_or(());
        }
        ctx.close_path();
        ctx.fill();
    }

    ctx.set_global_alpha(1.0);
    ctx.restore();
}

/// Neptune - vivid blue ice giant with Great Dark Spot and fast winds
fn draw_neptune(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, radius: f64, time: f64) {
    // Base vivid blue (methane atmosphere absorbs red)
    let gradient = ctx
        .create_radial_gradient(cx - radius * 0.3, cy - radius * 0.3, 0.0, cx, cy, radius)
        .unwrap();
    gradient.add_color_stop(0.0, "#B0D4FF").unwrap(); // Light blue highlight
    gradient.add_color_stop(0.3, "#6B9FDE").unwrap(); // Medium blue
    gradient.add_color_stop(0.6, "#3A6098").unwrap(); // Deeper blue
    gradient.add_color_stop(1.0, "#1A3050").unwrap(); // Very dark blue limb

    ctx.set_fill_style(&gradient);
    ctx.begin_path();
    ctx.arc(cx, cy, radius, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();

    // Clipping for surface features
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 0.98, 0.0, 2.0 * PI).unwrap_or(());
    ctx.clip();

    // Neptune has fastest winds in solar system - streaky clouds
    let rot = time * 0.012; // Fast rotation (16 hours)

    // Cloud bands with high-speed winds
    for i in 0..5 {
        let band_y = cy - radius * 0.8 + (i as f64 * radius * 0.35);
        let wind_speed = if i == 2 { 0.5 } else { 1.0 + i as f64 * 0.3 }; // Varies by latitude
        let wave = (rot * wind_speed + i as f64 * 0.8).sin() * radius * 0.15;
        let alpha = 0.08 + (i as f64 * 0.3).sin().abs() * 0.08;

        ctx.set_fill_style(&JsValue::from_str(&format!(
            "rgba(60, 100, 160, {})",
            alpha
        )));
        ctx.fill_rect(cx - radius + wave, band_y, radius * 2.0, radius * 0.15);
    }

    // Great Dark Spot (discovered by Voyager 2, changes over time)
    let spot_rot = rot * 0.8;
    if spot_rot.cos() > -0.3 {
        let spot_x = cx + spot_rot.cos() * radius * 0.35;
        let spot_y = cy - radius * 0.2;

        // Dark oval storm
        let spot_grad = ctx
            .create_radial_gradient(spot_x, spot_y, 0.0, spot_x, spot_y, radius * 0.2)
            .unwrap();
        spot_grad
            .add_color_stop(0.0, "rgba(20, 40, 80, 0.6)")
            .unwrap();
        spot_grad
            .add_color_stop(0.7, "rgba(30, 60, 100, 0.3)")
            .unwrap();
        spot_grad
            .add_color_stop(1.0, "rgba(40, 80, 140, 0)")
            .unwrap();

        ctx.set_fill_style(&spot_grad);
        ctx.begin_path();
        ctx.ellipse(
            spot_x,
            spot_y,
            radius * 0.18,
            radius * 0.1,
            0.1,
            0.0,
            2.0 * PI,
        )
        .unwrap_or(());
        ctx.fill();

        // Small companion (Scooter - fast-moving white cloud)
        if (spot_rot + 0.5).cos() > 0.0 {
            ctx.set_fill_style(&JsValue::from_str("rgba(200, 220, 255, 0.4)"));
            ctx.begin_path();
            let scooter_x = cx + (spot_rot + 0.5).cos() * radius * 0.4;
            ctx.arc(scooter_x, cy + radius * 0.1, radius * 0.06, 0.0, 2.0 * PI)
                .unwrap_or(());
            ctx.fill();
        }
    }

    // High-altitude white cirrus clouds
    ctx.set_fill_style(&JsValue::from_str("rgba(200, 220, 255, 0.25)"));
    for i in 0..4 {
        let seed = i as f64 * 1.9;
        let cloud_rot = rot * 1.5 + seed;
        if cloud_rot.cos() > 0.2 {
            let cloud_x = cx + cloud_rot.cos() * radius * 0.5;
            let cloud_y = cy + (seed * 1.7).sin() * radius * 0.5;
            ctx.begin_path();
            ctx.ellipse(
                cloud_x,
                cloud_y,
                radius * 0.08,
                radius * 0.03,
                cloud_rot * 0.3,
                0.0,
                2.0 * PI,
            )
            .unwrap_or(());
            ctx.fill();
        }
    }

    ctx.restore();

    // Blue atmosphere glow
    let atmo = ctx
        .create_radial_gradient(cx, cy, radius * 0.95, cx, cy, radius * 1.12)
        .unwrap();
    atmo.add_color_stop(0.0, "rgba(100, 150, 255, 0)").unwrap();
    atmo.add_color_stop(0.5, "rgba(100, 150, 255, 0.12)")
        .unwrap();
    atmo.add_color_stop(0.8, "rgba(80, 120, 200, 0.06)")
        .unwrap();
    atmo.add_color_stop(1.0, "rgba(60, 100, 180, 0)").unwrap();
    ctx.set_fill_style(&atmo);
    ctx.begin_path();
    ctx.arc(cx, cy, radius * 1.12, 0.0, 2.0 * PI).unwrap_or(());
    ctx.fill();
}

// ============================================================================
// MISSIONS
// ============================================================================

fn draw_missions(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    let view = &state.view;

    for m in 0..state.mission_count {
        if !state.mission_active[m] {
            continue;
        }

        let x = state.mission_x[m];
        let y = state.mission_y[m];

        // Visibility check
        if !view.is_visible(x, y, 5.0) {
            continue;
        }

        let (sx, sy, _) = state.project_3d(x, y, 0.0);
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
        ctx.set_font("500 10px 'Just Sans', monospace");
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.fill_text(name, sx + 12.0, sy - 5.0).unwrap_or(());

        // Distance from sun
        let dist = (x * x + y * y).sqrt();
        ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.5)"));
        ctx.fill_text(&format!("{:.1} AU", dist), sx + 12.0, sy + 8.0)
            .unwrap_or(());
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
    if pulse < 0.1 {
        return;
    }

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
    if count < 2 {
        return;
    }

    let wps = &state.mission_waypoints[idx];
    let color = state.mission_colors[idx];

    ctx.set_stroke_style(&JsValue::from_str(&format!("{}60", color)));
    ctx.set_line_width(1.0);
    ctx.begin_path();

    let (sx, sy, _) = state.project_3d(wps[0].1, wps[0].2, 0.0);
    ctx.move_to(sx, sy);

    for i in 1..count {
        let (sx, sy, _) = state.project_3d(wps[i].1, wps[i].2, 0.0);
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
    ctx.set_font("700 16px 'Just Sans', monospace");
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.9)"));
    ctx.fill_text(&format!("{:04}-{:02}-{:02}", year, month, day), 20.0, 30.0)
        .unwrap_or(());

    // Time scale indicator
    ctx.set_font("500 12px 'Just Sans', monospace");
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

    // CCA frame indicator (below zoom level)
    let epoch = Epoch::from_jd(state.julian_date, TimeScale::TDB);
    let frame = FrameId::HCI; // Heliocentric Inertial
    ctx.set_font("500 10px 'Just Sans', monospace");
    ctx.set_fill_style(&JsValue::from_str("rgba(100, 200, 255, 0.7)"));
    ctx.fill_text(
        &format!("CCA Frame: {:?} | JD: {:.2}", frame, epoch.jd()),
        w - 20.0,
        50.0,
    )
    .unwrap_or(());
    ctx.set_text_align("start");

    // FPS (bottom-left, only if debugging)
    #[cfg(debug_assertions)]
    {
        ctx.fill_text(&format!("FPS: {:.0}", state.fps), 20.0, h - 20.0)
            .unwrap_or(());
    }

    // Controls hint (bottom)
    ctx.set_font("500 11px 'Just Sans', sans-serif");
    ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.4)"));
    ctx.set_text_align("center");
    ctx.fill_text(
        "Scroll: zoom | Drag: pan | 1-8: planets | Space: pause | +/-: time scale",
        w / 2.0,
        h - 15.0,
    )
    .unwrap_or(());
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
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

// ============================================================================
// 3D STAR RENDERING (with parallax)
// ============================================================================

/// Draw stars from the StarDatabase at their true 3D positions
/// NOTE: Deprecated - now integrated into draw_bright_stars
#[allow(dead_code)]
fn draw_stars_3d(ctx: &CanvasRenderingContext2d, state: &SimulationState, time: f64) {
    use crate::cca_projection::ScaleLevel;

    let width = state.view.width;
    let height = state.view.height;

    // Get current scale level from camera
    let scale_level = state.camera.scale_level;

    // Only draw 3D stars at heliosphere scale and beyond
    // At smaller scales, the celestial sphere projection is sufficient
    let (enabled, mag_limit, show_labels) = match scale_level {
        ScaleLevel::Planet | ScaleLevel::Inner | ScaleLevel::Outer => return,
        ScaleLevel::Heliosphere => (true, 2.0, true), // Brightest stars only
        ScaleLevel::NearStars => (true, 5.0, true),   // Naked eye stars
        ScaleLevel::FarStars => (true, 7.0, false),   // Many stars, no labels
    };

    if !enabled {
        return;
    }

    // Calculate star size scaling based on zoom
    // At stellar scales, we want stars to be visible but not huge
    let size_scale = match scale_level {
        ScaleLevel::Heliosphere => 1.5,
        ScaleLevel::NearStars => 1.2,
        ScaleLevel::FarStars => 1.0,
        _ => 1.0,
    };

    // Draw stars from the database, filtered by magnitude
    for star in state.stars.brighter_than(mag_limit) {
        // Project star's 3D position to screen
        let (sx, sy, depth) = state.project_3d(star.position.x, star.position.y, star.position.z);

        // Skip if off screen (with large margin for star names)
        if sx < -100.0 || sx > width + 100.0 || sy < -100.0 || sy > height + 100.0 {
            continue;
        }

        // Skip if behind camera
        if depth < 0.0 {
            continue;
        }

        // Get star color from B-V index
        let (r, g, b) = star.color_rgb();
        let color = format!("rgb({}, {}, {})", r, g, b);

        // Size based on magnitude (brighter = larger)
        let base_size = star.apparent_size() * size_scale;

        // Twinkle effect for brighter stars
        let twinkle = if star.magnitude < 1.0 {
            0.9 + ((time * 1.5 + star.hip_id as f64 * 0.1).sin() * 0.1)
        } else {
            1.0
        };

        let size = base_size * twinkle;

        // Draw star glow for bright stars
        if star.magnitude < 3.0 {
            ctx.set_global_alpha(0.3 * twinkle);
            if let Ok(glow) = ctx.create_radial_gradient(sx, sy, 0.0, sx, sy, size * 3.0) {
                let _ = glow.add_color_stop(0.0, &color);
                let _ = glow.add_color_stop(1.0, "transparent");
                ctx.set_fill_style(&glow);
                ctx.begin_path();
                let _ = ctx.arc(sx, sy, size * 3.0, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
            }
        }

        // Draw star core
        ctx.set_global_alpha(twinkle);
        ctx.set_fill_style(&JsValue::from_str(&color));
        ctx.begin_path();
        let _ = ctx.arc(sx, sy, size, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();

        // Draw star name for bright stars (if labels enabled at this scale)
        if show_labels && star.magnitude < 2.0 && !star.name.is_empty() {
            ctx.set_global_alpha(0.8);
            ctx.set_fill_style(&JsValue::from_str("#aaa"));
            ctx.set_font("11px sans-serif");
            ctx.set_text_align("left");
            let _ = ctx.fill_text(&star.name, sx + size + 4.0, sy + 3.0);
        }
    }

    ctx.set_global_alpha(1.0);
}
