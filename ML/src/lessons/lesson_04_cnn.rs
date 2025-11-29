//! Lesson 04: Convolutional Neural Networks (CNNs)
//!
//! CNNs are the backbone of computer vision. They learn to detect features
//! (edges, corners, textures) by sliding small "filters" over an image.
//!
//! Key Concepts:
//! - Convolution: Sliding a filter over the input
//! - Pooling: Downsampling to reduce computation
//! - Feature Maps: What the CNN "sees" at each layer

use rand::prelude::*;
use serde_json::json;

/// A simple 2D convolution operation (no autograd, for visualization)
fn convolve2d(input: &[Vec<f64>], kernel: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let input_h = input.len();
    let input_w = input[0].len();
    let kernel_h = kernel.len();
    let kernel_w = kernel[0].len();
    
    let out_h = input_h - kernel_h + 1;
    let out_w = input_w - kernel_w + 1;
    
    let mut output = vec![vec![0.0; out_w]; out_h];
    
    for i in 0..out_h {
        for j in 0..out_w {
            let mut sum = 0.0;
            for ki in 0..kernel_h {
                for kj in 0..kernel_w {
                    sum += input[i + ki][j + kj] * kernel[ki][kj];
                }
            }
            output[i][j] = sum;
        }
    }
    
    output
}

/// Max pooling 2x2
fn max_pool2d(input: &[Vec<f64>], pool_size: usize) -> Vec<Vec<f64>> {
    let h = input.len() / pool_size;
    let w = input[0].len() / pool_size;
    
    let mut output = vec![vec![0.0; w]; h];
    
    for i in 0..h {
        for j in 0..w {
            let mut max_val = f64::NEG_INFINITY;
            for pi in 0..pool_size {
                for pj in 0..pool_size {
                    let val = input[i * pool_size + pi][j * pool_size + pj];
                    if val > max_val {
                        max_val = val;
                    }
                }
            }
            output[i][j] = max_val;
        }
    }
    
    output
}

/// ReLU activation
fn relu_2d(input: &[Vec<f64>]) -> Vec<Vec<f64>> {
    input.iter()
        .map(|row| row.iter().map(|&v| if v > 0.0 { v } else { 0.0 }).collect())
        .collect()
}

/// Generate a simple synthetic image with a shape
fn generate_synthetic_image(shape: &str, size: usize) -> Vec<Vec<f64>> {
    let mut img = vec![vec![0.0; size]; size];
    let center = size / 2;
    
    match shape {
        "vertical_line" => {
            for i in 2..(size - 2) {
                img[i][center] = 1.0;
            }
        }
        "horizontal_line" => {
            for j in 2..(size - 2) {
                img[center][j] = 1.0;
            }
        }
        "diagonal" => {
            for i in 2..(size - 2) {
                img[i][i] = 1.0;
            }
        }
        "cross" => {
            for i in 2..(size - 2) {
                img[i][center] = 1.0;
                img[center][i] = 1.0;
            }
        }
        "square" => {
            for i in 3..(size - 3) {
                img[3][i] = 1.0;
                img[size - 4][i] = 1.0;
                img[i][3] = 1.0;
                img[i][size - 4] = 1.0;
            }
        }
        _ => {}
    }
    
    img
}

/// Common edge detection kernels
fn get_kernels() -> Vec<(&'static str, Vec<Vec<f64>>)> {
    vec![
        ("Vertical Edge", vec![
            vec![-1.0, 0.0, 1.0],
            vec![-2.0, 0.0, 2.0],
            vec![-1.0, 0.0, 1.0],
        ]),
        ("Horizontal Edge", vec![
            vec![-1.0, -2.0, -1.0],
            vec![ 0.0,  0.0,  0.0],
            vec![ 1.0,  2.0,  1.0],
        ]),
        ("Diagonal Edge", vec![
            vec![ 0.0,  1.0,  2.0],
            vec![-1.0,  0.0,  1.0],
            vec![-2.0, -1.0,  0.0],
        ]),
        ("Sharpen", vec![
            vec![ 0.0, -1.0,  0.0],
            vec![-1.0,  5.0, -1.0],
            vec![ 0.0, -1.0,  0.0],
        ]),
    ]
}

