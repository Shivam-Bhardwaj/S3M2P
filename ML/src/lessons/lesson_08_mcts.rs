//! Lesson 08: Monte Carlo Tree Search (MCTS)
//!
//! MCTS is a search algorithm that builds a game tree by simulating random
//! playouts and using statistics to guide exploration.
//!
//! Key Concepts:
//! - Selection: Use UCB1 to pick promising nodes
//! - Expansion: Add new nodes to the tree
//! - Simulation: Random rollout to estimate value
//! - Backpropagation: Update statistics up the tree

use rand::prelude::*;
use serde_json::json;
use std::collections::HashMap;

/// Simple game: Tic-Tac-Toe
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct TicTacToe {
    board: [[i8; 3]; 3],  // 0=empty, 1=X, -1=O
    current_player: i8,
}

impl TicTacToe {
    fn new() -> Self {
        Self {
            board: [[0; 3]; 3],
            current_player: 1,
        }
    }
    
    fn get_legal_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                if self.board[i][j] == 0 {
                    moves.push((i, j));
                }
            }
        }
        moves
    }
    
    fn make_move(&mut self, pos: (usize, usize)) -> bool {
        if self.board[pos.0][pos.1] != 0 {
            return false;
        }
        self.board[pos.0][pos.1] = self.current_player;
        self.current_player = -self.current_player;
        true
    }
    
    fn check_winner(&self) -> Option<i8> {
        // Check rows
        for i in 0..3 {
            let sum: i8 = self.board[i].iter().sum();
            if sum == 3 { return Some(1); }
            if sum == -3 { return Some(-1); }
        }
        
        // Check columns
        for j in 0..3 {
            let sum: i8 = (0..3).map(|i| self.board[i][j]).sum();
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
    
    fn get_result(&self, player: i8) -> f64 {
        match self.check_winner() {
            Some(winner) if winner == player => 1.0,
            Some(_) => 0.0,
            None => 0.5, // Draw
        }
    }
    
    fn render(&self) -> String {
        let symbols = |v: i8| match v {
            1 => "X",
            -1 => "O",
            _ => ".",
        };
        
        self.board.iter()
            .map(|row| row.iter().map(|&v| symbols(v)).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// MCTS Node
struct MCTSNode {
    state: TicTacToe,
    parent: Option<usize>,
    children: HashMap<(usize, usize), usize>,
    visits: u32,
    wins: f64,
    untried_moves: Vec<(usize, usize)>,
}

impl MCTSNode {
    fn new(state: TicTacToe) -> Self {
        let untried_moves = state.get_legal_moves();
        Self {
            state,
            parent: None,
            children: HashMap::new(),
            visits: 0,
            wins: 0.0,
            untried_moves,
        }
    }
    
    fn ucb1(&self, child_idx: usize, nodes: &[MCTSNode], c: f64) -> f64 {
        let child = &nodes[child_idx];
        if child.visits == 0 {
            return f64::INFINITY;
        }
        
        let exploitation = child.wins / child.visits as f64;
        let exploration = c * ((self.visits as f64).ln() / child.visits as f64).sqrt();
        
        exploitation + exploration
    }
}

/// MCTS Algorithm
struct MCTS {
    nodes: Vec<MCTSNode>,
    exploration_constant: f64,
}

impl MCTS {
    fn new(root_state: TicTacToe) -> Self {
        let root = MCTSNode::new(root_state);
        Self {
            nodes: vec![root],
            exploration_constant: 1.414,
        }
    }
    
    fn select(&self, node_idx: usize) -> usize {
        let node = &self.nodes[node_idx];
        
        if !node.untried_moves.is_empty() || node.state.is_terminal() {
            return node_idx;
        }
        
        // Select child with highest UCB1
        let best_child = node.children.values()
            .max_by(|&&a, &&b| {
                node.ucb1(a, &self.nodes, self.exploration_constant)
                    .partial_cmp(&node.ucb1(b, &self.nodes, self.exploration_constant))
                    .unwrap()
            })
            .copied()
            .unwrap();
        
        self.select(best_child)
    }
    
    fn expand(&mut self, node_idx: usize, rng: &mut impl Rng) -> usize {
        let node = &mut self.nodes[node_idx];
        
        if node.untried_moves.is_empty() {
            return node_idx;
        }
        
        let move_idx = rng.random_range(0..node.untried_moves.len());
        let mv = node.untried_moves.remove(move_idx);
        
        let mut new_state = node.state.clone();
        new_state.make_move(mv);
        
        let new_idx = self.nodes.len();
        let mut new_node = MCTSNode::new(new_state);
        new_node.parent = Some(node_idx);
        
        self.nodes.push(new_node);
        self.nodes[node_idx].children.insert(mv, new_idx);
        
        new_idx
    }
    
    fn simulate(&self, state: &TicTacToe, rng: &mut impl Rng) -> f64 {
        let mut sim_state = state.clone();
        let original_player = -state.current_player; // Player who just moved
        
        while !sim_state.is_terminal() {
            let moves = sim_state.get_legal_moves();
            let mv = moves[rng.random_range(0..moves.len())];
            sim_state.make_move(mv);
        }
        
        sim_state.get_result(original_player)
    }
    
    fn backpropagate(&mut self, mut node_idx: usize, result: f64) {
        loop {
            let node = &mut self.nodes[node_idx];
            node.visits += 1;
            // The result is from the perspective of the player who just moved
            // We need to flip it for the parent (who is the opponent)
            node.wins += result;
            
            if let Some(parent) = node.parent {
                node_idx = parent;
            } else {
                break;
            }
        }
    }
    
    fn search(&mut self, iterations: u32, rng: &mut impl Rng) {
        for _ in 0..iterations {
            // 1. Selection
            let selected = self.select(0);
            
            // 2. Expansion
            let expanded = self.expand(selected, rng);
            
            // 3. Simulation
            let result = self.simulate(&self.nodes[expanded].state, rng);
            
            // 4. Backpropagation
            self.backpropagate(expanded, result);
        }
    }
    
    fn best_move(&self) -> Option<(usize, usize)> {
        let root = &self.nodes[0];
        
        root.children.iter()
            .max_by_key(|&(_, &child_idx)| self.nodes[child_idx].visits)
            .map(|(&mv, _)| mv)
    }
    
    fn get_move_stats(&self) -> Vec<((usize, usize), u32, f64)> {
        let root = &self.nodes[0];
        
        root.children.iter()
            .map(|(&mv, &child_idx)| {
                let child = &self.nodes[child_idx];
                let win_rate = if child.visits > 0 {
                    child.wins / child.visits as f64
                } else {
                    0.0
                };
                (mv, child.visits, win_rate)
            })
            .collect()
    }
}

pub fn run() {
    println!("--- Lesson 08: Monte Carlo Tree Search (MCTS) ---");
    
    let mut rng = rand::rng();
    
    println!("Playing Tic-Tac-Toe: MCTS (X) vs Random (O)\n");
    
    let mut mcts_wins = 0;
    let mut random_wins = 0;
    let mut draws = 0;
    let num_games = 100;
    
    let mut game_trees = Vec::new(); // For visualization
    
    for game_num in 0..num_games {
        let mut game = TicTacToe::new();
        let mut move_history = Vec::new();
        
        while !game.is_terminal() {
            let mv = if game.current_player == 1 {
                // MCTS plays X
                let mut mcts = MCTS::new(game.clone());
                mcts.search(500, &mut rng);
                
                if game_num < 3 {
                    let stats = mcts.get_move_stats();
                    for (mv, visits, win_rate) in stats {
                        game_trees.push(json!({
                            "game": game_num,
                            "move_num": move_history.len(),
                            "row": mv.0,
                            "col": mv.1,
                            "visits": visits,
                            "win_rate": win_rate,
                            "type": "search_stats"
                        }));
                    }
                }
                
                mcts.best_move().unwrap()
            } else {
                // Random plays O
                let moves = game.get_legal_moves();
                moves[rng.random_range(0..moves.len())]
            };
            
            move_history.push((mv, game.current_player));
            game.make_move(mv);
            
            if game_num < 3 {
                for i in 0..3 {
                    for j in 0..3 {
                        game_trees.push(json!({
                            "game": game_num,
                            "move_num": move_history.len(),
                            "row": i,
                            "col": j,
                            "value": game.board[i][j],
                            "symbol": match game.board[i][j] {
                                1 => "X",
                                -1 => "O",
                                _ => ""
                            },
                            "type": "board"
                        }));
                    }
                }
            }
        }
        
        match game.check_winner() {
            Some(1) => mcts_wins += 1,
            Some(-1) => random_wins += 1,
            _ => draws += 1,
        }
        
        if game_num < 3 {
            println!("Game {}:", game_num + 1);
            println!("{}", game.render());
            println!("Winner: {:?}\n", game.check_winner());
        }
    }
    
    println!("Results over {} games:", num_games);
    println!("  MCTS (X) wins: {} ({:.1}%)", mcts_wins, mcts_wins as f64 / num_games as f64 * 100.0);
    println!("  Random (O) wins: {} ({:.1}%)", random_wins, random_wins as f64 / num_games as f64 * 100.0);
    println!("  Draws: {} ({:.1}%)", draws, draws as f64 / num_games as f64 * 100.0);
    
    // Demonstrate MCTS thinking
    println!("\n--- MCTS Decision Making ---");
    
    // Custom position
    let mut demo_game = TicTacToe::new();
    demo_game.board = [
        [1, -1, 0],
        [0, 1, 0],
        [-1, 0, 0],
    ];
    demo_game.current_player = 1;
    
    println!("Position (X to move):");
    println!("{}\n", demo_game.render());
    
    let mut mcts = MCTS::new(demo_game.clone());
    mcts.search(2000, &mut rng);
    
    println!("Move analysis:");
    let mut stats = mcts.get_move_stats();
    stats.sort_by(|a, b| b.1.cmp(&a.1));
    for (mv, visits, win_rate) in &stats {
        println!("  ({}, {}): {} visits, {:.1}% win rate", 
                 mv.0, mv.1, visits, win_rate * 100.0);
    }
    
    let best = mcts.best_move().unwrap();
    println!("\nBest move: ({}, {})", best.0, best.1);
    
    // Generate visualization
    let mut viz_data = game_trees;
    
    // Add stats summary
    viz_data.push(json!({
        "player": "MCTS",
        "wins": mcts_wins,
        "type": "results"
    }));
    viz_data.push(json!({
        "player": "Random",
        "wins": random_wins,
        "type": "results"
    }));
    viz_data.push(json!({
        "player": "Draw",
        "wins": draws,
        "type": "results"
    }));
    
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "MCTS Tic-Tac-Toe Visualization",
        "vconcat": [
            {
                "title": "MCTS vs Random Results",
                "width": 400,
                "height": 200,
                "data": { "values": viz_data },
                "transform": [{ "filter": "datum.type == 'results'" }],
                "mark": "bar",
                "encoding": {
                    "x": { "field": "player", "type": "nominal", "title": "Player" },
                    "y": { "field": "wins", "type": "quantitative", "title": "Games Won" },
                    "color": {
                        "field": "player",
                        "type": "nominal",
                        "scale": { "domain": ["MCTS", "Random", "Draw"], "range": ["#2ecc71", "#e74c3c", "#95a5a6"] }
                    }
                }
            },
            {
                "title": "Sample Game Boards",
                "hconcat": [
                    {
                        "title": "Game 1",
                        "width": 120,
                        "height": 120,
                        "data": { "values": viz_data },
                        "transform": [
                            { "filter": "datum.type == 'board'" },
                            { "filter": "datum.game == 0" },
                            { "filter": "datum.move_num == 9 || datum.move_num == (datum.move_num)" }
                        ],
                        "mark": { "type": "text", "fontSize": 30 },
                        "encoding": {
                            "x": { "field": "col", "type": "ordinal" },
                            "y": { "field": "row", "type": "ordinal", "sort": "descending" },
                            "text": { "field": "symbol", "type": "nominal" },
                            "color": {
                                "field": "value",
                                "type": "quantitative",
                                "scale": { "domain": [-1, 0, 1], "range": ["red", "gray", "blue"] }
                            }
                        }
                    },
                    {
                        "title": "Game 2",
                        "width": 120,
                        "height": 120,
                        "data": { "values": viz_data },
                        "transform": [
                            { "filter": "datum.type == 'board'" },
                            { "filter": "datum.game == 1" }
                        ],
                        "mark": { "type": "text", "fontSize": 30 },
                        "encoding": {
                            "x": { "field": "col", "type": "ordinal" },
                            "y": { "field": "row", "type": "ordinal", "sort": "descending" },
                            "text": { "field": "symbol", "type": "nominal" },
                            "color": {
                                "field": "value",
                                "type": "quantitative",
                                "scale": { "domain": [-1, 0, 1], "range": ["red", "gray", "blue"] }
                            }
                        }
                    }
                ]
            }
        ]
    });
    
    let filename = "lesson_08.json";
    std::fs::write(filename, spec.to_string()).unwrap();
    println!("Visualization saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tictactoe_winner() {
        let mut game = TicTacToe::new();
        
        // X wins with row
        game.board = [
            [1, 1, 1],
            [0, -1, 0],
            [-1, 0, 0],
        ];
        assert_eq!(game.check_winner(), Some(1));
    }

    #[test]
    fn test_tictactoe_column_win() {
        let mut game = TicTacToe::new();
        
        game.board = [
            [-1, 1, 0],
            [-1, 1, 0],
            [-1, 0, 1],
        ];
        assert_eq!(game.check_winner(), Some(-1));
    }

    #[test]
    fn test_tictactoe_diagonal_win() {
        let mut game = TicTacToe::new();
        
        game.board = [
            [1, -1, 0],
            [0, 1, -1],
            [0, 0, 1],
        ];
        assert_eq!(game.check_winner(), Some(1));
    }

    #[test]
    fn test_mcts_finds_winning_move() {
        let mut game = TicTacToe::new();
        game.board = [
            [1, 1, 0],  // X can win by playing (0, 2)
            [-1, -1, 0],
            [0, 0, 0],
        ];
        game.current_player = 1;
        
        let mut mcts = MCTS::new(game);
        let mut rng = rand::rng();
        mcts.search(1000, &mut rng);
        
        let best = mcts.best_move().unwrap();
        assert_eq!(best, (0, 2), "MCTS should find the winning move");
    }

    #[test]
    fn test_mcts_plays_legal_move() {
        // Test that MCTS always returns a legal move
        let mut game = TicTacToe::new();
        game.board = [
            [-1, -1, 0],
            [1, 0, 0],
            [0, 1, 0],
        ];
        game.current_player = 1;
        
        let legal_moves = game.get_legal_moves();
        
        let mut mcts = MCTS::new(game);
        let mut rng = rand::rng();
        mcts.search(500, &mut rng);
        
        let best = mcts.best_move().unwrap();
        assert!(legal_moves.contains(&best), "MCTS should return a legal move");
    }
}

