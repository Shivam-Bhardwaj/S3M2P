pub fn run() {
    println!("--- Lesson 00: Rust Refresher (Meta Learning) ---");

    // This lesson doesn't train a model, but generates a visualization 
    // to demonstrate Rust's ownership and borrowing concepts metaphorically.
    
    println!("Generating ownership visualization...");
    
    // We will create a simple visualization of "Memory Ownership"
    // x-axis: Time steps
    // y-axis: Memory Address / Owner ID
    
    let mut x_data = Vec::new();
    let mut y_data = Vec::new(); // 0 = Owner A, 1 = Owner B, 2 = Borrowed
    let mut labels = Vec::new();

    // Step 0-3: Owned by A
    for i in 0..4 {
        x_data.push(i as f64);
        y_data.push(0.0); 
        labels.push("Owned by A".to_string());
    }

    // Step 4-6: Moved to B
    for i in 4..7 {
        x_data.push(i as f64);
        y_data.push(1.0);
        labels.push("Moved to B".to_string());
    }

    // Step 7-9: Borrowed by C (Reference)
    for i in 7..10 {
        x_data.push(i as f64);
        y_data.push(1.5); // Visualization offset
        labels.push("Borrowed by C".to_string());
    }

    // We reuse the classification visualizer but hijack it for this meta-lesson
    // Ideally we would make a custom one, but this proves the point.
    // We treat "y" as the "Class/Owner"
    
    let filename = "lesson_00.json";
    
    // We'll construct a custom JSON manually for this unique lesson
    // instead of using the helper, to show flexibility.
    let json = generate_ownership_json(&x_data, &y_data, &labels);
    
    std::fs::write(filename, json).unwrap();
    println!("Visualization saved to: {}", filename);
}

fn generate_ownership_json(x: &[f64], y: &[f64], labels: &[String]) -> String {
    use serde_json::json;
    
    let mut values = Vec::new();
    for i in 0..x.len() {
        values.push(json!({
            "time": x[i],
            "owner_level": y[i],
            "status": labels[i]
        }));
    }

    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "Rust Ownership Visualization",
        "width": 600,
        "height": 300,
        "data": { "values": values },
        "mark": { "type": "circle", "size": 200 },
        "encoding": {
            "x": { "field": "time", "type": "ordinal", "title": "Time Step" },
            "y": { 
                "field": "owner_level", 
                "type": "nominal", 
                "title": "Memory Owner",
                "scale": { "domain": [0, 1, 1.5], "range": [100, 200, 300] },
                "axis": { "values": [0, 1, 1.5], "labelExpr": "datum.value == 0 ? 'Variable A' : datum.value == 1 ? 'Variable B' : 'Reference C'" }
            },
            "color": { "field": "status", "type": "nominal", "title": "Ownership State" },
            "tooltip": [
                {"field": "time", "title": "Time"},
                {"field": "status", "title": "Status"}
            ]
        }
    });
    
    spec.to_string()
}

