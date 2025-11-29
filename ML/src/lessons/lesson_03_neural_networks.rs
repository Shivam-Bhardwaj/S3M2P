use crate::engine::Value;
use rand::prelude::*;

pub fn run() {
    println!("--- Lesson 03: Neural Networks (XOR Problem) ---");

    // 1. The Problem: XOR (Exclusive OR)
    // Linear models (like Lesson 2) CANNOT solve this.
    // (0,0) -> 0
    // (0,1) -> 1
    // (1,0) -> 1
    // (1,1) -> 0
    
    let inputs = vec![
        vec![0.0, 0.0],
        vec![0.0, 1.0],
        vec![1.0, 0.0],
        vec![1.0, 1.0],
    ];
    let targets = vec![0.0, 1.0, 1.0, 0.0];

    // 2. The Model: MLP (Multi-Layer Perceptron)
    // 2 Inputs -> 4 Hidden Neurons (Tanh) -> 1 Output Neuron (Linear -> Sigmoid logic via Loss)
    // For simplicity in this engine, we'll implement manually without a Layer struct first.
    
    let mut rng = rand::rng();
    
    // Hidden Layer Weights (2 inputs x 4 neurons)
    let mut w1 = Vec::new();
    for _ in 0..8 { w1.push(Value::new(rng.random_range(-1.0..1.0))); }
    let mut b1 = Vec::new();
    for _ in 0..4 { b1.push(Value::new(0.0)); } // 4 Biases

    // Output Layer Weights (4 inputs x 1 neuron)
    let mut w2 = Vec::new();
    for _ in 0..4 { w2.push(Value::new(rng.random_range(-1.0..1.0))); }
    let b2 = Value::new(0.0);

    println!("Training MLP on XOR...");

    // 3. Training Loop
    let learning_rate = 0.1; // Slower rate for deep nets
    let epochs = 500; // More epochs needed

    for epoch in 0..epochs {
        let mut total_loss = Value::new(0.0);
        
        for (i, x) in inputs.iter().enumerate() {
            // Forward Pass
            let x1 = Value::new(x[0]);
            let x2 = Value::new(x[1]);

            // Hidden Layer (4 Neurons)
            let mut hidden_outs = Vec::new();
            for n in 0..4 {
                // z = w1*x1 + w2*x2 + b
                let z = x1.clone() * w1[n*2].clone() + x2.clone() * w1[n*2+1].clone() + b1[n].clone();
                hidden_outs.push(z.tanh());
            }

            // Output Layer
            let mut final_z = b2.clone();
            for n in 0..4 {
                final_z = final_z + hidden_outs[n].clone() * w2[n].clone();
            }
            
            // Loss (Squared Error for simplicity: (pred - target)^2)
            // Note: Usually we use BCE for classification, but MSE works for XOR demo
            let target = Value::new(targets[i]);
            let diff = final_z - target;
            let loss = diff.pow(2.0);
            
            total_loss = total_loss + loss;
        }

        // Zero Gradients
        for w in w1.iter() { w.zero_grad(); }
        for b in b1.iter() { b.zero_grad(); }
        for w in w2.iter() { w.zero_grad(); }
        b2.zero_grad();

        // Backward Pass (Magic!)
        total_loss.backward();

        // Update Steps (Gradient Descent)
        // w = w - lr * grad
        // Note: We need to mutate data inside Value. In a real generic engine, 
        // we would have a parameter update method. Here we cheat slightly by accessing internal Rc if we could,
        // but our Value struct encapsulates it. We need a helper or access to update.
        // Actually, Value is immutable from outside perspective mostly.
        // We implemented Add/Mul creating NEW Values.
        // To update weights, we need interior mutability access or recreate them?
        // Wait, Value holds Rc<RefCell<ValueInternal>>. We can modify data!
        
        // Let's add a helper to Value to update data given gradient
        update_param(&w1, learning_rate);
        update_param(&b1, learning_rate);
        update_param(&w2, learning_rate);
        update_param_single(&b2, learning_rate);

        if epoch % 50 == 0 {
            println!("Epoch {}: Loss = {:.4}", epoch, total_loss.data());
        }
    }

    println!("--- Final Predictions ---");
    for x in &inputs {
        let (pred, _) = forward(x, &w1, &b1, &w2, &b2);
        println!("Input: {:?}, Pred: {:.4}", x, pred);
    }

    // Visualization
    // We need to generate a heatmap of the decision boundary
    let filename = "lesson_03.json";
    println!("Generating XOR visualization...");
    // Generate heatmap data
    // We'll sample a grid from -0.5 to 1.5
    let mut grid_data = Vec::new();
    let steps = 20;
    for i in 0..steps {
        for j in 0..steps {
            let x = i as f64 / steps as f64 * 1.5 - 0.25;
            let y = j as f64 / steps as f64 * 1.5 - 0.25;
            let (pred, _) = forward(&vec![x, y], &w1, &b1, &w2, &b2);
            grid_data.push((x, y, pred));
        }
    }
    
    let json = generate_xor_json(&inputs, &targets, &grid_data);
    std::fs::write(filename, json).unwrap();
    println!("Visualization saved to: {}", filename);
}

