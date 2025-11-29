use ndarray::Array1;
use rand::prelude::*;
use crate::utils::visualization;

pub fn run() {
    println!("--- Lesson 01: Linear Regression from Scratch ---");

    // 1. Data Preparation
    // We want to learn the function: y = 2x + 1
    let true_w = 2.0;
    let true_b = 1.0;
    let num_samples = 100;

    println!("Target function: y = {}x + {}", true_w, true_b);

    // Generate inputs (x) evenly spaced between -1 and 1
    let x: Array1<f64> = Array1::linspace(-1.0, 1.0, num_samples);
    
    // Generate targets (y) with some noise
    let mut rng = rand::rng();
    let noise: Array1<f64> = Array1::from_shape_fn(num_samples, |_| {
        let n: f64 = rng.random_range(-0.5..0.5); // Increased noise for better visualization
        n
    });
    
    let y = &x * true_w + true_b + &noise;

    // 2. Initialization
    // Start with random weights
    let mut w = rng.random::<f64>(); // Random float 0..1
    let mut b = rng.random::<f64>();

    println!("Initial parameters: w = {:.4}, b = {:.4}", w, b);

    // 3. Training Loop (Gradient Descent)
    let learning_rate = 0.1;
    let epochs = 100;

    for epoch in 0..epochs {
        // Forward pass: y_pred = w * x + b
        // Note: ndarray handles broadcasting automatically
        let y_pred = &x * w + b;

        // Calculate Loss (MSE)
        // loss = mean((y_pred - y)^2)
        let error = &y_pred - &y;
        let mse = error.mapv(|e| e.powi(2)).mean().unwrap();

        // Backward pass (Gradients)
        // d_mse/d_w = 2 * mean(error * x)
        // d_mse/d_b = 2 * mean(error)
        
        let dw = 2.0 * (&error * &x).mean().unwrap();
        let db = 2.0 * error.mean().unwrap();

        // Update parameters
        w -= learning_rate * dw;
        b -= learning_rate * db;

        if epoch % 10 == 0 {
            println!("Epoch {}: Loss = {:.6}, w = {:.4}, b = {:.4}", epoch, mse, w, b);
        }
    }

    println!("--- Final Result ---");
    println!("True:      w = {:.4}, b = {:.4}", true_w, true_b);
    println!("Learned:   w = {:.4}, b = {:.4}", w, b);
    
    // Visualization
    println!("Generating interactive visualization...");
    let filename = "lesson_01.json";
    if let Err(e) = visualization::generate_linear_regression_json(
        x.as_slice().unwrap(),
        y.as_slice().unwrap(),
        w,
        b,
        filename
    ) {
        eprintln!("Error generating JSON: {}", e);
    } else {
        println!("Visualization saved to: {}", filename);
    }
    
    println!("-----------------------");
}
