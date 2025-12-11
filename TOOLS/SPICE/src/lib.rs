//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | TOOLS/SPICE/src/lib.rs
//! PURPOSE: SPICE circuit simulator WASM application
//! MODIFIED: 2025-12-09
//! LAYER: TOOLS → SPICE
//! ═══════════════════════════════════════════════════════════════════════════════

#![allow(unexpected_cfgs)]
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, Document, Element, HtmlCanvasElement, HtmlElement, HtmlInputElement,
    HtmlTextAreaElement,
};

use spice_engine::{
    ac_analysis, find_cutoff_frequency, generate_bode_plot, BodePoint, Element as CircuitElement,
    Netlist, SourceValue,
};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"SPICE Simulator initialized".into());

    if let Err(e) = init_ui() {
        web_sys::console::error_1(&format!("Failed to initialize UI: {:?}", e).into());
    }

    Ok(())
}

fn init_ui() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Set up analyze button
    if let Some(btn) = document.get_element_by_id("analyze-btn") {
        let btn: HtmlElement = btn.dyn_into()?;
        let closure = Closure::wrap(Box::new(move || {
            if let Err(e) = run_analysis() {
                web_sys::console::error_1(&format!("Analysis failed: {:?}", e).into());
            }
        }) as Box<dyn FnMut()>);
        btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Set default netlist
    if let Some(textarea) = document.get_element_by_id("netlist") {
        let textarea: HtmlTextAreaElement = textarea.dyn_into()?;
        textarea.set_value(
            "* RC Low-Pass Filter\n\
             V1 in 0 AC 1\n\
             R1 in out 1k\n\
             C1 out 0 1u",
        );
    }

    // Run initial analysis
    run_analysis()?;

    Ok(())
}

fn run_analysis() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Read netlist
    let netlist_text = get_textarea_value(&document, "netlist")?;
    let freq_start = get_input_value(&document, "freq-start")?;
    let freq_stop = get_input_value(&document, "freq-stop")?;
    let output_node = get_input_value(&document, "output-node")? as usize;

    // Parse netlist
    let netlist = match parse_netlist(&netlist_text) {
        Ok(n) => n,
        Err(e) => {
            show_error(&document, &e)?;
            return Err(JsValue::from_str(&e));
        }
    };

    // Run AC analysis
    match ac_analysis(&netlist, freq_start, freq_stop, 50) {
        Ok(ac_result) => {
            hide_error(&document)?;

            // Generate Bode plot
            let bode = generate_bode_plot(&ac_result, output_node);

            // Display results
            display_results(&document, &bode)?;

            // Draw plots
            draw_magnitude_plot(&document, &bode)?;
            draw_phase_plot(&document, &bode)?;

            Ok(())
        }
        Err(e) => {
            show_error(&document, &e)?;
            Err(JsValue::from_str(&e))
        }
    }
}

fn parse_netlist(text: &str) -> Result<Netlist, String> {
    let mut netlist = Netlist::new("User Circuit".to_string());

    for line in text.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('*') || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].to_uppercase();
        let first_char = name.chars().next().unwrap_or(' ');

        match first_char {
            'R' => {
                // Resistor: R1 node+ node- value
                if parts.len() < 4 {
                    return Err(format!("Invalid resistor: {}", line));
                }
                let value = parse_value(parts[3])?;
                netlist.add_element(CircuitElement::Resistor {
                    name: name.clone(),
                    node_p: parts[1].to_string(),
                    node_n: parts[2].to_string(),
                    value,
                });
            }
            'C' => {
                // Capacitor: C1 node+ node- value
                if parts.len() < 4 {
                    return Err(format!("Invalid capacitor: {}", line));
                }
                let value = parse_value(parts[3])?;
                netlist.add_element(CircuitElement::Capacitor {
                    name: name.clone(),
                    node_p: parts[1].to_string(),
                    node_n: parts[2].to_string(),
                    value,
                });
            }
            'L' => {
                // Inductor: L1 node+ node- value
                if parts.len() < 4 {
                    return Err(format!("Invalid inductor: {}", line));
                }
                let value = parse_value(parts[3])?;
                netlist.add_element(CircuitElement::Inductor {
                    name: name.clone(),
                    node_p: parts[1].to_string(),
                    node_n: parts[2].to_string(),
                    value,
                });
            }
            'V' => {
                // Voltage source: V1 node+ node- [DC value] [AC mag [phase]]
                if parts.len() < 4 {
                    return Err(format!("Invalid voltage source: {}", line));
                }

                let source_value = parse_source_value(&parts[3..])?;
                netlist.add_element(CircuitElement::VoltageSource {
                    name: name.clone(),
                    node_p: parts[1].to_string(),
                    node_n: parts[2].to_string(),
                    value: source_value,
                });
            }
            'I' => {
                // Current source: I1 node+ node- value
                if parts.len() < 4 {
                    return Err(format!("Invalid current source: {}", line));
                }

                let value = parse_value(parts[3])?;
                netlist.add_element(CircuitElement::CurrentSource {
                    name: name.clone(),
                    node_p: parts[1].to_string(),
                    node_n: parts[2].to_string(),
                    value,
                });
            }
            _ => {
                // Ignore unknown elements
                web_sys::console::warn_1(&format!("Unknown element: {}", line).into());
            }
        }
    }

    Ok(netlist)
}

