use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::Write;

// ... existing imports ...

pub fn generate_linear_regression_json(
    x_data: &[f64],
    y_data: &[f64],
    w_learned: f64,
    b_learned: f64,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    // ... existing code ...
    let mut data_values = Vec::new();
    for (i, &x) in x_data.iter().enumerate() {
        data_values.push(json!({
            "x": x,
            "y": y_data[i],
            "type": "Training Data"
        }));
    }

    let x_min = x_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let x_max = x_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    let line_start = w_learned * x_min + b_learned;
    let line_end = w_learned * x_max + b_learned;

    data_values.push(json!({ "x": x_min, "y": line_start, "type": "Learned Line" }));
    data_values.push(json!({ "x": x_max, "y": line_end, "type": "Learned Line" }));

    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "Linear Regression Interactive Plot",
        "width": 600,
        "height": 400,
        "data": { "values": data_values },
        "layer": [
            {
                "mark": "point",
                "encoding": {
                    "x": { "field": "x", "type": "quantitative", "title": "Input (x)" },
                    "y": { "field": "y", "type": "quantitative", "title": "Target (y)" },
                    "color": { "field": "type", "type": "nominal", "legend": {"title": "Legend"} },
                    "tooltip": [
                        {"field": "x", "type": "quantitative"},
                        {"field": "y", "type": "quantitative"}
                    ]
                },
                "transform": [ {"filter": "datum.type == 'Training Data'"} ]
            },
            {
                "mark": { "type": "line", "color": "red", "strokeWidth": 3 },
                "encoding": {
                    "x": { "field": "x", "type": "quantitative" },
                    "y": { "field": "y", "type": "quantitative" }
                },
                "transform": [ {"filter": "datum.type == 'Learned Line'"} ]
            }
        ],
        "selection": {
            "grid": { "type": "interval", "bind": "scales" }
        }
    });

    let mut file = File::create(filename)?;
    file.write_all(spec.to_string().as_bytes())?;
    Ok(())
}

pub fn generate_classification_json(
    x1_data: &[f64],
    x2_data: &[f64],
    labels: &[f64], // 0.0 or 1.0
    w1: f64,
    w2: f64,
    b: f64,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    
    // 1. Prepare Data
    let mut data_values = Vec::new();
    for (i, &x1) in x1_data.iter().enumerate() {
        let label_str = if labels[i] > 0.5 { "Blue Team" } else { "Red Team" };
        data_values.push(json!({
            "x1": x1,
            "x2": x2_data[i],
            "team": label_str,
            "type": "Data"
        }));
    }

    // 2. Calculate Decision Boundary Line
    // Boundary is where w1*x1 + w2*x2 + b = 0
    // => x2 = -(w1/w2)*x1 - (b/w2)
    
    let x1_min = x1_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let x1_max = x1_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    // Avoid division by zero
    if w2.abs() > 1e-5 {
        let x2_start = -(w1 * x1_min + b) / w2;
        let x2_end = -(w1 * x1_max + b) / w2;
        
        data_values.push(json!({ "x1": x1_min, "x2": x2_start, "type": "Boundary" }));
        data_values.push(json!({ "x1": x1_max, "x2": x2_end, "type": "Boundary" }));
    }

    // 3. Build Spec
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "Logistic Regression Classification",
        "width": 600,
        "height": 400,
        "data": { "values": data_values },
        "layer": [
            {
                "mark": "circle",
                "encoding": {
                    "x": { "field": "x1", "type": "quantitative", "title": "Feature 1" },
                    "y": { "field": "x2", "type": "quantitative", "title": "Feature 2" },
                    "color": { 
                        "field": "team", 
                        "type": "nominal", 
                        "scale": {"domain": ["Red Team", "Blue Team"], "range": ["#ff6b6b", "#4dabf7"]},
                        "legend": {"title": "Class"} 
                    },
                    "tooltip": [
                        {"field": "x1", "type": "quantitative"},
                        {"field": "x2", "type": "quantitative"},
                        {"field": "team", "type": "nominal"}
                    ]
                },
                "transform": [ {"filter": "datum.type == 'Data'"} ]
            },
            {
                "mark": { "type": "line", "color": "black", "strokeDash": [4, 4], "strokeWidth": 2 },
                "encoding": {
                    "x": { "field": "x1", "type": "quantitative" },
                    "y": { "field": "x2", "type": "quantitative" }
                },
                "transform": [ {"filter": "datum.type == 'Boundary'"} ]
            }
        ],
        "selection": {
            "grid": { "type": "interval", "bind": "scales" }
        }
    });

    let mut file = File::create(filename)?;
    file.write_all(spec.to_string().as_bytes())?;

    Ok(())
}
