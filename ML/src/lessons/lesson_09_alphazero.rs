//! Lesson 09: AlphaZero - Self-Play Mastery
//!
//! AlphaZero combines MCTS with neural networks. Instead of random rollouts,
//! it uses a neural network to evaluate positions and guide search.
//!
//! Key Concepts:
//! - Neural Network Evaluation: Policy head + Value head
//! - Self-Play: The agent plays against itself to generate training data
//! - PUCT: A variant of UCB1 that uses policy priors

use crate::engine::Value;
use rand::prelude::*;
use serde_json::json;
use std::collections::HashMap;

/// Simple Connect-4 style game (3x3 with gravity)
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct ConnectGame {
    board: [[i8; 3]; 3],
    current_player: i8,
}

impl ConnectGame {
    fn new() -> Self {
        Self {
            board: [[0; 3]; 3],
            current_player: 1,
        }
    }
    
    fn get_legal_moves(&self) -> Vec<usize> {
        // Columns where you can drop a piece
        (0..3).filter(|&col| self.board[0][col] == 0).collect()
    }
    
    fn make_move(&mut self, col: usize) -> bool {
        // Find lowest empty row in column
        for row in (0..3).rev() {
            if self.board[row][col] == 0 {
                self.board[row][col] = self.current_player;
                self.current_player = -self.current_player;
                return true;
            }
        }
        false
    }
    
    fn check_winner(&self) -> Option<i8> {
        // Check rows
        for row in 0..3 {
            let sum: i8 = self.board[row].iter().sum();
            if sum == 3 { return Some(1); }
            if sum == -3 { return Some(-1); }
        }
        
        // Check columns
        for col in 0..3 {
            let sum: i8 = (0..3).map(|row| self.board[row][col]).sum();
            if sum == 3 { return Some(1); }
            if sum == -3 { return Some(-1); }
        }
        
        // Check diagonals
        let diag1: i8 = (0..3).map(|i| self.board[i][i]).sum();
        let diag2: i8 = (0..3).map(|i| self.board[i][2-i]).sum();
        
        if diag1 == 3 || diag2 == 3 { return Some(1); }
        if diag1 == -3 || diag2 == -3 { return Some(-1); }
        
        None
    }
    
    fn is_terminal(&self) -> bool {
        self.check_winner().is_some() || self.get_legal_moves().is_empty()
    }
    
    fn to_features(&self) -> Vec<f64> {
        // Flatten board + current player
        let mut features = Vec::with_capacity(10);
        for row in &self.board {
            for &cell in row {
                features.push(cell as f64);
            }
        }
        features.push(self.current_player as f64);
        features
    }
    
