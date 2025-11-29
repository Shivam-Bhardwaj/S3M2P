//! Lesson 05: Policy Networks
//!
//! A Policy Network directly maps states to actions (as probabilities).
//! This is the simplest form of a neural network that can play games.
//!
//! Key Concepts:
//! - Policy: A probability distribution over actions given a state
//! - Softmax: Converts raw outputs to probabilities
//! - Action Sampling: Pick actions based on the policy

use crate::engine::Value;
use rand::prelude::*;
use serde_json::json;

/// Softmax function: converts logits to probabilities
fn softmax(logits: &[f64]) -> Vec<f64> {
    let max = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exp_vals: Vec<f64> = logits.iter().map(|&x| (x - max).exp()).collect();
    let sum: f64 = exp_vals.iter().sum();
    exp_vals.iter().map(|&x| x / sum).collect()
}

/// Simple 2D grid environment
#[derive(Clone, Debug)]
struct GridWorld {
    width: usize,
    height: usize,
    agent_pos: (usize, usize),
    goal_pos: (usize, usize),
}

impl GridWorld {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            agent_pos: (0, 0),
            goal_pos: (width - 1, height - 1),
        }
    }
    
    fn reset(&mut self) {
        self.agent_pos = (0, 0);
    }
    
    fn state(&self) -> Vec<f64> {
        // One-hot encoding of position + goal direction
        let mut state = vec![0.0; self.width * self.height + 4];
        state[self.agent_pos.1 * self.width + self.agent_pos.0] = 1.0;
        
        // Goal direction hints
        if self.goal_pos.0 > self.agent_pos.0 { state[self.width * self.height] = 1.0; }
        if self.goal_pos.0 < self.agent_pos.0 { state[self.width * self.height + 1] = 1.0; }
        if self.goal_pos.1 > self.agent_pos.1 { state[self.width * self.height + 2] = 1.0; }
        if self.goal_pos.1 < self.agent_pos.1 { state[self.width * self.height + 3] = 1.0; }
        
        state
    }
    
    /// Actions: 0=Right, 1=Left, 2=Down, 3=Up
    fn step(&mut self, action: usize) -> (f64, bool) {
        let (x, y) = self.agent_pos;
        
        let new_pos = match action {
            0 => (x.saturating_add(1).min(self.width - 1), y),  // Right
            1 => (x.saturating_sub(1), y),                       // Left
            2 => (x, y.saturating_add(1).min(self.height - 1)), // Down
            3 => (x, y.saturating_sub(1)),                       // Up
            _ => (x, y),
        };
        
        self.agent_pos = new_pos;
        
        let done = self.agent_pos == self.goal_pos;
        let reward = if done { 10.0 } else { -0.1 }; // Small penalty per step
        
        (reward, done)
    }
}

/// Simple Policy Network: state -> action probabilities
struct PolicyNetwork {
    weights: Vec<Vec<Value>>,  // [state_dim x action_dim]
    biases: Vec<Value>,        // [action_dim]
}

impl PolicyNetwork {
    fn new(state_dim: usize, action_dim: usize) -> Self {
        let mut rng = rand::rng();
        
        let weights: Vec<Vec<Value>> = (0..state_dim)
            .map(|_| {
                (0..action_dim)
                    .map(|_| Value::new(rng.random_range(-0.1..0.1)))
                    .collect()
            })
            .collect();
        
        let biases: Vec<Value> = (0..action_dim)
            .map(|_| Value::new(0.0))
            .collect();
        
        Self { weights, biases }
    }
    
    fn forward(&self, state: &[f64]) -> Vec<Value> {
        let action_dim = self.biases.len();
        let mut logits = Vec::with_capacity(action_dim);
        
        for a in 0..action_dim {
            let mut sum = self.biases[a].clone();
            for (s, &state_val) in state.iter().enumerate() {
                if state_val.abs() > 1e-6 {
                    sum = sum + self.weights[s][a].clone() * state_val;
                }
            }
            logits.push(sum);
        }
        
        logits
    }
    