fn parse_value(s: &str) -> Result<f64, String> {
    let s = s.trim().to_uppercase();

    // Handle SI prefixes
    let (num_part, multiplier) = if s.ends_with('K') {
        (&s[..s.len() - 1], 1e3)
    } else if s.ends_with('M') {
        (&s[..s.len() - 1], 1e6)
    } else if s.ends_with('G') {
        (&s[..s.len() - 1], 1e9)
    } else if s.ends_with('U') {
        (&s[..s.len() - 1], 1e-6)
    } else if s.ends_with('N') {
        (&s[..s.len() - 1], 1e-9)
    } else if s.ends_with('P') {
        (&s[..s.len() - 1], 1e-12)
    } else if s.ends_with('F') {
        (&s[..s.len() - 1], 1e-15)
    } else if s.contains("MEG") {
        (&s[..s.find("MEG").unwrap()], 1e6)
    } else {
        (s.as_str(), 1.0)
    };

    num_part
        .parse::<f64>()
        .map(|v| v * multiplier)
        .map_err(|_| format!("Invalid value: {}", s))
}

fn parse_source_value(parts: &[&str]) -> Result<SourceValue, String> {
    if parts.is_empty() {
        return Ok(SourceValue::DC(0.0));
    }

    let first = parts[0].to_uppercase();

    if first == "AC" {
        let magnitude = if parts.len() > 1 {
            parse_value(parts[1])?
        } else {
            1.0
        };
        let phase = if parts.len() > 2 {
            parse_value(parts[2])?
        } else {
            0.0
        };
        Ok(SourceValue::AC { magnitude, phase })
    } else if first == "DC" {
        let value = if parts.len() > 1 {
            parse_value(parts[1])?
        } else {
            0.0
        };
        Ok(SourceValue::DC(value))
    } else {
        // Assume DC value
        let value = parse_value(&first)?;
        Ok(SourceValue::DC(value))
    }
}

fn display_results(document: &Document, bode: &[BodePoint]) -> Result<(), JsValue> {
    if bode.is_empty() {
        return Ok(());
    }

    // DC gain (first point)
    let dc_gain = bode[0].magnitude_db;
    set_text(document, "result-dc-gain", &format!("{:.1} dB", dc_gain))?;

    // Cutoff frequency
    if let Some(fc) = find_cutoff_frequency(bode) {
        let fc_text = if fc >= 1e6 {
            format!("{:.2} MHz", fc / 1e6)
        } else if fc >= 1e3 {
            format!("{:.2} kHz", fc / 1e3)
        } else {
            format!("{:.2} Hz", fc)
        };
        set_text(document, "result-cutoff", &fc_text)?;

        // Find phase at cutoff
        let phase_at_cutoff = bode
            .iter()
            .find(|p| p.frequency >= fc)
            .map(|p| p.phase_deg)
            .unwrap_or(0.0);
        set_text(
            document,
            "result-phase",
            &format!("{:.1}°", phase_at_cutoff),
        )?;
    } else {
        set_text(document, "result-cutoff", "N/A")?;
        set_text(document, "result-phase", "N/A")?;
    }

    // High frequency rolloff (compare last two decades)
    if bode.len() >= 10 {
        let last = &bode[bode.len() - 1];
        let mid = &bode[bode.len() / 2];
        let decades = (last.frequency / mid.frequency).log10();
        if decades > 0.1 {
            let rolloff = (last.magnitude_db - mid.magnitude_db) / decades;
            set_text(
                document,
                "result-rolloff",
                &format!("{:.0} dB/dec", rolloff),
            )?;
        }
    }

    Ok(())
}

