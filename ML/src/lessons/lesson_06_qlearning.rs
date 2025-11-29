//! Lesson 06: Q-Learning
//!
//! Q-Learning learns a "value" for each (state, action) pair.
//! Q(s, a) = Expected future reward if we take action a in state s.
//!
//! Key Concepts:
//! - Q-Table: A lookup table storing Q-values
//! - Bellman Equation: Q(s,a) = r + γ * max_a' Q(s', a')
//! - Epsilon-Greedy: Explore random actions vs exploit known good ones

use rand::prelude::*;
use serde_json::json;
use std::collections::HashMap;

/// Grid World Environment with obstacles
#[derive(Clone)]
struct GridWorld {
    width: usize,
    height: usize,
    agent_pos: (i32, i32),
    goal_pos: (i32, i32),
    obstacles: Vec<(i32, i32)>,
    traps: Vec<(i32, i32)>,
}

impl GridWorld {
    fn new() -> Self {
        // 5x5 grid with obstacles and traps
        Self {
            width: 5,
            height: 5,
            agent_pos: (0, 0),
            goal_pos: (4, 4),
            obstacles: vec![(2, 0), (2, 1), (2, 3), (2, 4)], // Wall with gap
            traps: vec![(1, 2), (3, 1)], // Penalty zones
        }
    }
    
    fn reset(&mut self) -> (i32, i32) {
        self.agent_pos = (0, 0);
        self.agent_pos
    }
    
    fn is_valid(&self, pos: (i32, i32)) -> bool {
        pos.0 >= 0 && pos.0 < self.width as i32 &&
        pos.1 >= 0 && pos.1 < self.height as i32 &&
        !self.obstacles.contains(&pos)
    }
    
    /// Actions: 0=Right, 1=Left, 2=Down, 3=Up
    fn step(&mut self, action: usize) -> ((i32, i32), f64, bool) {
        let (x, y) = self.agent_pos;
        
        let new_pos = match action {
            0 => (x + 1, y),     // Right
            1 => (x - 1, y),     // Left
            2 => (x, y + 1),     // Down
            3 => (x, y - 1),     // Up
            _ => (x, y),
        };
        
        // Only move if valid
        if self.is_valid(new_pos) {
            self.agent_pos = new_pos;
        }
        
        // Calculate reward
        let reward = if self.agent_pos == self.goal_pos {
            100.0  // Goal reached!
        } else if self.traps.contains(&self.agent_pos) {
            -10.0  // Trap penalty
        } else {
            -1.0   // Step penalty
        };
        
        let done = self.agent_pos == self.goal_pos;
        
        (self.agent_pos, reward, done)
    }
    
    fn render(&self) -> Vec<Vec<char>> {
        let mut grid = vec![vec!['.'; self.width]; self.height];
        
        for &(x, y) in &self.obstacles {
            grid[y as usize][x as usize] = '#';
        }
        for &(x, y) in &self.traps {
            grid[y as usize][x as usize] = 'X';
        }
        grid[self.goal_pos.1 as usize][self.goal_pos.0 as usize] = 'G';
        grid[self.agent_pos.1 as usize][self.agent_pos.0 as usize] = 'A';
        
        grid
    }
}

/// Q-Table: stores Q(s, a) values
struct QTable {
    table: HashMap<(i32, i32), [f64; 4]>,
}

impl QTable {
    fn new() -> Self {
        Self { table: HashMap::new() }
    }
    
    fn get(&self, state: (i32, i32), action: usize) -> f64 {
        self.table.get(&state).map(|q| q[action]).unwrap_or(0.0)
    }
    
    fn get_all(&self, state: (i32, i32)) -> [f64; 4] {
        *self.table.get(&state).unwrap_or(&[0.0; 4])
    }
    
    fn set(&mut self, state: (i32, i32), action: usize, value: f64) {
        let q = self.table.entry(state).or_insert([0.0; 4]);
        q[action] = value;
    }
    
    fn best_action(&self, state: (i32, i32)) -> usize {
        let q = self.get_all(state);
        q.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0
    }
    
    fn max_q(&self, state: (i32, i32)) -> f64 {
        let q = self.get_all(state);
        q.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }
}

