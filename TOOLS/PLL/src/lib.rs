use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, Document, Element, HtmlCanvasElement, HtmlElement, HtmlInputElement,
};

use dna::pll::{design_pll, PLLArchitecture, PLLRequirements};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"PLL Designer initialized".into());

    // Initialize the UI
    if let Err(e) = init_ui() {
        web_sys::console::error_1(&format!("Failed to initialize UI: {:?}", e).into());
    }

    Ok(())
}

fn init_ui() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Set up event listeners for inputs
    setup_input_listeners(&document)?;

    // Set up design button
    if let Some(btn) = document.get_element_by_id("design-btn") {
        let btn: HtmlElement = btn.dyn_into()?;
        let closure = Closure::wrap(Box::new(move || {
            if let Err(e) = run_design() {
                web_sys::console::error_1(&format!("Design failed: {:?}", e).into());
            }
        }) as Box<dyn FnMut()>);
        btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Run initial design with default values
    run_design()?;

    Ok(())
}

fn setup_input_listeners(document: &Document) -> Result<(), JsValue> {
    const INPUT_IDS: &[&str] = &[
        "ref-freq",
        "output-freq-min",
        "output-freq-max",
        "loop-bandwidth",
        "phase-margin",
    ];

    for &id in INPUT_IDS {
        // Set up number input listener
        if let Some(input) = document.get_element_by_id(id) {
            let input: HtmlInputElement = input.dyn_into()?;
            let slider_id = format!("{}-slider", id);
            let doc_clone = document.clone();

            let closure = Closure::wrap(Box::new(move || {
                // Sync slider with input value
                if let Some(slider_elem) = doc_clone.get_element_by_id(&slider_id) {
                    if let Ok(slider) = slider_elem.dyn_into::<HtmlInputElement>() {
                        if let Ok(input_elem) = doc_clone.get_element_by_id(id).unwrap().dyn_into::<HtmlInputElement>() {
                            slider.set_value(&input_elem.value());
                        }
                    }
                }

                if let Err(e) = run_design() {
                    web_sys::console::error_1(&format!("Design update failed: {:?}", e).into());
                }
            }) as Box<dyn FnMut()>);
            input.set_oninput(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        // Set up slider listener
        let slider_id = format!("{}-slider", id);
        if let Some(slider) = document.get_element_by_id(&slider_id) {
            let slider: HtmlInputElement = slider.dyn_into()?;
            let input_id_clone = id.to_string();
            let doc_clone = document.clone();

            let closure = Closure::wrap(Box::new(move || {
                // Sync input with slider value
                if let Some(input_elem) = doc_clone.get_element_by_id(&input_id_clone) {
                    if let Ok(input) = input_elem.dyn_into::<HtmlInputElement>() {
                        if let Ok(slider_elem) = doc_clone.get_element_by_id(&format!("{}-slider", input_id_clone)).unwrap().dyn_into::<HtmlInputElement>() {
                            input.set_value(&slider_elem.value());
                        }
                    }
                }

                if let Err(e) = run_design() {
                    web_sys::console::error_1(&format!("Design update failed: {:?}", e).into());
                }
            }) as Box<dyn FnMut()>);
            slider.set_oninput(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }

    Ok(())
}

fn run_design() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Read input values
    let ref_freq = get_input_value(&document, "ref-freq")? * 1e6; // MHz to Hz
    let output_freq_min = get_input_value(&document, "output-freq-min")? * 1e6;
    let output_freq_max = get_input_value(&document, "output-freq-max")? * 1e6;
    let loop_bandwidth = get_input_value(&document, "loop-bandwidth")? * 1e3; // kHz to Hz
    let phase_margin = get_input_value(&document, "phase-margin")?;

    // Create requirements
    let requirements = PLLRequirements {
        ref_freq_hz: ref_freq,
        output_freq_min_hz: output_freq_min,
        output_freq_max_hz: output_freq_max,
        loop_bandwidth_hz: loop_bandwidth,
        phase_margin_deg: phase_margin,
        architecture: PLLArchitecture::IntegerN,
        supply_voltage: 3.3,
    };

    // Run design
    match design_pll(&requirements) {
        Ok(design) => {
            // Display results
            display_results(&document, &design)?;

            // Draw schematic
            draw_schematic(&document, &design)?;

            // Draw Bode plot
            draw_bode_plot(&document, &design.bode_plot)?;

            Ok(())
        }
        Err(e) => {
            // Display error
            if let Some(elem) = document.get_element_by_id("error-msg") {
                let elem: HtmlElement = elem.dyn_into()?;
                elem.set_inner_html(&format!("Design Error: {}", e));
                elem.style().set_property("display", "block")?;
            }
            Err(JsValue::from_str(&e))
        }
    }
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

fn display_results(
    document: &Document,
    design: &dna::pll::PLLDesign,
) -> Result<(), JsValue> {
    // Hide error message
    if let Some(elem) = document.get_element_by_id("error-msg") {
        let elem: HtmlElement = elem.dyn_into()?;
        elem.style().set_property("display", "none")?;
    }

    // Display dividers
    set_text(document, "result-r", &format!("R = {}", design.divider_r))?;

    let n_value = match &design.divider_n {
        dna::pll::DividerConfig::IntegerN { n, .. } => *n,
        dna::pll::DividerConfig::FractionalN { n_int, .. } => *n_int,
    };
    set_text(document, "result-n", &format!("N = {}", n_value))?;
    set_text(
        document,
        "result-pfd",
        &format!("{:.2} MHz", design.pfd_freq_hz / 1e6),
    )?;

    // Display loop filter components (find from components vector)
    let c1 = design.loop_filter.components.iter().find(|c| c.designator == "C1");
    let r1 = design.loop_filter.components.iter().find(|c| c.designator == "R1");
    let c2 = design.loop_filter.components.iter().find(|c| c.designator == "C2");

    if let Some(c) = c1 {
        set_text(document, "result-c1", &format!("{:.2} {}", c.actual_value, c.unit))?;
    }
    if let Some(r) = r1 {
        set_text(document, "result-r1", &format!("{:.0} {}", r.actual_value, r.unit))?;
    }
    if let Some(c) = c2 {
        set_text(document, "result-c2", &format!("{:.2} {}", c.actual_value, c.unit))?;
    }

    // Display performance metrics
    set_text(
        document,
        "result-pm",
        &format!("{:.1}°", design.performance.phase_margin_deg),
    )?;
    set_text(
        document,
        "result-gm",
        &format!("{:.1} dB", design.performance.gain_margin_db),
    )?;
    set_text(
        document,
        "result-fc",
        &format!("{:.1} kHz", design.performance.crossover_freq_hz / 1e3),
    )?;

    // Show results section
    if let Some(elem) = document.get_element_by_id("results") {
        let elem: HtmlElement = elem.dyn_into()?;
        elem.style().set_property("display", "block")?;
    }

    Ok(())
}

fn set_text(document: &Document, id: &str, text: &str) -> Result<(), JsValue> {
    if let Some(elem) = document.get_element_by_id(id) {
        let elem: HtmlElement = elem.dyn_into()?;
        elem.set_inner_text(text);
    }
    Ok(())
}

fn draw_schematic(document: &Document, design: &dna::pll::PLLDesign) -> Result<(), JsValue> {
    let canvas = document
        .get_element_by_id("schematic-canvas")
        .ok_or("Canvas not found")?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    // Get device pixel ratio for sharp rendering on high-DPI displays
    let window = web_sys::window().ok_or("No window")?;
    let dpr = window.device_pixel_ratio();

    // Get CSS dimensions - need to cast to Element first
    let canvas_element: Element = canvas.clone().into();
    let rect = canvas_element.get_bounding_client_rect();
    let css_width = rect.width();
    let css_height = rect.height();

    // Set actual canvas resolution based on DPR
    canvas.set_width((css_width * dpr) as u32);
    canvas.set_height((css_height * dpr) as u32);

    let ctx = canvas
        .get_context("2d")?
        .ok_or("No 2D context")?
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Scale context to account for DPR
    ctx.scale(dpr, dpr)?;

    let width = css_width;
    let height = css_height;

    // Clear canvas
    ctx.set_fill_style(&JsValue::from_str("#0a0a12"));
    ctx.fill_rect(0.0, 0.0, width, height);

    let block_width = 120.0;
    let block_height = 70.0;
    let y_center = height / 2.0;
    let spacing = 60.0;

    // Calculate positions for 5 blocks horizontally
    let total_width = 5.0 * block_width + 4.0 * spacing;
    let start_x = (width - total_width) / 2.0;

    // Block positions
    let pfd_x = start_x;
    let cp_x = pfd_x + block_width + spacing;
    let filter_x = cp_x + block_width + spacing;
    let vco_x = filter_x + block_width + spacing;
    let div_x = vco_x + block_width + spacing;

    // Draw blocks with enhanced styling
    ctx.set_stroke_style(&JsValue::from_str("#00ffaa"));
    ctx.set_fill_style(&JsValue::from_str("#00ffaa"));
    ctx.set_line_width(2.5);
    ctx.set_font("bold 15px 'SF Mono', Monaco, 'Courier New', monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");

    // PFD block
    draw_block(&ctx, pfd_x, y_center - block_height / 2.0, block_width, block_height, "PFD")?;

    // Charge Pump block
    draw_block(&ctx, cp_x, y_center - block_height / 2.0, block_width, block_height, "Charge\nPump")?;

    // Loop Filter block
    draw_block(&ctx, filter_x, y_center - block_height / 2.0, block_width, block_height, "Loop\nFilter")?;

    // VCO block
    draw_block(&ctx, vco_x, y_center - block_height / 2.0, block_width, block_height, "VCO")?;

    // Divider block
    let n_value = match &design.divider_n {
        dna::pll::DividerConfig::IntegerN { n, .. } => format!("÷{}", n),
        dna::pll::DividerConfig::FractionalN { n_int, .. } => format!("÷{}", n_int),
    };
    draw_block(&ctx, div_x, y_center - block_height / 2.0, block_width, block_height, &n_value)?;

    // Draw forward path connections
    ctx.set_stroke_style(&JsValue::from_str("#606060"));
    ctx.set_line_width(3.0);

    // PFD to CP
    draw_arrow(&ctx, pfd_x + block_width, y_center, cp_x, y_center)?;

    // CP to Filter
    draw_arrow(&ctx, cp_x + block_width, y_center, filter_x, y_center)?;

    // Filter to VCO
    draw_arrow(&ctx, filter_x + block_width, y_center, vco_x, y_center)?;

    // VCO to Divider
    draw_arrow(&ctx, vco_x + block_width, y_center, div_x, y_center)?;

    // Draw feedback path (divider back to PFD)
    let feedback_y = y_center + block_height / 2.0 + 40.0;
    ctx.begin_path();
    ctx.move_to(div_x + block_width / 2.0, y_center + block_height / 2.0);
    ctx.line_to(div_x + block_width / 2.0, feedback_y);
    ctx.line_to(pfd_x + block_width / 2.0, feedback_y);
    ctx.line_to(pfd_x + block_width / 2.0, y_center + block_height / 2.0);
    ctx.stroke();

    // Draw arrow at feedback input to PFD
    draw_arrow_head(&ctx, pfd_x + block_width / 2.0, y_center + block_height / 2.0, 0.0, -1.0)?;

    // Draw reference input
    let ref_x = pfd_x - 60.0;
    draw_arrow(&ctx, ref_x, y_center, pfd_x, y_center)?;
    ctx.set_fill_style(&JsValue::from_str("#808080"));
    ctx.set_font("12px Monaco");
    ctx.set_text_align("right");
    ctx.fill_text(&format!("{:.1} MHz", design.requirements.ref_freq_hz / 1e6), ref_x - 10.0, y_center)?;

    // Draw output
    let out_x = vco_x + block_width + 30.0;
    draw_arrow(&ctx, vco_x + block_width, y_center, out_x, y_center)?;
    ctx.set_text_align("left");
    let output_freq = (design.requirements.output_freq_min_hz + design.requirements.output_freq_max_hz) / 2.0;
    ctx.fill_text(&format!("{:.0} MHz", output_freq / 1e6), out_x + 10.0, y_center)?;

    Ok(())
}

fn draw_block(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    label: &str,
) -> Result<(), JsValue> {
    let corner_radius = 8.0;

    // Draw shadow (offset slightly)
    ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.3)"));
    draw_rounded_rect(ctx, x + 3.0, y + 3.0, width, height, corner_radius);
    ctx.fill();

    // Draw background fill
    ctx.set_fill_style(&JsValue::from_str("rgba(0, 255, 170, 0.08)"));
    draw_rounded_rect(ctx, x, y, width, height, corner_radius);
    ctx.fill();

    // Draw border
    ctx.set_stroke_style(&JsValue::from_str("#00ffaa"));
    ctx.set_line_width(2.5);
    draw_rounded_rect(ctx, x, y, width, height, corner_radius);
    ctx.stroke();

    // Draw label (handle multiline)
    ctx.set_fill_style(&JsValue::from_str("#00ffaa"));
    let lines: Vec<&str> = label.split('\n').collect();
    let line_height = 20.0;
    let total_height = lines.len() as f64 * line_height;
    let start_y = y + height / 2.0 - total_height / 2.0 + line_height / 2.0;

    for (i, line) in lines.iter().enumerate() {
        let line_y = start_y + i as f64 * line_height;
        ctx.fill_text(line, x + width / 2.0, line_y)?;
    }

    Ok(())
}

fn draw_rounded_rect(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
    ctx.begin_path();
    ctx.move_to(x + radius, y);
    ctx.line_to(x + width - radius, y);
    ctx.arc_to(x + width, y, x + width, y + radius, radius).ok();
    ctx.line_to(x + width, y + height - radius);
    ctx.arc_to(x + width, y + height, x + width - radius, y + height, radius).ok();
    ctx.line_to(x + radius, y + height);
    ctx.arc_to(x, y + height, x, y + height - radius, radius).ok();
    ctx.line_to(x, y + radius);
    ctx.arc_to(x, y, x + radius, y, radius).ok();
    ctx.close_path();
}

fn draw_arrow(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> Result<(), JsValue> {
    // Draw line
    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.line_to(x2, y2);
    ctx.stroke();

    // Draw arrowhead
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        let ux = dx / len;
        let uy = dy / len;
        draw_arrow_head(ctx, x2, y2, ux, uy)?;
    }

    Ok(())
}

fn draw_arrow_head(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    ux: f64,
    uy: f64,
) -> Result<(), JsValue> {
    let arrow_size = 8.0;

    ctx.begin_path();
    ctx.move_to(x, y);
    ctx.line_to(
        x - arrow_size * ux - arrow_size * 0.5 * uy,
        y - arrow_size * uy + arrow_size * 0.5 * ux,
    );
    ctx.move_to(x, y);
    ctx.line_to(
        x - arrow_size * ux + arrow_size * 0.5 * uy,
        y - arrow_size * uy - arrow_size * 0.5 * ux,
    );
    ctx.stroke();

    Ok(())
}

fn draw_bode_plot(
    document: &Document,
    bode: &dna::pll::BodePlot,
) -> Result<(), JsValue> {
    let canvas = document
        .get_element_by_id("bode-canvas")
        .ok_or("Canvas not found")?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;

    let ctx = canvas
        .get_context("2d")?
        .ok_or("No 2D context")?
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Clear canvas
    ctx.set_fill_style(&JsValue::from_str("#0a0a12"));
    ctx.fill_rect(0.0, 0.0, width, height);

    // Calculate plot area
    let margin = 60.0;
    let plot_width = width - 2.0 * margin;
    let plot_height = (height - 2.0 * margin) / 2.0; // Two plots (magnitude and phase)

    // Draw magnitude plot
    draw_magnitude_plot(&ctx, bode, margin, margin, plot_width, plot_height)?;

    // Draw phase plot
    draw_phase_plot(
        &ctx,
        bode,
        margin,
        margin + plot_height + 20.0,
        plot_width,
        plot_height,
    )?;

    Ok(())
}

fn draw_magnitude_plot(
    ctx: &CanvasRenderingContext2d,
    bode: &dna::pll::BodePlot,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), JsValue> {
    // Draw axes
    ctx.set_stroke_style(&JsValue::from_str("#404040"));
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.move_to(x, y);
    ctx.line_to(x, y + height);
    ctx.line_to(x + width, y + height);
    ctx.stroke();

    // Find magnitude range
    let mag_min = bode
        .magnitude_db
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let mag_max = bode
        .magnitude_db
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

    let mag_range = mag_max - mag_min;
    let mag_padding = mag_range * 0.1;

    // Draw grid lines and labels
    ctx.set_stroke_style(&JsValue::from_str("#202020"));
    ctx.set_fill_style(&JsValue::from_str("#808080"));
    ctx.set_font("10px Monaco");

    for i in 0..=4 {
        let mag = mag_min - mag_padding + (mag_range + 2.0 * mag_padding) * i as f64 / 4.0;
        let plot_y = y + height - (mag - (mag_min - mag_padding)) / (mag_range + 2.0 * mag_padding) * height;

        ctx.begin_path();
        ctx.move_to(x, plot_y);
        ctx.line_to(x + width, plot_y);
        ctx.stroke();

        ctx.fill_text(&format!("{:.0} dB", mag), x - 45.0, plot_y + 4.0)?;
    }

    // Draw magnitude curve
    ctx.set_stroke_style(&JsValue::from_str("#00ffaa"));
    ctx.set_line_width(2.0);
    ctx.begin_path();

    for (i, &mag) in bode.magnitude_db.iter().enumerate() {
        let freq = bode.frequencies_hz[i];
        let log_freq = freq.log10();
        let freq_min = bode.frequencies_hz[0].log10();
        let freq_max = bode.frequencies_hz[bode.frequencies_hz.len() - 1].log10();

        let plot_x = x + (log_freq - freq_min) / (freq_max - freq_min) * width;
        let plot_y = y + height - (mag - (mag_min - mag_padding)) / (mag_range + 2.0 * mag_padding) * height;

        if i == 0 {
            ctx.move_to(plot_x, plot_y);
        } else {
            ctx.line_to(plot_x, plot_y);
        }
    }
    ctx.stroke();

    // Label
    ctx.set_fill_style(&JsValue::from_str("#00ffaa"));
    ctx.set_font("12px Monaco");
    ctx.fill_text("Magnitude", x + 10.0, y + 20.0)?;

    Ok(())
}

fn draw_phase_plot(
    ctx: &CanvasRenderingContext2d,
    bode: &dna::pll::BodePlot,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), JsValue> {
    // Draw axes
    ctx.set_stroke_style(&JsValue::from_str("#404040"));
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.move_to(x, y);
    ctx.line_to(x, y + height);
    ctx.line_to(x + width, y + height);
    ctx.stroke();

    // Find phase range
    let phase_min = bode
        .phase_deg
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let phase_max = bode
        .phase_deg
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

    let phase_range = phase_max - phase_min;
    let phase_padding = phase_range * 0.1;

    // Draw grid lines and labels
    ctx.set_stroke_style(&JsValue::from_str("#202020"));
    ctx.set_fill_style(&JsValue::from_str("#808080"));
    ctx.set_font("10px Monaco");

    for i in 0..=4 {
        let phase = phase_min - phase_padding + (phase_range + 2.0 * phase_padding) * i as f64 / 4.0;
        let plot_y = y + height - (phase - (phase_min - phase_padding)) / (phase_range + 2.0 * phase_padding) * height;

        ctx.begin_path();
        ctx.move_to(x, plot_y);
        ctx.line_to(x + width, plot_y);
        ctx.stroke();

        ctx.fill_text(&format!("{:.0}°", phase), x - 45.0, plot_y + 4.0)?;
    }

    // Draw frequency labels
    let freq_min = bode.frequencies_hz[0].log10();
    let freq_max = bode.frequencies_hz[bode.frequencies_hz.len() - 1].log10();

    for i in 0..=4 {
        let log_freq = freq_min + (freq_max - freq_min) * i as f64 / 4.0;
        let freq = 10f64.powf(log_freq);
        let plot_x = x + (log_freq - freq_min) / (freq_max - freq_min) * width;

        ctx.fill_text(
            &format!("{:.0} kHz", freq / 1e3),
            plot_x - 20.0,
            y + height + 20.0,
        )?;
    }

    // Draw phase curve
    ctx.set_stroke_style(&JsValue::from_str("#ffaa00"));
    ctx.set_line_width(2.0);
    ctx.begin_path();

    for (i, &phase) in bode.phase_deg.iter().enumerate() {
        let freq = bode.frequencies_hz[i];
        let log_freq = freq.log10();

        let plot_x = x + (log_freq - freq_min) / (freq_max - freq_min) * width;
        let plot_y = y + height - (phase - (phase_min - phase_padding)) / (phase_range + 2.0 * phase_padding) * height;

        if i == 0 {
            ctx.move_to(plot_x, plot_y);
        } else {
            ctx.line_to(plot_x, plot_y);
        }
    }
    ctx.stroke();

    // Label
    ctx.set_fill_style(&JsValue::from_str("#ffaa00"));
    ctx.set_font("12px Monaco");
    ctx.fill_text("Phase", x + 10.0, y + 20.0)?;

    Ok(())
}