    fn get_action_probs(&self, state: &[f64]) -> Vec<f64> {
        let logits: Vec<f64> = self.forward(state).iter().map(|v| v.data()).collect();
        softmax(&logits)
    }
    
    fn sample_action(&self, state: &[f64], rng: &mut impl Rng) -> usize {
        let probs = self.get_action_probs(state);
        let r: f64 = rng.random();
        let mut cumsum = 0.0;
        for (i, &p) in probs.iter().enumerate() {
            cumsum += p;
            if r < cumsum {
                return i;
            }
        }
        probs.len() - 1
    }
    
    fn update(&mut self, learning_rate: f64) {
        for row in &self.weights {
            for w in row {
                w.apply_gradient_descent(learning_rate);
            }
        }
        for b in &self.biases {
            b.apply_gradient_descent(learning_rate);
        }
    }
    
    fn zero_grad(&self) {
        for row in &self.weights {
            for w in row {
                w.zero_grad();
            }
        }
        for b in &self.biases {
            b.zero_grad();
        }
    }
}

pub fn run() {
    println!("--- Lesson 05: Policy Networks ---");
    
    let mut rng = rand::rng();
    let grid_size = 4;
    let mut env = GridWorld::new(grid_size, grid_size);
    
    let state_dim = grid_size * grid_size + 4; // Grid + direction hints
    let action_dim = 4; // Right, Left, Down, Up
    
    let mut policy = PolicyNetwork::new(state_dim, action_dim);
    
    println!("Training policy network on {}x{} grid...", grid_size, grid_size);
    
    let num_episodes = 500;
    let max_steps = 50;
    let learning_rate = 0.01;
    
    let mut episode_rewards = Vec::new();
    let mut trajectories = Vec::new(); // For visualization
    
    for episode in 0..num_episodes {
        env.reset();
        let mut log_probs = Vec::new();
        let mut rewards = Vec::new();
        let mut trajectory = vec![env.agent_pos];
        
        for _ in 0..max_steps {
            let state = env.state();
            
            // Forward pass with autograd
            let logits = policy.forward(&state);
            let logits_data: Vec<f64> = logits.iter().map(|v| v.data()).collect();
            let probs = softmax(&logits_data);
            
            // Sample action
            let action = policy.sample_action(&state, &mut rng);
            
            // Compute log probability for this action
            let log_prob = (probs[action] + 1e-8).ln();
            
            // Store log_prob (we need the actual Value for backprop)
            // Create a simplified version: use the logit directly
            let action_logit = logits[action].clone();
            log_probs.push((action_logit, log_prob, action));
            
            let (reward, done) = env.step(action);
            rewards.push(reward);
            trajectory.push(env.agent_pos);
            
            if done {
                break;
            }
        }
        
        let total_reward: f64 = rewards.iter().sum();
        episode_rewards.push(total_reward);
        
        // Store some trajectories for visualization
        if episode < 10 || episode >= num_episodes - 10 {
            trajectories.push((episode, trajectory.clone(), total_reward));
        }
        
        // REINFORCE: compute returns and update
        let mut returns = Vec::new();
        let mut g = 0.0;
        for &r in rewards.iter().rev() {
            g = r + 0.99 * g;
            returns.push(g);
        }
        returns.reverse();
        
        // Normalize returns
        let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let std: f64 = (returns.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64).sqrt();
        let normalized: Vec<f64> = returns.iter().map(|&r| (r - mean) / (std + 1e-8)).collect();
        
        // Policy gradient update
        policy.zero_grad();
        
        for (i, (logit, _, _)) in log_probs.iter().enumerate() {
            // Loss = -log_prob * return (we want to maximize, so minimize negative)
            // Simplified: just use the advantage to scale the gradient
            let loss = logit.clone() * (-normalized[i]);
            loss.backward();
        }
        
        policy.update(learning_rate);
        
        if episode % 50 == 0 {
            let avg_reward: f64 = episode_rewards.iter().rev().take(50).sum::<f64>() / 50.0;
            println!("Episode {}: Avg Reward = {:.2}", episode, avg_reward);
        }
    }
    
    // Test the trained policy
    println!("\n--- Testing Trained Policy ---");
    let mut test_rewards = Vec::new();
    for _ in 0..100 {
        env.reset();
        let mut total = 0.0;
        for _ in 0..max_steps {
            let state = env.state();
            let probs = policy.get_action_probs(&state);
            // Greedy action
            let action = probs.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
            let (reward, done) = env.step(action);
            total += reward;
            if done { break; }
        }
        test_rewards.push(total);
    }
    
    let avg_test: f64 = test_rewards.iter().sum::<f64>() / test_rewards.len() as f64;
    let success_rate = test_rewards.iter().filter(|&&r| r > 5.0).count() as f64 / test_rewards.len() as f64;
    println!("Test Avg Reward: {:.2}, Success Rate: {:.1}%", avg_test, success_rate * 100.0);
    
    // Generate visualization
    let mut viz_data = Vec::new();
    
    // Learning curve
    for (i, &r) in episode_rewards.iter().enumerate() {
        viz_data.push(json!({
            "episode": i,
            "reward": r,
            "type": "reward"
        }));
    }
    
    // Grid and trajectories
    for (episode, trajectory, _reward) in &trajectories {
        for (step, &(x, y)) in trajectory.iter().enumerate() {
            viz_data.push(json!({
                "episode": episode,
                "step": step,
                "x": x,
                "y": y,
                "type": "trajectory"
            }));
        }
    }
    
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "Policy Network Learning",
        "vconcat": [
            {
                "title": "Learning Curve",
                "width": 600,
                "height": 200,
                "data": { "values": viz_data },
                "transform": [{ "filter": "datum.type == 'reward'" }],
                "mark": { "type": "line", "opacity": 0.7 },
                "encoding": {
                    "x": { "field": "episode", "type": "quantitative", "title": "Episode" },
                    "y": { "field": "reward", "type": "quantitative", "title": "Total Reward" }
                }
            },
            {
                "title": "Agent Trajectories (Early vs Late Training)",
                "width": 600,
                "height": 300,
                "data": { "values": viz_data },
                "transform": [{ "filter": "datum.type == 'trajectory'" }],
                "mark": { "type": "line", "point": true },
                "encoding": {
                    "x": { "field": "x", "type": "quantitative", "title": "X Position", "scale": { "domain": [0, grid_size] } },
                    "y": { "field": "y", "type": "quantitative", "title": "Y Position", "scale": { "domain": [0, grid_size] } },
                    "color": { 
                        "field": "episode", 
                        "type": "quantitative",
                        "scale": { "scheme": "viridis" },
                        "legend": { "title": "Episode" }
                    },
                    "detail": { "field": "episode" }
                }
            }
        ]
    });
    
    let filename = "lesson_05.json";
    std::fs::write(filename, spec.to_string()).unwrap();
    println!("Visualization saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_softmax() {
        let logits = vec![1.0, 2.0, 3.0];
        let probs = softmax(&logits);
        
        // Sum should be 1
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        
        // Larger logit = larger prob
        assert!(probs[2] > probs[1]);
        assert!(probs[1] > probs[0]);
    }

    #[test]
    fn test_grid_world() {
        let mut env = GridWorld::new(3, 3);
        assert_eq!(env.agent_pos, (0, 0));
        
        // Move right
        env.step(0);
        assert_eq!(env.agent_pos, (1, 0));
        
        // Move down
        env.step(2);
        assert_eq!(env.agent_pos, (1, 1));
        
        // Check goal detection
        env.agent_pos = (2, 2);
        let (reward, done) = env.step(0); // Try to move right (blocked by wall)
        assert!(done);
        assert!(reward > 0.0);
    }

    #[test]
    fn test_policy_network_forward() {
        let policy = PolicyNetwork::new(5, 3);
        let state = vec![1.0, 0.0, 0.0, 0.0, 0.0];
        
        let probs = policy.get_action_probs(&state);
        
        assert_eq!(probs.len(), 3);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }
}

