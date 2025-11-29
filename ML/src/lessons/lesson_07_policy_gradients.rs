//! Lesson 07: Policy Gradients (REINFORCE with Baseline)
//!
//! Policy Gradients optimize the policy directly by computing gradients
//! of expected reward with respect to policy parameters.
//!
//! Key Concepts:
//! - REINFORCE Algorithm: ∇J(θ) = E[∇log π(a|s) * G]
//! - Baseline: Subtract a value estimate to reduce variance
//! - Actor-Critic: Learn both policy (actor) and value (critic)

use crate::engine::Value;
use rand::prelude::*;
use serde_json::json;

/// CartPole-like environment
/// State: [position, velocity, angle, angular_velocity]
/// Actions: 0 = push left, 1 = push right
#[derive(Clone)]
struct CartPole {
    x: f64,         // Cart position
    v: f64,         // Cart velocity
    theta: f64,     // Pole angle (radians)
    omega: f64,     // Pole angular velocity
    gravity: f64,
    mass_cart: f64,
    mass_pole: f64,
    length: f64,
    force_mag: f64,
    dt: f64,
}

impl CartPole {
    fn new() -> Self {
        Self {
            x: 0.0,
            v: 0.0,
            theta: 0.0,
            omega: 0.0,
            gravity: 9.8,
            mass_cart: 1.0,
            mass_pole: 0.1,
            length: 0.5,
            force_mag: 10.0,
            dt: 0.02,
        }
    }
    
    fn reset(&mut self, rng: &mut impl Rng) -> Vec<f64> {
        self.x = rng.random_range(-0.05..0.05);
        self.v = rng.random_range(-0.05..0.05);
        self.theta = rng.random_range(-0.05..0.05);
        self.omega = rng.random_range(-0.05..0.05);
        self.state()
    }
    
    fn state(&self) -> Vec<f64> {
        // Normalize state for neural network
        vec![
            self.x / 2.4,
            self.v / 4.0,
            self.theta / 0.21,
            self.omega / 4.0,
        ]
    }
    
    fn step(&mut self, action: usize) -> (Vec<f64>, f64, bool) {
        let force = if action == 1 { self.force_mag } else { -self.force_mag };
        
        let cos_theta = self.theta.cos();
        let sin_theta = self.theta.sin();
        
        let total_mass = self.mass_cart + self.mass_pole;
        let pole_mass_length = self.mass_pole * self.length;
        
        let temp = (force + pole_mass_length * self.omega.powi(2) * sin_theta) / total_mass;
        let theta_acc = (self.gravity * sin_theta - cos_theta * temp) /
            (self.length * (4.0/3.0 - self.mass_pole * cos_theta.powi(2) / total_mass));
        let x_acc = temp - pole_mass_length * theta_acc * cos_theta / total_mass;
        
        // Euler integration
        self.x += self.dt * self.v;
        self.v += self.dt * x_acc;
        self.theta += self.dt * self.omega;
        self.omega += self.dt * theta_acc;
        
        // Check termination
        let done = self.x.abs() > 2.4 || self.theta.abs() > 0.21;
        let reward = if done { 0.0 } else { 1.0 };
        
        (self.state(), reward, done)
    }
}

/// Actor Network: state -> action probabilities
struct Actor {
    w1: Vec<Vec<Value>>,  // [4 x 16]
    b1: Vec<Value>,       // [16]
    w2: Vec<Vec<Value>>,  // [16 x 2]
    b2: Vec<Value>,       // [2]
}

impl Actor {
    fn new() -> Self {
        let mut rng = rand::rng();
        
        let w1: Vec<Vec<Value>> = (0..4)
            .map(|_| (0..16).map(|_| Value::new(rng.random_range(-0.3..0.3))).collect())
            .collect();
        let b1: Vec<Value> = (0..16).map(|_| Value::new(0.0)).collect();
        
        let w2: Vec<Vec<Value>> = (0..16)
            .map(|_| (0..2).map(|_| Value::new(rng.random_range(-0.3..0.3))).collect())
            .collect();
        let b2: Vec<Value> = (0..2).map(|_| Value::new(0.0)).collect();
        
        Self { w1, b1, w2, b2 }
    }
    