    fn render(&self) -> String {
        let symbols = |v: i8| match v { 1 => "X", -1 => "O", _ => "." };
        self.board.iter()
            .map(|row| row.iter().map(|&v| symbols(v)).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Neural Network for AlphaZero (Policy + Value heads)
struct AlphaNet {
    // Shared backbone
    w1: Vec<Vec<Value>>,  // [10 x 32]
    b1: Vec<Value>,
    
    // Policy head
    w_policy: Vec<Vec<Value>>,  // [32 x 3]
    b_policy: Vec<Value>,
    
    // Value head
    w_value: Vec<Value>,  // [32 x 1]
    b_value: Value,
}

impl AlphaNet {
    fn new() -> Self {
        let mut rng = rand::rng();
        
        let w1: Vec<Vec<Value>> = (0..10)
            .map(|_| (0..32).map(|_| Value::new(rng.random_range(-0.3..0.3))).collect())
            .collect();
        let b1: Vec<Value> = (0..32).map(|_| Value::new(0.0)).collect();
        
        let w_policy: Vec<Vec<Value>> = (0..32)
            .map(|_| (0..3).map(|_| Value::new(rng.random_range(-0.3..0.3))).collect())
            .collect();
        let b_policy: Vec<Value> = (0..3).map(|_| Value::new(0.0)).collect();
        
        let w_value: Vec<Value> = (0..32)
            .map(|_| Value::new(rng.random_range(-0.3..0.3)))
            .collect();
        let b_value = Value::new(0.0);
        
        Self { w1, b1, w_policy, b_policy, w_value, b_value }
    }
    
    fn forward(&self, features: &[f64]) -> (Vec<f64>, f64) {
        // Backbone
        let mut hidden = Vec::with_capacity(32);
        for h in 0..32 {
            let mut sum = self.b1[h].data();
            for (f, &feat) in features.iter().enumerate() {
                sum += self.w1[f][h].data() * feat;
            }
            hidden.push(sum.tanh());
        }
        
        // Policy head (softmax)
        let mut policy_logits = Vec::with_capacity(3);
        for a in 0..3 {
            let mut sum = self.b_policy[a].data();
            for (h, &hid) in hidden.iter().enumerate() {
                sum += self.w_policy[h][a].data() * hid;
            }
            policy_logits.push(sum);
        }
        
        let max_logit = policy_logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_vals: Vec<f64> = policy_logits.iter().map(|&x| (x - max_logit).exp()).collect();
        let sum: f64 = exp_vals.iter().sum();
        let policy: Vec<f64> = exp_vals.iter().map(|&x| x / sum).collect();
        
        // Value head (tanh for [-1, 1])
        let mut value = self.b_value.data();
        for (h, &hid) in hidden.iter().enumerate() {
            value += self.w_value[h].data() * hid;
        }
        let value = value.tanh();
        
        (policy, value)
    }
    
    fn train_step(&mut self, features: &[f64], target_policy: &[f64], target_value: f64, lr: f64) {
        // Forward pass with autograd
        let mut hidden = Vec::with_capacity(32);
        for h in 0..32 {
            let mut sum = self.b1[h].clone();
            for (f, &feat) in features.iter().enumerate() {
                sum = sum + self.w1[f][h].clone() * feat;
            }
            hidden.push(sum.tanh());
        }
        
        // Policy logits
        let mut policy_logits = Vec::with_capacity(3);
        for a in 0..3 {
            let mut sum = self.b_policy[a].clone();
            for (h, hid) in hidden.iter().enumerate() {
                sum = sum + self.w_policy[h][a].clone() * hid.data();
            }
            policy_logits.push(sum);
        }
        
        // Value
        let mut value = self.b_value.clone();
        for (h, hid) in hidden.iter().enumerate() {
            value = value + self.w_value[h].clone() * hid.data();
        }
        let value = value.tanh();
        
        // Loss = MSE(value, target) + CrossEntropy(policy, target)
        let value_loss = (value.clone() - Value::new(target_value)).pow(2.0);
        
        // Simplified cross-entropy using logits
        let logits_data: Vec<f64> = policy_logits.iter().map(|v| v.data()).collect();
        let max_l = logits_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let log_sum_exp: f64 = logits_data.iter().map(|&x| (x - max_l).exp()).sum::<f64>().ln() + max_l;
        
        let mut policy_loss = Value::new(0.0);
        for (a, logit) in policy_logits.iter().enumerate() {
            if target_policy[a] > 0.01 {
                // -target * log(softmax) = -target * (logit - log_sum_exp)
                policy_loss = policy_loss + (logit.clone() * (-target_policy[a]));
            }
        }
        
        let total_loss = value_loss + policy_loss;
        
        // Zero gradients
        for row in &self.w1 { for w in row { w.zero_grad(); } }
        for b in &self.b1 { b.zero_grad(); }
        for row in &self.w_policy { for w in row { w.zero_grad(); } }
        for b in &self.b_policy { b.zero_grad(); }
        for w in &self.w_value { w.zero_grad(); }
        self.b_value.zero_grad();
        
        // Backward
        total_loss.backward();
        
        // Update
        for row in &self.w1 { for w in row { w.apply_gradient_descent(lr); } }
        for b in &self.b1 { b.apply_gradient_descent(lr); }
        for row in &self.w_policy { for w in row { w.apply_gradient_descent(lr); } }
        for b in &self.b_policy { b.apply_gradient_descent(lr); }
        for w in &self.w_value { w.apply_gradient_descent(lr); }
        self.b_value.apply_gradient_descent(lr);
    }
}

/// MCTS with Neural Network guidance
struct AlphaMCTS<'a> {
    root_state: ConnectGame,
    net: &'a AlphaNet,
    visits: HashMap<Vec<i8>, HashMap<usize, u32>>,  // state -> action -> visits
    q_values: HashMap<Vec<i8>, HashMap<usize, f64>>, // state -> action -> Q
    priors: HashMap<Vec<i8>, Vec<f64>>,              // state -> policy prior
    c_puct: f64,
}

impl<'a> AlphaMCTS<'a> {
    fn new(state: ConnectGame, net: &'a AlphaNet) -> Self {
        Self {
            root_state: state,
            net,
            visits: HashMap::new(),
            q_values: HashMap::new(),
            priors: HashMap::new(),
            c_puct: 1.5,
        }
    }
    
    fn state_key(state: &ConnectGame) -> Vec<i8> {
        state.board.iter().flat_map(|r| r.iter()).copied().collect()
    }
    
    fn search(&mut self, iterations: u32, rng: &mut impl Rng) {
        for _ in 0..iterations {
            let mut state = self.root_state.clone();
            let mut path: Vec<(Vec<i8>, usize)> = Vec::new();
            
            // Selection & Expansion
            while !state.is_terminal() {
                let key = Self::state_key(&state);
                let legal_moves = state.get_legal_moves();
                
                if !self.priors.contains_key(&key) {
                    // Expand: get neural network evaluation
                    let (policy, value) = self.net.forward(&state.to_features());
                    self.priors.insert(key.clone(), policy);
                    
                    // Backpropagate the value
                    let mut v = value * state.current_player as f64;
                    for (s, a) in path.iter().rev() {
                        let visits = self.visits.entry(s.clone()).or_default();
                        let q = self.q_values.entry(s.clone()).or_default();
                        
                        let n = visits.entry(*a).or_insert(0);
                        let old_q = q.entry(*a).or_insert(0.0);
                        
                        *n += 1;
                        *old_q += (v - *old_q) / *n as f64;
                        
                        v = -v; // Flip for opponent
                    }
                    break;
                }
                
                // Select action using PUCT
                let prior = self.priors.get(&key).unwrap();
                let visits = self.visits.entry(key.clone()).or_default();
                let q = self.q_values.entry(key.clone()).or_default();
                
                let total_n: u32 = visits.values().sum();
                
                let best_action = legal_moves.iter()
                    .map(|&a| {
                        let n = *visits.get(&a).unwrap_or(&0);
                        let q_val = *q.get(&a).unwrap_or(&0.0);
                        let p = prior[a];
                        
                        let exploration = self.c_puct * p * ((total_n as f64).sqrt()) / (1.0 + n as f64);
                        (a, q_val + exploration)
                    })
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap().0;
                
                path.push((key, best_action));
                state.make_move(best_action);
            }
            
            // Terminal state backprop
            if state.is_terminal() {
                let result = match state.check_winner() {
                    Some(w) => w as f64,
                    None => 0.0,
                };
                
                let mut v = result * state.current_player as f64;
                for (s, a) in path.iter().rev() {
                    let visits = self.visits.entry(s.clone()).or_default();
                    let q = self.q_values.entry(s.clone()).or_default();
                    
                    let n = visits.entry(*a).or_insert(0);
                    let old_q = q.entry(*a).or_insert(0.0);
                    
                    *n += 1;
                    *old_q += (v - *old_q) / *n as f64;
                    
                    v = -v;
                }
            }
        }
    }
    
    fn get_policy(&self, temperature: f64) -> Vec<f64> {
        let key = Self::state_key(&self.root_state);
        let visits = self.visits.get(&key);
        
        let mut policy = vec![0.0; 3];
        
        if let Some(v) = visits {
            if temperature < 0.01 {
                // Argmax
                let best = v.iter().max_by_key(|&(_, &n)| n).map(|(&a, _)| a).unwrap_or(0);
                policy[best] = 1.0;
            } else {
                let counts: Vec<f64> = (0..3).map(|a| (*v.get(&a).unwrap_or(&0) as f64).powf(1.0 / temperature)).collect();
                let sum: f64 = counts.iter().sum();
                if sum > 0.0 {
                    for (i, c) in counts.iter().enumerate() {
                        policy[i] = c / sum;
                    }
                }
            }
        }
        
        policy
    }
}

pub fn run() {
    println!("--- Lesson 09: AlphaZero (Self-Play) ---");
    
    let mut rng = rand::rng();
    let mut net = AlphaNet::new();
    
    println!("Training AlphaZero on Mini-Connect game...\n");
    
    let num_iterations = 20;
    let games_per_iteration = 20;
    let mcts_simulations = 50;
    let learning_rate = 0.01;
    
    let mut training_history = Vec::new();
    
    for iteration in 0..num_iterations {
        let mut training_data = Vec::new();
        let mut outcomes = [0i32; 3]; // wins, losses, draws for player 1
        
        // Self-play
        for _ in 0..games_per_iteration {
            let mut game = ConnectGame::new();
            let mut game_data = Vec::new();
            
            while !game.is_terminal() {
                let mut mcts = AlphaMCTS::new(game.clone(), &net);
                mcts.search(mcts_simulations, &mut rng);
                
                let policy = mcts.get_policy(1.0);
                game_data.push((game.to_features(), policy.clone(), game.current_player));
                
                // Sample action
                let r: f64 = rng.random();
                let mut cumsum = 0.0;
                let mut action = 0;
                for (a, &p) in policy.iter().enumerate() {
                    cumsum += p;
                    if r < cumsum && game.get_legal_moves().contains(&a) {
                        action = a;
                        break;
                    }
                }
                
                game.make_move(action);
            }
            
            // Get outcome
            let outcome = match game.check_winner() {
                Some(1) => { outcomes[0] += 1; 1.0 },
                Some(-1) => { outcomes[1] += 1; -1.0 },
                _ => { outcomes[2] += 1; 0.0 },
            };
            
            // Add to training data with outcome
            for (features, policy, player) in game_data {
                let value = outcome * player as f64;
                training_data.push((features, policy, value));
            }
        }
        
        // Training
        training_data.shuffle(&mut rng);
        for (features, policy, value) in &training_data {
            net.train_step(features, policy, *value, learning_rate);
        }
        
        training_history.push(json!({
            "iteration": iteration,
            "wins": outcomes[0],
            "losses": outcomes[1],
            "draws": outcomes[2],
            "samples": training_data.len()
        }));
        
        if iteration % 5 == 0 {
            println!("Iteration {}: P1 Wins={}, Losses={}, Draws={}, Samples={}", 
                     iteration, outcomes[0], outcomes[1], outcomes[2], training_data.len());
        }
    }
    
    // Evaluate against random
    println!("\n--- Evaluation vs Random ---");
    let mut alpha_wins = 0;
    let mut random_wins = 0;
    let mut draws = 0;
    
    for _ in 0..50 {
        let mut game = ConnectGame::new();
        
        while !game.is_terminal() {
            let action = if game.current_player == 1 {
                let mut mcts = AlphaMCTS::new(game.clone(), &net);
                mcts.search(100, &mut rng);
                let policy = mcts.get_policy(0.0);
                policy.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0
            } else {
                let legal = game.get_legal_moves();
                legal[rng.random_range(0..legal.len())]
            };
            
            game.make_move(action);
        }
        
        match game.check_winner() {
            Some(1) => alpha_wins += 1,
            Some(-1) => random_wins += 1,
            _ => draws += 1,
        }
    }
    
    println!("AlphaZero: {} wins, Random: {} wins, Draws: {}", alpha_wins, random_wins, draws);
    
    // Show example game
    println!("\n--- Example Game ---");
    let mut game = ConnectGame::new();
    while !game.is_terminal() {
        let mut mcts = AlphaMCTS::new(game.clone(), &net);
        mcts.search(100, &mut rng);
        let policy = mcts.get_policy(0.0);
        let action = policy.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        
        println!("Player {} plays column {}", if game.current_player == 1 { "X" } else { "O" }, action);
        game.make_move(action);
        println!("{}\n", game.render());
    }
    
    if let Some(winner) = game.check_winner() {
        println!("Winner: {}", if winner == 1 { "X" } else { "O" });
    } else {
        println!("Draw!");
    }
    
    // Generate visualization
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "AlphaZero Self-Play Training",
        "vconcat": [
            {
                "title": "Training Progress (Self-Play Outcomes)",
                "width": 600,
                "height": 200,
                "data": { "values": training_history },
                "layer": [
                    {
                        "mark": { "type": "line", "point": true },
                        "encoding": {
                            "x": { "field": "iteration", "type": "quantitative", "title": "Iteration" },
                            "y": { "field": "wins", "type": "quantitative", "title": "Count" },
                            "color": { "value": "green" }
                        }
                    },
                    {
                        "mark": { "type": "line", "point": true },
                        "encoding": {
                            "x": { "field": "iteration", "type": "quantitative" },
                            "y": { "field": "losses", "type": "quantitative" },
                            "color": { "value": "red" }
                        }
                    },
                    {
                        "mark": { "type": "line", "point": true },
                        "encoding": {
                            "x": { "field": "iteration", "type": "quantitative" },
                            "y": { "field": "draws", "type": "quantitative" },
                            "color": { "value": "gray" }
                        }
                    }
                ]
            },
            {
                "title": "Final Evaluation Results",
                "width": 300,
                "height": 200,
                "data": {
                    "values": [
                        { "player": "AlphaZero", "wins": alpha_wins },
                        { "player": "Random", "wins": random_wins },
                        { "player": "Draws", "wins": draws }
                    ]
                },
                "mark": "bar",
                "encoding": {
                    "x": { "field": "player", "type": "nominal" },
                    "y": { "field": "wins", "type": "quantitative" },
                    "color": {
                        "field": "player",
                        "scale": { "domain": ["AlphaZero", "Random", "Draws"], "range": ["#3498db", "#e74c3c", "#95a5a6"] }
                    }
                }
            }
        ]
    });
    
    let filename = "lesson_09.json";
    std::fs::write(filename, spec.to_string()).unwrap();
    println!("Visualization saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_game_move() {
        let mut game = ConnectGame::new();
        
        // Piece should fall to bottom
        game.make_move(0);
        assert_eq!(game.board[2][0], 1);
        
        // Next piece stacks on top
        game.make_move(0);
        assert_eq!(game.board[1][0], -1);
    }

    #[test]
    fn test_connect_game_winner() {
        let mut game = ConnectGame::new();
        game.board = [
            [1, 0, 0],
            [1, -1, 0],
            [1, -1, -1],
        ];
        assert_eq!(game.check_winner(), Some(1));
    }

    #[test]
    fn test_alphanet_forward() {
        let net = AlphaNet::new();
        let features = vec![0.0; 10];
        
        let (policy, value) = net.forward(&features);
        
        assert_eq!(policy.len(), 3);
        let sum: f64 = policy.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        assert!(value >= -1.0 && value <= 1.0);
    }
}