fn draw_magnitude_plot(document: &Document, bode: &[BodePoint]) -> Result<(), JsValue> {
    let canvas = document
        .get_element_by_id("magnitude-canvas")
        .ok_or("Canvas not found")?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    let window = web_sys::window().ok_or("No window")?;
    let dpr = window.device_pixel_ratio();

    let canvas_element: Element = canvas.clone().into();
    let rect = canvas_element.get_bounding_client_rect();
    let css_width = rect.width();
    let css_height = rect.height();

    let target_width = (css_width * dpr) as u32;
    let target_height = (css_height * dpr) as u32;

    if (canvas.width() as i32 - target_width as i32).abs() > 2
        || (canvas.height() as i32 - target_height as i32).abs() > 2
    {
        canvas.set_width(target_width);
        canvas.set_height(target_height);
    }

    let ctx = canvas
        .get_context("2d")?
        .ok_or("No 2D context")?
        .dyn_into::<CanvasRenderingContext2d>()?;

    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    ctx.scale(dpr, dpr)?;

    draw_bode_curve(&ctx, bode, css_width, css_height, true)?;

    Ok(())
}

fn draw_phase_plot(document: &Document, bode: &[BodePoint]) -> Result<(), JsValue> {
    let canvas = document
        .get_element_by_id("phase-canvas")
        .ok_or("Canvas not found")?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    let window = web_sys::window().ok_or("No window")?;
    let dpr = window.device_pixel_ratio();

    let canvas_element: Element = canvas.clone().into();
    let rect = canvas_element.get_bounding_client_rect();
    let css_width = rect.width();
    let css_height = rect.height();

    let target_width = (css_width * dpr) as u32;
    let target_height = (css_height * dpr) as u32;

    if (canvas.width() as i32 - target_width as i32).abs() > 2
        || (canvas.height() as i32 - target_height as i32).abs() > 2
    {
        canvas.set_width(target_width);
        canvas.set_height(target_height);
    }

    let ctx = canvas
        .get_context("2d")?
        .ok_or("No 2D context")?
        .dyn_into::<CanvasRenderingContext2d>()?;

    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    ctx.scale(dpr, dpr)?;

    draw_bode_curve(&ctx, bode, css_width, css_height, false)?;

    Ok(())
}