    fn forward(&self, state: &[f64]) -> (Vec<Value>, Vec<f64>) {
        // Hidden layer
        let mut hidden = Vec::with_capacity(16);
        for h in 0..16 {
            let mut sum = self.b1[h].clone();
            for (s, &state_val) in state.iter().enumerate() {
                sum = sum + self.w1[s][h].clone() * state_val;
            }
            hidden.push(sum.tanh());
        }
        
        // Output layer (logits)
        let mut logits = Vec::with_capacity(2);
        for o in 0..2 {
            let mut sum = self.b2[o].clone();
            for (h, hidden_val) in hidden.iter().enumerate() {
                sum = sum + self.w2[h][o].clone() * hidden_val.data();
            }
            logits.push(sum);
        }
        
        // Softmax for probabilities
        let logits_data: Vec<f64> = logits.iter().map(|v| v.data()).collect();
        let max_logit = logits_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_vals: Vec<f64> = logits_data.iter().map(|&x| (x - max_logit).exp()).collect();
        let sum: f64 = exp_vals.iter().sum();
        let probs: Vec<f64> = exp_vals.iter().map(|&x| x / sum).collect();
        
        (logits, probs)
    }
    
    fn get_params(&self) -> Vec<&Value> {
        let mut params = Vec::new();
        for row in &self.w1 { for w in row { params.push(w); } }
        for b in &self.b1 { params.push(b); }
        for row in &self.w2 { for w in row { params.push(w); } }
        for b in &self.b2 { params.push(b); }
        params
    }
    
    fn update(&self, lr: f64) {
        for p in self.get_params() {
            p.apply_gradient_descent(lr);
        }
    }
    
    fn zero_grad(&self) {
        for p in self.get_params() {
            p.zero_grad();
        }
    }
}

/// Critic Network: state -> value estimate
struct Critic {
    w1: Vec<Vec<f64>>,  // [4 x 16]
    b1: Vec<f64>,       // [16]
    w2: Vec<f64>,       // [16 x 1]
    b2: f64,
}

impl Critic {
    fn new() -> Self {
        let mut rng = rand::rng();
        
        let w1: Vec<Vec<f64>> = (0..4)
            .map(|_| (0..16).map(|_| rng.random_range(-0.3..0.3)).collect())
            .collect();
        let b1: Vec<f64> = vec![0.0; 16];
        let w2: Vec<f64> = (0..16).map(|_| rng.random_range(-0.3..0.3)).collect();
        let b2 = 0.0;
        
        Self { w1, b1, w2, b2 }
    }
    
    fn forward(&self, state: &[f64]) -> f64 {
        let mut hidden = vec![0.0; 16];
        for h in 0..16 {
            let mut sum = self.b1[h];
            for (s, &state_val) in state.iter().enumerate() {
                sum += self.w1[s][h] * state_val;
            }
            hidden[h] = sum.tanh();
        }
        
        let mut value = self.b2;
        for (h, &hidden_val) in hidden.iter().enumerate() {
            value += self.w2[h] * hidden_val;
        }
        
        value
    }
    
    fn update(&mut self, state: &[f64], target: f64, lr: f64) {
        let pred = self.forward(state);
        let error = pred - target;
        
        // Simple gradient descent on MSE
        // (Simplified: not using autograd for critic)
        let mut hidden = vec![0.0; 16];
        for h in 0..16 {
            let mut sum = self.b1[h];
            for (s, &state_val) in state.iter().enumerate() {
                sum += self.w1[s][h] * state_val;
            }
            hidden[h] = sum.tanh();
        }
        
        // d_loss/d_w2 = error * hidden * d_tanh
        for h in 0..16 {
            self.w2[h] -= lr * error * hidden[h];
        }
        self.b2 -= lr * error;
        
        // Backprop to w1 (simplified)
        for h in 0..16 {
            let d_hidden = error * self.w2[h] * (1.0 - hidden[h].powi(2));
            for (s, &state_val) in state.iter().enumerate() {
                self.w1[s][h] -= lr * d_hidden * state_val;
            }
            self.b1[h] -= lr * d_hidden;
        }
    }
}

