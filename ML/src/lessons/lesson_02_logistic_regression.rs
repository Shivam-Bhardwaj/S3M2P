use ndarray::Array1;
use rand::prelude::*;
use crate::utils::visualization;

pub fn run() {
    println!("--- Lesson 02: Logistic Regression (Classification) ---");

    // 1. Data Preparation (2 Features: x1, x2)
    // We will generate two blobs of points.
    // Red Team (0): Centered at (-1, -1)
    // Blue Team (1): Centered at (1, 1)
    
    let num_samples_per_class = 100;
    let total_samples = num_samples_per_class * 2;
    let mut rng = rand::rng();

    let mut x1_data = Vec::with_capacity(total_samples);
    let mut x2_data = Vec::with_capacity(total_samples);
    let mut y_data = Vec::with_capacity(total_samples);

    // Generate Red Team (Class 0)
    for _ in 0..num_samples_per_class {
        x1_data.push(rng.random_range(-2.0..0.5)); // Spread around -1
        x2_data.push(rng.random_range(-2.0..0.5)); 
        y_data.push(0.0);
    }

    // Generate Blue Team (Class 1)
    for _ in 0..num_samples_per_class {
        x1_data.push(rng.random_range(-0.5..2.0)); // Spread around 1
        x2_data.push(rng.random_range(-0.5..2.0));
        y_data.push(1.0);
    }

    // Convert to ndarray for easier math (optional, but good practice)
    let x1 = Array1::from(x1_data.clone());
    let x2 = Array1::from(x2_data.clone());
    let y = Array1::from(y_data.clone());

    // 2. Initialization
    // Model: z = w1*x1 + w2*x2 + b
    //        pred = sigmoid(z)
    
    let mut w1 = rng.random_range(-1.0..1.0);
    let mut w2 = rng.random_range(-1.0..1.0);
    let mut b = 0.0;

    println!("Initial params: w1={:.2}, w2={:.2}, b={:.2}", w1, w2, b);

    // 3. Training Loop (Gradient Descent with Cross-Entropy Loss)
    let learning_rate = 0.1;
    let epochs = 500;

    for epoch in 0..epochs {
        // Forward Pass
        // z = w1*x1 + w2*x2 + b
        let z = &x1 * w1 + &x2 * w2 + b;
        
        // Sigmoid Activation: 1 / (1 + e^-z)
        let pred = z.mapv(|v: f64| 1.0 / (1.0 + (-v).exp()));

        // Loss (Binary Cross Entropy) - For display only
        // L = -mean(y*log(pred) + (1-y)*log(1-pred))
        // We skip strict calculation to avoid log(0) issues for now, focusing on gradients.

        // Backward Pass (Gradients)
        // dL/dz = pred - y  (Beautifully simple!)
        let error = &pred - &y;
        
        let d_w1 = (&error * &x1).mean().unwrap();
        let d_w2 = (&error * &x2).mean().unwrap();
        let d_b = error.mean().unwrap();

        // Update
        w1 -= learning_rate * d_w1;
        w2 -= learning_rate * d_w2;
        b -= learning_rate * d_b;

        if epoch % 50 == 0 {
            // Simple accuracy check
            let accuracy = pred.iter().zip(y.iter()).filter(|(p, t)| {
                let class = if **p > 0.5 { 1.0 } else { 0.0 };
                (class - **t).abs() < 0.1f64
            }).count() as f64 / total_samples as f64;
            
            println!("Epoch {}: Accuracy = {:.2}%, w1={:.2}, w2={:.2}, b={:.2}", 
                epoch, accuracy * 100.0, w1, w2, b);
        }
    }

    println!("--- Final Result ---");
    println!("Learned Boundary: {:.2}x1 + {:.2}x2 + {:.2} = 0", w1, w2, b);

    // Visualization
    let filename = "lesson_02.json";
    println!("Generating classification visualization...");
    if let Err(e) = visualization::generate_classification_json(
        &x1_data,
        &x2_data,
        &y_data,
        w1,
        w2,
        b,
        filename
    ) {
        eprintln!("Error generating JSON: {}", e);
    } else {
        println!("Visualization saved to: {}", filename);
    }
}