// Helper to run forward pass for inference
fn forward(x: &Vec<f64>, w1: &Vec<Value>, b1: &Vec<Value>, w2: &Vec<Value>, b2: &Value) -> (f64, Value) {
    let x1 = Value::new(x[0]);
    let x2 = Value::new(x[1]);

    let mut hidden_outs = Vec::new();
    for n in 0..4 {
        let z = x1.clone() * w1[n*2].clone() + x2.clone() * w1[n*2+1].clone() + b1[n].clone();
        hidden_outs.push(z.tanh());
    }

    let mut final_z = b2.clone();
    for n in 0..4 {
        final_z = final_z + hidden_outs[n].clone() * w2[n].clone();
    }
    
    (final_z.data(), final_z)
}

// Helper to update parameters since our Value struct hides internals
// We need to expose a method on Value to update data, or use this hack
fn update_param(params: &Vec<Value>, lr: f64) {
    for p in params {
        update_param_single(p, lr);
    }
}

fn update_param_single(p: &Value, lr: f64) {
    // We need to add a method to Value to allow updating data strictly for optimization
    // For now, we will rely on a new method we must add to Value: `apply_gradient_descent`
    p.apply_gradient_descent(lr);
}

fn generate_xor_json(inputs: &Vec<Vec<f64>>, targets: &Vec<f64>, grid: &Vec<(f64, f64, f64)>) -> String {
    use serde_json::json;
    
    let mut values = Vec::new();
    
    // Heatmap Grid
    for (x, y, val) in grid {
        values.push(json!({
            "x": x,
            "y": y,
            "val": val,
            "type": "grid"
        }));
    }

    // True Points
    for (i, inp) in inputs.iter().enumerate() {
        values.push(json!({
            "x": inp[0],
            "y": inp[1],
            "val": targets[i], // Color by target
            "type": "point",
            "label": format!("Target: {}", targets[i])
        }));
    }

    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "XOR Neural Network Decision Boundary",
        "width": 500,
        "height": 400,
        "data": { "values": values },
        "layer": [
            {
                "mark": "rect",
                "encoding": {
                    "x": { "field": "x", "type": "quantitative", "bin": {"maxbins": 20}, "title": "Input 1" },
                    "y": { "field": "y", "type": "quantitative", "bin": {"maxbins": 20}, "title": "Input 2" },
                    "color": { 
                        "field": "val", 
                        "type": "quantitative", 
                        "scale": {"scheme": "purpleorange"},
                        "legend": {"title": "Prediction"}
                    },
                    "tooltip": [{"field": "val", "format": ".2f"}]
                },
                "transform": [{"filter": "datum.type == 'grid'"}]
            },
            {
                "mark": {"type": "circle", "size": 100, "stroke": "black", "strokeWidth": 2},
                "encoding": {
                    "x": { "field": "x", "type": "quantitative" },
                    "y": { "field": "y", "type": "quantitative" },
                    "color": { "field": "val", "type": "quantitative" }, // Match scheme
                    "tooltip": [{"field": "label"}]
                },
                "transform": [{"filter": "datum.type == 'point'"}]
            }
        ]
    });
    
    spec.to_string()
}