pub fn run() {
    println!("--- Lesson 04: Convolutional Neural Networks (CNNs) ---");
    
    // 1. Create synthetic images
    let img_size = 16;
    let shapes = ["vertical_line", "horizontal_line", "cross", "square"];
    
    println!("Generating synthetic images...");
    
    // 2. Get edge detection kernels
    let kernels = get_kernels();
    
    // 3. Apply convolutions and collect results for visualization
    let mut all_results = Vec::new();
    
    for shape in &shapes {
        let img = generate_synthetic_image(shape, img_size);
        
        // Store original image
        for (i, row) in img.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                all_results.push(json!({
                    "shape": shape,
                    "layer": "Input",
                    "x": j,
                    "y": i,
                    "value": val
                }));
            }
        }
        
        // Apply each kernel
        for (kernel_name, kernel) in &kernels {
            let conv_result = convolve2d(&img, kernel);
            let activated = relu_2d(&conv_result);
            
            for (i, row) in activated.iter().enumerate() {
                for (j, &val) in row.iter().enumerate() {
                    all_results.push(json!({
                        "shape": shape,
                        "layer": *kernel_name,
                        "x": j,
                        "y": i,
                        "value": val.abs().min(1.0) // Normalize for viz
                    }));
                }
            }
        }
    }
    
    // 4. Demonstrate a simple CNN for binary classification
    println!("\nTraining simple CNN classifier (Cross vs Others)...");
    
    let mut rng = rand::rng();
    
    // Simple learnable 3x3 kernel
    let mut kernel: Vec<Vec<f64>> = (0..3)
        .map(|_| (0..3).map(|_| rng.random_range(-0.5..0.5)).collect())
        .collect();
    
    // Training data
    let cross_imgs: Vec<_> = (0..20).map(|_| {
        let mut img = generate_synthetic_image("cross", img_size);
        // Add noise
        for row in &mut img {
            for val in row {
                *val += rng.random_range(-0.1..0.1);
            }
        }
        (img, 1.0f64) // Label: 1 = cross
    }).collect();
    
    let other_imgs: Vec<_> = (0..20).map(|i| {
        let shape = if i % 2 == 0 { "vertical_line" } else { "horizontal_line" };
        let mut img = generate_synthetic_image(shape, img_size);
        for row in &mut img {
            for val in row {
                *val += rng.random_range(-0.1..0.1);
            }
        }
        (img, 0.0f64) // Label: 0 = not cross
    }).collect();
    
    let mut training_data: Vec<_> = cross_imgs.into_iter().chain(other_imgs.into_iter()).collect();
    
    // Training loop
    let learning_rate = 0.01;
    let epochs = 100;
    
    for epoch in 0..epochs {
        let mut total_loss = 0.0;
        let mut correct = 0;
        
        // Shuffle
        training_data.shuffle(&mut rng);
        
        for (img, target) in &training_data {
            // Forward pass
            let conv = convolve2d(img, &kernel);
            let activated = relu_2d(&conv);
            
            // Global average pooling
            let sum: f64 = activated.iter().flat_map(|r| r.iter()).sum();
            let count = (activated.len() * activated[0].len()) as f64;
            let pooled = sum / count;
            
            // Sigmoid for probability
            let pred = 1.0 / (1.0 + (-pooled * 10.0).exp());
            
            // Binary cross-entropy loss
            let loss = -(target * pred.ln() + (1.0 - target) * (1.0 - pred).ln());
            total_loss += loss;
            
            if (pred > 0.5) == (*target > 0.5) {
                correct += 1;
            }
            
            // Backprop (simplified gradient)
            let d_loss = pred - target; // Gradient of loss w.r.t. pred
            
            // Update kernel (very simplified)
            for ki in 0..3 {
                for kj in 0..3 {
                    let mut grad = 0.0;
                    for i in 0..(img.len() - 2) {
                        for j in 0..(img[0].len() - 2) {
                            if conv[i][j] > 0.0 { // ReLU gradient
                                grad += img[i + ki][j + kj] * d_loss;
                            }
                        }
                    }
                    kernel[ki][kj] -= learning_rate * grad / (count);
                }
            }
        }
        
        if epoch % 20 == 0 {
            let acc = 100.0 * correct as f64 / training_data.len() as f64;
            println!("Epoch {}: Loss = {:.4}, Accuracy = {:.1}%", 
                     epoch, total_loss / training_data.len() as f64, acc);
        }
    }
    
    let final_acc = {
        let mut correct = 0;
        for (img, target) in &training_data {
            let conv = convolve2d(img, &kernel);
            let activated = relu_2d(&conv);
            let sum: f64 = activated.iter().flat_map(|r| r.iter()).sum();
            let count = (activated.len() * activated[0].len()) as f64;
            let pred = 1.0 / (1.0 + (-(sum / count) * 10.0).exp());
            if (pred > 0.5) == (*target > 0.5) {
                correct += 1;
            }
        }
        100.0 * correct as f64 / training_data.len() as f64
    };
    
    println!("Final Accuracy: {:.1}%", final_acc);
    println!("Learned kernel:");
    for row in &kernel {
        println!("  {:?}", row.iter().map(|v| format!("{:.2}", v)).collect::<Vec<_>>());
    }
    
    // Add learned kernel visualization
    for (i, row) in kernel.iter().enumerate() {
        for (j, &val) in row.iter().enumerate() {
            all_results.push(json!({
                "shape": "Learned",
                "layer": "Kernel",
                "x": j,
                "y": i,
                "value": (val + 1.0) / 2.0 // Normalize to 0-1
            }));
        }
    }
    
    // 5. Generate Vega-Lite visualization
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "CNN Feature Maps Visualization",
        "title": "What the CNN 'Sees'",
        "data": { "values": all_results },
        "facet": {
            "column": { "field": "layer", "type": "nominal", "title": "Layer/Filter" },
            "row": { "field": "shape", "type": "nominal", "title": "Input Shape" }
        },
        "spec": {
            "width": 80,
            "height": 80,
            "mark": "rect",
            "encoding": {
                "x": { "field": "x", "type": "ordinal", "axis": null },
                "y": { "field": "y", "type": "ordinal", "axis": null, "sort": "descending" },
                "color": {
                    "field": "value",
                    "type": "quantitative",
                    "scale": { "scheme": "viridis" },
                    "legend": { "title": "Activation" }
                },
                "tooltip": [
                    { "field": "layer" },
                    { "field": "value", "format": ".2f" }
                ]
            }
        }
    });
    
    let filename = "lesson_04.json";
    std::fs::write(filename, spec.to_string()).unwrap();
    println!("Visualization saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convolve2d_identity() {
        // Identity kernel (center = 1)
        let input = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];
        let kernel = vec![
            vec![0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];
        
        let result = convolve2d(&input, &kernel);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert!((result[0][0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_convolve2d_sum() {
        // All-ones kernel sums the values
        let input = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let kernel = vec![
            vec![1.0, 1.0],
            vec![1.0, 1.0],
        ];
        
        let result = convolve2d(&input, &kernel);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 2);
        assert!((result[0][0] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_pool() {
        let input = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![9.0, 10.0, 11.0, 12.0],
            vec![13.0, 14.0, 15.0, 16.0],
        ];
        
        let result = max_pool2d(&input, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 2);
        assert!((result[0][0] - 6.0).abs() < 1e-6);  // max of 1,2,5,6
        assert!((result[0][1] - 8.0).abs() < 1e-6);  // max of 3,4,7,8
        assert!((result[1][0] - 14.0).abs() < 1e-6); // max of 9,10,13,14
        assert!((result[1][1] - 16.0).abs() < 1e-6); // max of 11,12,15,16
    }

    #[test]
    fn test_relu() {
        let input = vec![
            vec![-1.0, 2.0],
            vec![3.0, -4.0],
        ];
        
        let result = relu_2d(&input);
        assert!((result[0][0] - 0.0).abs() < 1e-6);
        assert!((result[0][1] - 2.0).abs() < 1e-6);
        assert!((result[1][0] - 3.0).abs() < 1e-6);
        assert!((result[1][1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_cross() {
        let img = generate_synthetic_image("cross", 8);
        assert_eq!(img.len(), 8);
        assert_eq!(img[0].len(), 8);
        // Center should have the cross
        assert!(img[4][4] > 0.0);
    }
}