fn draw_bode_curve(
    ctx: &CanvasRenderingContext2d,
    bode: &[BodePoint],
    width: f64,
    height: f64,
    is_magnitude: bool,
) -> Result<(), JsValue> {
    // Clear canvas
    ctx.set_fill_style(&JsValue::from_str("#0a0a12"));
    ctx.fill_rect(0.0, 0.0, width, height);

    if bode.is_empty() {
        return Ok(());
    }

    let margin_left = 60.0;
    let margin_right = 20.0;
    let margin_top = 20.0;
    let margin_bottom = 40.0;
    let plot_width = width - margin_left - margin_right;
    let plot_height = height - margin_top - margin_bottom;

    // Find data range
    let freq_min = bode[0].frequency.log10();
    let freq_max = bode.last().unwrap().frequency.log10();

    let (y_values, y_label, color): (Vec<f64>, &str, &str) = if is_magnitude {
        (
            bode.iter().map(|p| p.magnitude_db).collect(),
            "Magnitude (dB)",
            "#00aaff",
        )
    } else {
        (
            bode.iter().map(|p| p.phase_deg).collect(),
            "Phase (degrees)",
            "#ffaa00",
        )
    };

    let y_min = y_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_range = y_max - y_min;
    let y_padding = y_range * 0.1;

    let x_to_pixel = |freq: f64| -> f64 {
        let log_freq = freq.log10();
        margin_left + ((log_freq - freq_min) / (freq_max - freq_min)) * plot_width
    };

    let y_to_pixel = |val: f64| -> f64 {
        margin_top + (1.0 - (val - (y_min - y_padding)) / (y_range + 2.0 * y_padding)) * plot_height
    };

    // Draw grid
    ctx.set_stroke_style(&JsValue::from_str("rgba(0, 170, 255, 0.1)"));
    ctx.set_line_width(1.0);
    ctx.begin_path();

    // X-axis grid (logarithmic)
    for i in 0..=5 {
        let log_freq = freq_min + (freq_max - freq_min) * (i as f64 / 5.0);
        let freq = 10f64.powf(log_freq);
        let x = x_to_pixel(freq);
        ctx.move_to(x, margin_top);
        ctx.line_to(x, height - margin_bottom);

        ctx.set_fill_style(&JsValue::from_str("#808080"));
        ctx.set_font("10px Monaco");
        ctx.set_text_align("center");
        let label = if freq >= 1e6 {
            format!("{:.0}M", freq / 1e6)
        } else if freq >= 1e3 {
            format!("{:.0}k", freq / 1e3)
        } else {
            format!("{:.0}", freq)
        };
        ctx.fill_text(&label, x, height - margin_bottom + 15.0)?;
    }

    // Y-axis grid
    for i in 0..=5 {
        let val = (y_min - y_padding) + (y_range + 2.0 * y_padding) * (i as f64 / 5.0);
        let y = y_to_pixel(val);
        ctx.move_to(margin_left, y);
        ctx.line_to(width - margin_right, y);

        ctx.set_fill_style(&JsValue::from_str("#808080"));
        ctx.set_text_align("right");
        ctx.fill_text(&format!("{:.0}", val), margin_left - 10.0, y + 3.0)?;
    }
    ctx.stroke();

    // Draw curve
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(2.0);
    ctx.begin_path();

    for (i, point) in bode.iter().enumerate() {
        let x = x_to_pixel(point.frequency);
        let y = y_to_pixel(y_values[i]);

        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke();

    // Labels
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.set_font("12px Monaco");
    ctx.set_text_align("left");
    ctx.fill_text(y_label, margin_left, margin_top - 5.0)?;
    ctx.set_text_align("center");
    ctx.fill_text("Frequency (Hz)", width / 2.0, height - 5.0)?;

    Ok(())
}

fn get_textarea_value(document: &Document, id: &str) -> Result<String, JsValue> {
    let elem = document
        .get_element_by_id(id)
        .ok_or(format!("Textarea {} not found", id))?;
    let textarea: HtmlTextAreaElement = elem.dyn_into()?;
    Ok(textarea.value())
}

fn get_input_value(document: &Document, id: &str) -> Result<f64, JsValue> {
    let input = document
        .get_element_by_id(id)
        .ok_or(format!("Input {} not found", id))?;
    let input: HtmlInputElement = input.dyn_into()?;
    input
        .value()
        .parse::<f64>()
        .map_err(|_| JsValue::from_str("Invalid number"))
}

fn set_text(document: &Document, id: &str, text: &str) -> Result<(), JsValue> {
    if let Some(elem) = document.get_element_by_id(id) {
        let elem: HtmlElement = elem.dyn_into()?;
        elem.set_inner_text(text);
    }
    Ok(())
}

fn show_error(document: &Document, msg: &str) -> Result<(), JsValue> {
    if let Some(elem) = document.get_element_by_id("error-msg") {
        let elem: HtmlElement = elem.dyn_into()?;
        elem.set_inner_html(&format!("Error: {}", msg));
        elem.style().set_property("display", "block")?;
    }
    Ok(())
}

fn hide_error(document: &Document) -> Result<(), JsValue> {
    if let Some(elem) = document.get_element_by_id("error-msg") {
        let elem: HtmlElement = elem.dyn_into()?;
        elem.style().set_property("display", "none")?;
    }
    Ok(())
}