pub fn run() {
    println!("--- Lesson 06: Q-Learning ---");
    
    let mut rng = rand::rng();
    let mut env = GridWorld::new();
    let mut q_table = QTable::new();
    
    // Hyperparameters
    let alpha = 0.1;       // Learning rate
    let gamma = 0.95;      // Discount factor
    let epsilon_start = 1.0;
    let epsilon_end = 0.01;
    let epsilon_decay = 0.995;
    let num_episodes = 1000;
    let max_steps = 100;
    
    println!("Training Q-Learning agent...");
    println!("Grid: {}x{}, Goal at {:?}", env.width, env.height, env.goal_pos);
    
    let mut epsilon = epsilon_start;
    let mut episode_rewards = Vec::new();
    let mut episode_lengths = Vec::new();
    let mut q_history = Vec::new(); // Track Q-value evolution
    
    for episode in 0..num_episodes {
        let mut state = env.reset();
        let mut total_reward = 0.0;
        let mut steps = 0;
        
        for step in 0..max_steps {
            // Epsilon-greedy action selection
            let action = if rng.random::<f64>() < epsilon {
                rng.random_range(0..4) // Random action
            } else {
                q_table.best_action(state) // Greedy action
            };
            
            let (next_state, reward, done) = env.step(action);
            
            // Q-Learning update: Q(s,a) = Q(s,a) + α * (r + γ * max_a' Q(s',a') - Q(s,a))
            let old_q = q_table.get(state, action);
            let next_max_q = if done { 0.0 } else { q_table.max_q(next_state) };
            let td_target = reward + gamma * next_max_q;
            let new_q = old_q + alpha * (td_target - old_q);
            q_table.set(state, action, new_q);
            
            total_reward += reward;
            steps = step + 1;
            state = next_state;
            
            if done {
                break;
            }
        }
        
        episode_rewards.push(total_reward);
        episode_lengths.push(steps);
        
        // Decay epsilon
        epsilon = (epsilon * epsilon_decay).max(epsilon_end);
        
        // Store Q-value snapshot for start state
        if episode % 50 == 0 {
            let start_q = q_table.get_all((0, 0));
            q_history.push((episode, start_q));
            
            let avg_reward: f64 = episode_rewards.iter().rev().take(50).sum::<f64>() / 50.0_f64.min(episode_rewards.len() as f64);
            println!("Episode {}: Avg Reward = {:.1}, Epsilon = {:.3}, Q(start) = {:?}", 
                     episode, avg_reward, epsilon, 
                     start_q.iter().map(|q| format!("{:.1}", q)).collect::<Vec<_>>());
        }
    }
    
    // Test the trained agent
    println!("\n--- Testing Trained Agent ---");
    
    let mut test_rewards = Vec::new();
    let mut test_paths = Vec::new();
    
    for test_ep in 0..10 {
        let mut state = env.reset();
        let mut total_reward = 0.0;
        let mut path = vec![state];
        
        for _ in 0..max_steps {
            let action = q_table.best_action(state);
            let (next_state, reward, done) = env.step(action);
            path.push(next_state);
            total_reward += reward;
            state = next_state;
            if done { break; }
        }
        
        test_rewards.push(total_reward);
        if test_ep < 3 {
            test_paths.push(path);
        }
    }
    
    let avg_test: f64 = test_rewards.iter().sum::<f64>() / test_rewards.len() as f64;
    let success_rate = test_rewards.iter().filter(|&&r| r > 50.0).count() as f64 / test_rewards.len() as f64;
    println!("Test Avg Reward: {:.1}, Success Rate: {:.1}%", avg_test, success_rate * 100.0);
    
    // Print optimal path
    println!("\nOptimal Path:");
    if let Some(path) = test_paths.first() {
        let action_names = ["→", "←", "↓", "↑"];
        for i in 0..path.len().saturating_sub(1) {
            let dx = path[i + 1].0 - path[i].0;
            let dy = path[i + 1].1 - path[i].1;
            let action = match (dx, dy) {
                (1, 0) => 0, (-1, 0) => 1, (0, 1) => 2, (0, -1) => 3, _ => 0
            };
            print!("{:?}{}", path[i], action_names[action]);
        }
        println!("{:?} (GOAL!)", path.last().unwrap());
    }
    
    // Print Q-table heatmap for visualization
    println!("\nQ-Table (Best Action per State):");
    let grid = env.render();
    for (y, row) in grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == '#' {
                print!(" ### ");
            } else {
                let action = q_table.best_action((x as i32, y as i32));
                let arrows = ["→", "←", "↓", "↑"];
                print!(" {} ", arrows[action]);
            }
        }
        println!();
    }
    
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
    
    // Q-value heatmap
    for y in 0..env.height {
        for x in 0..env.width {
            let pos = (x as i32, y as i32);
            let max_q = q_table.max_q(pos);
            let is_obstacle = env.obstacles.contains(&pos);
            let is_trap = env.traps.contains(&pos);
            let is_goal = pos == env.goal_pos;
            
            viz_data.push(json!({
                "x": x,
                "y": y,
                "q_value": if is_obstacle { 0.0 } else { max_q },
                "cell_type": if is_obstacle { "obstacle" } else if is_trap { "trap" } else if is_goal { "goal" } else { "normal" },
                "type": "heatmap"
            }));
            
            // Best action arrow
            if !is_obstacle {
                let action = q_table.best_action(pos);
                let arrows = ["→", "←", "↓", "↑"];
                viz_data.push(json!({
                    "x": x,
                    "y": y,
                    "action": action,
                    "arrow": arrows[action],
                    "type": "policy"
                }));
            }
        }
    }
    
    // Optimal path
    if let Some(path) = test_paths.first() {
        for (step, &(x, y)) in path.iter().enumerate() {
            viz_data.push(json!({
                "x": x,
                "y": y,
                "step": step,
                "type": "path"
            }));
        }
    }
    
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "Q-Learning Visualization",
        "vconcat": [
            {
                "title": "Learning Curve",
                "width": 600,
                "height": 150,
                "data": { "values": viz_data },
                "transform": [{ "filter": "datum.type == 'reward'" }],
                "mark": { "type": "line", "opacity": 0.5 },
                "encoding": {
                    "x": { "field": "episode", "type": "quantitative" },
                    "y": { "field": "reward", "type": "quantitative", "title": "Episode Reward" }
                }
            },
            {
                "hconcat": [
                    {
                        "title": "Q-Value Heatmap (max Q per cell)",
                        "width": 250,
                        "height": 250,
                        "data": { "values": viz_data },
                        "transform": [{ "filter": "datum.type == 'heatmap'" }],
                        "mark": "rect",
                        "encoding": {
                            "x": { "field": "x", "type": "ordinal", "title": "X" },
                            "y": { "field": "y", "type": "ordinal", "title": "Y", "sort": "descending" },
                            "color": {
                                "field": "q_value",
                                "type": "quantitative",
                                "scale": { "scheme": "viridis" },
                                "legend": { "title": "Max Q" }
                            },
                            "tooltip": [
                                { "field": "cell_type" },
                                { "field": "q_value", "format": ".1f" }
                            ]
                        }
                    },
                    {
                        "title": "Learned Policy (Best Action)",
                        "width": 250,
                        "height": 250,
                        "data": { "values": viz_data },
                        "transform": [{ "filter": "datum.type == 'policy'" }],
                        "mark": { "type": "text", "fontSize": 24 },
                        "encoding": {
                            "x": { "field": "x", "type": "ordinal" },
                            "y": { "field": "y", "type": "ordinal", "sort": "descending" },
                            "text": { "field": "arrow", "type": "nominal" }
                        }
                    }
                ]
            }
        ]
    });
    
    let filename = "lesson_06.json";
    std::fs::write(filename, spec.to_string()).unwrap();
    println!("Visualization saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_world_movement() {
        let mut env = GridWorld::new();
        
        // Start at (0,0)
        assert_eq!(env.agent_pos, (0, 0));
        
        // Move right
        env.step(0);
        assert_eq!(env.agent_pos, (1, 0));
        
        // Move down
        env.step(2);
        assert_eq!(env.agent_pos, (1, 1));
    }

    #[test]
    fn test_grid_world_obstacles() {
        let mut env = GridWorld::new();
        env.agent_pos = (1, 0);
        
        // Try to move right into obstacle at (2, 0)
        env.step(0);
        assert_eq!(env.agent_pos, (1, 0)); // Should stay
    }

    #[test]
    fn test_q_table() {
        let mut q = QTable::new();
        
        q.set((0, 0), 0, 1.0);
        q.set((0, 0), 1, 2.0);
        q.set((0, 0), 2, 0.5);
        q.set((0, 0), 3, 1.5);
        
        assert_eq!(q.best_action((0, 0)), 1); // Action 1 has highest Q
        assert!((q.max_q((0, 0)) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_epsilon_greedy_converges() {
        let mut epsilon = 1.0;
        for _ in 0..1000 {
            epsilon = (epsilon * 0.995_f64).max(0.01);
        }
        assert!(epsilon < 0.02);
    }
}