pub fn run() {
    println!("--- Lesson 07: Policy Gradients (Actor-Critic) ---");
    
    let mut rng = rand::rng();
    let mut env = CartPole::new();
    let actor = Actor::new();
    let mut critic = Critic::new();
    
    let num_episodes = 500;
    let max_steps = 200;
    let gamma = 0.99;
    let actor_lr = 0.001;
    let critic_lr = 0.01;
    
    println!("Training Actor-Critic on CartPole...");
    
    let mut episode_lengths = Vec::new();
    
    for episode in 0..num_episodes {
        let mut state = env.reset(&mut rng);
        let mut log_probs = Vec::new();
        let mut rewards = Vec::new();
        let mut values = Vec::new();
        let mut states = Vec::new();
        
        for _ in 0..max_steps {
            states.push(state.clone());
            
            let (logits, probs) = actor.forward(&state);
            let value = critic.forward(&state);
            values.push(value);
            
            // Sample action
            let action = if rng.random::<f64>() < probs[0] { 0 } else { 1 };
            
            // Store log probability
            log_probs.push((logits[action].clone(), probs[action].ln()));
            
            let (next_state, reward, done) = env.step(action);
            rewards.push(reward);
            state = next_state;
            
            if done {
                break;
            }
        }
        
        episode_lengths.push(rewards.len());
        
        // Compute returns and advantages
        let mut returns = vec![0.0; rewards.len()];
        let mut g = 0.0;
        for t in (0..rewards.len()).rev() {
            g = rewards[t] + gamma * g;
            returns[t] = g;
        }
        
        // Update critic
        for (t, state) in states.iter().enumerate() {
            critic.update(state, returns[t], critic_lr);
        }
        
        // Compute advantages (return - baseline)
        let advantages: Vec<f64> = returns.iter()
            .zip(values.iter())
            .map(|(&r, &v)| r - v)
            .collect();
        
        // Normalize advantages
        let mean: f64 = advantages.iter().sum::<f64>() / advantages.len() as f64;
        let std: f64 = (advantages.iter().map(|&a| (a - mean).powi(2)).sum::<f64>() / advantages.len() as f64).sqrt();
        let norm_adv: Vec<f64> = advantages.iter().map(|&a| (a - mean) / (std + 1e-8)).collect();
        
        // Update actor
        actor.zero_grad();
        for (t, (logit, _)) in log_probs.iter().enumerate() {
            let loss = logit.clone() * (-norm_adv[t]);
            loss.backward();
        }
        actor.update(actor_lr);
        
        if episode % 50 == 0 {
            let avg_len: f64 = episode_lengths.iter().rev().take(50).map(|&l| l as f64).sum::<f64>() / 50.0_f64.min(episode_lengths.len() as f64);
            println!("Episode {}: Avg Length = {:.1}", episode, avg_len);
        }
    }
    
    // Test the trained policy
    println!("\n--- Testing Trained Policy ---");
    let mut test_lengths = Vec::new();
    for _ in 0..100 {
        let mut state = env.reset(&mut rng);
        let mut steps = 0;
        for _ in 0..max_steps {
            let (_, probs) = actor.forward(&state);
            let action = if probs[0] > probs[1] { 0 } else { 1 };
            let (next_state, _, done) = env.step(action);
            state = next_state;
            steps += 1;
            if done { break; }
        }
        test_lengths.push(steps);
    }
    
    let avg_test: f64 = test_lengths.iter().map(|&l| l as f64).sum::<f64>() / test_lengths.len() as f64;
    let max_test = test_lengths.iter().max().unwrap();
    println!("Test Avg Length: {:.1}, Max: {}", avg_test, max_test);
    
    // Generate visualization
    let mut viz_data = Vec::new();
    
    // Learning curve
    for (i, &len) in episode_lengths.iter().enumerate() {
        viz_data.push(json!({
            "episode": i,
            "length": len,
            "type": "training"
        }));
    }
    
    // Moving average
    let window = 20;
    for i in window..episode_lengths.len() {
        let avg: f64 = episode_lengths[i-window..i].iter().map(|&l| l as f64).sum::<f64>() / window as f64;
        viz_data.push(json!({
            "episode": i,
            "length": avg,
            "type": "moving_avg"
        }));
    }
    
    // Test episode simulation
    let mut state = env.reset(&mut rng);
    for step in 0..max_steps {
        viz_data.push(json!({
            "step": step,
            "x": state[0] * 2.4,
            "theta": state[2] * 0.21 * 180.0 / std::f64::consts::PI,
            "type": "simulation"
        }));
        
        let (_, probs) = actor.forward(&state);
        let action = if probs[0] > probs[1] { 0 } else { 1 };
        let (next_state, _, done) = env.step(action);
        state = next_state;
        if done { break; }
    }
    
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "Policy Gradients (Actor-Critic) on CartPole",
        "vconcat": [
            {
                "title": "Episode Length During Training",
                "width": 600,
                "height": 200,
                "layer": [
                    {
                        "data": { "values": viz_data },
                        "transform": [{ "filter": "datum.type == 'training'" }],
                        "mark": { "type": "point", "opacity": 0.3, "size": 10 },
                        "encoding": {
                            "x": { "field": "episode", "type": "quantitative" },
                            "y": { "field": "length", "type": "quantitative", "title": "Steps Survived" }
                        }
                    },
                    {
                        "data": { "values": viz_data },
                        "transform": [{ "filter": "datum.type == 'moving_avg'" }],
                        "mark": { "type": "line", "color": "red", "strokeWidth": 2 },
                        "encoding": {
                            "x": { "field": "episode", "type": "quantitative" },
                            "y": { "field": "length", "type": "quantitative" }
                        }
                    }
                ]
            },
            {
                "title": "Trained Policy Simulation (Cart Position & Pole Angle)",
                "width": 600,
                "height": 200,
                "data": { "values": viz_data },
                "transform": [{ "filter": "datum.type == 'simulation'" }],
                "layer": [
                    {
                        "mark": "line",
                        "encoding": {
                            "x": { "field": "step", "type": "quantitative", "title": "Time Step" },
                            "y": { "field": "x", "type": "quantitative", "title": "Cart Position" },
                            "color": { "value": "blue" }
                        }
                    },
                    {
                        "mark": { "type": "line", "strokeDash": [4, 4] },
                        "encoding": {
                            "x": { "field": "step", "type": "quantitative" },
                            "y": { "field": "theta", "type": "quantitative", "title": "Pole Angle (deg)" },
                            "color": { "value": "orange" }
                        }
                    }
                ]
            }
        ]
    });
    
    let filename = "lesson_07.json";
    std::fs::write(filename, spec.to_string()).unwrap();
    println!("Visualization saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cartpole_step() {
        let mut env = CartPole::new();
        let mut rng = rand::rng();
        env.reset(&mut rng);
        
        // Take some steps
        for _ in 0..10 {
            let (state, reward, done) = env.step(1);
            assert_eq!(state.len(), 4);
            if done { break; }
            assert!(reward >= 0.0);
        }
    }

    #[test]
    fn test_actor_forward() {
        let actor = Actor::new();
        let state = vec![0.0, 0.0, 0.0, 0.0];
        
        let (logits, probs) = actor.forward(&state);
        
        assert_eq!(logits.len(), 2);
        assert_eq!(probs.len(), 2);
        
        // Probabilities should sum to 1
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_critic_forward() {
        let critic = Critic::new();
        let state = vec![0.0, 0.0, 0.0, 0.0];
        
        let value = critic.forward(&state);
        
        // Value should be finite
        assert!(value.is_finite());
    }

    #[test]
    fn test_discount_returns() {
        let rewards = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let gamma = 0.99;
        
        let mut returns = vec![0.0; rewards.len()];
        let mut g = 0.0;
        for t in (0..rewards.len()).rev() {
            g = rewards[t] + gamma * g;
            returns[t] = g;
        }
        
        // First return should be highest
        assert!(returns[0] > returns[4]);
        // Last return should be just the last reward
        assert!((returns[4] - 1.0_f64).abs() < 1e-6);
    }
}

