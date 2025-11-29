//! Lesson 11: AGI Architecture (Multimodal Integration & Reasoning)
//!
//! AGI combines multiple AI capabilities into a unified system that can:
//! - Process multiple modalities (text, vision, actions)
//! - Reason and plan across domains
//! - Learn continuously and adapt
//!
//! Key Concepts:
//! - Multimodal Embeddings: Unified representation space
//! - Memory Systems: Short-term and long-term memory
//! - Goal-Directed Behavior: Planning and execution

use rand::prelude::*;
use serde_json::json;
use std::collections::{HashMap, VecDeque};

/// Modality types the AGI can process
#[derive(Clone, Debug, PartialEq)]
enum Modality {
    Text(String),
    Vision(Vec<Vec<f64>>),  // Simplified: 2D grid
    Action(String),
    Goal(String),
}

/// Unified embedding space for all modalities
struct MultimodalEncoder {
    d_model: usize,
    text_embedding: HashMap<String, Vec<f64>>,
    rng: rand::rngs::ThreadRng,
}

impl MultimodalEncoder {
    fn new(d_model: usize) -> Self {
        let mut rng = rand::rng();
        let mut text_embedding = HashMap::new();
        
        // Pre-compute embeddings for common words
        let words = ["cat", "dog", "food", "water", "move", "left", "right", 
                     "up", "down", "find", "eat", "drink", "goal", "hungry", 
                     "thirsty", "explore", "remember", "think", "plan", "act"];
        
        for word in words {
            let embedding: Vec<f64> = (0..d_model)
                .map(|_| rng.random_range(-1.0..1.0))
                .collect();
            text_embedding.insert(word.to_string(), embedding);
        }
        
        Self { d_model, text_embedding, rng }
    }
    
    fn encode(&mut self, modality: &Modality) -> Vec<f64> {
        match modality {
            Modality::Text(text) => {
                let words: Vec<&str> = text.split_whitespace().collect();
                let mut combined = vec![0.0; self.d_model];
                let mut count = 0;
                
                for word in words {
                    if let Some(emb) = self.text_embedding.get(word) {
                        for (i, &e) in emb.iter().enumerate() {
                            combined[i] += e;
                        }
                        count += 1;
                    }
                }
                
                if count > 0 {
                    for v in &mut combined {
                        *v /= count as f64;
                    }
                }
                
                combined
            }
            Modality::Vision(grid) => {
                // Simple CNN-like pooling
                let mut features = vec![0.0; self.d_model];
                let total: f64 = grid.iter().flat_map(|r| r.iter()).sum();
                let avg = total / (grid.len() * grid[0].len()) as f64;
                
                // Encode basic statistics
                features[0] = avg;
                features[1] = grid.len() as f64 / 10.0;
                features[2] = grid[0].len() as f64 / 10.0;
                
                // Add some hash-based features
                for (i, row) in grid.iter().enumerate().take(4) {
                    for (j, &val) in row.iter().enumerate().take(4) {
                        if i * 4 + j + 3 < self.d_model {
                            features[i * 4 + j + 3] = val;
                        }
                    }
                }
                
                features
            }
            Modality::Action(action) => {
                self.encode(&Modality::Text(action.clone()))
            }
            Modality::Goal(goal) => {
                let mut embedding = self.encode(&Modality::Text(goal.clone()));
                // Mark as goal with special feature
                if self.d_model > 0 {
                    embedding[self.d_model - 1] = 1.0;
                }
                embedding
            }
        }
    }
}

/// Working Memory (short-term)
struct WorkingMemory {
    capacity: usize,
    buffer: VecDeque<(Modality, Vec<f64>)>,
}

impl WorkingMemory {
    fn new(capacity: usize) -> Self {
        Self { 
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }
    
    fn push(&mut self, item: Modality, embedding: Vec<f64>) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back((item, embedding));
    }
    
    fn attend(&self, query: &[f64]) -> Vec<f64> {
        // Attention over working memory
        let mut scores = Vec::new();
        
        for (_, embedding) in &self.buffer {
            let dot: f64 = query.iter()
                .zip(embedding.iter())
                .map(|(&q, &e)| q * e)
                .sum();
            scores.push(dot);
        }
        
        // Softmax
        let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp: Vec<f64> = scores.iter().map(|&s| (s - max).exp()).collect();
        let sum: f64 = exp.iter().sum();
        let weights: Vec<f64> = exp.iter().map(|&e| e / (sum + 1e-8)).collect();
        
        // Weighted sum
        let d = if self.buffer.is_empty() { 
            query.len() 
        } else { 
            self.buffer[0].1.len() 
        };
        let mut result = vec![0.0; d];
        
        for (w, (_, emb)) in weights.iter().zip(self.buffer.iter()) {
            for (i, &e) in emb.iter().enumerate() {
                result[i] += w * e;
            }
        }
        
        result
    }
}

/// Long-term Memory (episodic)
struct LongTermMemory {
    episodes: Vec<Episode>,
    max_episodes: usize,
}

#[derive(Clone)]
struct Episode {
    context: Vec<f64>,
    action: String,
    outcome: f64,
    timestamp: usize,
}

impl LongTermMemory {
    fn new(max_episodes: usize) -> Self {
        Self { episodes: Vec::new(), max_episodes }
    }
    
    fn store(&mut self, context: Vec<f64>, action: String, outcome: f64, timestamp: usize) {
        if self.episodes.len() >= self.max_episodes {
            // Remove oldest or least useful
            self.episodes.remove(0);
        }
        self.episodes.push(Episode { context, action, outcome, timestamp });
    }
    
    fn recall(&self, query: &[f64], k: usize) -> Vec<&Episode> {
        // Retrieve k most similar episodes
        let mut scored: Vec<(&Episode, f64)> = self.episodes.iter()
            .map(|ep| {
                let sim: f64 = query.iter()
                    .zip(ep.context.iter())
                    .map(|(&q, &c)| q * c)
                    .sum();
                (ep, sim)
            })
            .collect();
        
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.into_iter().take(k).map(|(ep, _)| ep).collect()
    }
}

/// Reasoning Engine
struct ReasoningEngine {
    d_model: usize,
    w_reason: Vec<Vec<f64>>,
}

impl ReasoningEngine {
    fn new(d_model: usize) -> Self {
        let mut rng = rand::rng();
        let w_reason: Vec<Vec<f64>> = (0..d_model)
            .map(|_| (0..d_model).map(|_| rng.random_range(-0.3..0.3)).collect())
            .collect();
        
        Self { d_model, w_reason }
    }
    
    fn reason(&self, state: &[f64], goal: &[f64], memory_context: &[f64]) -> Vec<f64> {
        // Combine state, goal, and memory for reasoning
        let mut combined = vec![0.0; self.d_model];
        
        for i in 0..self.d_model {
            let s = state.get(i).copied().unwrap_or(0.0);
            let g = goal.get(i).copied().unwrap_or(0.0);
            let m = memory_context.get(i).copied().unwrap_or(0.0);
            combined[i] = s + g * 0.5 + m * 0.3;
        }
        
        // Transform through reasoning layer
        let mut output = vec![0.0; self.d_model];
        for o in 0..self.d_model {
            for (i, &c) in combined.iter().enumerate() {
                output[o] += self.w_reason[i][o] * c;
            }
            output[o] = output[o].tanh();
        }
        
        output
    }
}

/// Action Generator
struct ActionGenerator {
    actions: Vec<String>,
    w_action: Vec<Vec<f64>>,
}

impl ActionGenerator {
    fn new(d_model: usize) -> Self {
        let actions = vec![
            "move left".to_string(),
            "move right".to_string(),
            "move up".to_string(),
            "move down".to_string(),
            "explore".to_string(),
            "interact".to_string(),
            "think".to_string(),
            "remember".to_string(),
        ];
        
        let mut rng = rand::rng();
        let w_action: Vec<Vec<f64>> = (0..d_model)
            .map(|_| (0..actions.len()).map(|_| rng.random_range(-0.3..0.3)).collect())
            .collect();
        
        Self { actions, w_action }
    }
    
    fn generate(&self, reasoning_output: &[f64], rng: &mut impl Rng) -> String {
        // Compute action logits
        let mut logits = vec![0.0; self.actions.len()];
        for (a, logit) in logits.iter_mut().enumerate() {
            for (i, &r) in reasoning_output.iter().enumerate() {
                *logit += self.w_action[i][a] * r;
            }
        }
        
        // Softmax and sample
        let max = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp: Vec<f64> = logits.iter().map(|&l| (l - max).exp()).collect();
        let sum: f64 = exp.iter().sum();
        let probs: Vec<f64> = exp.iter().map(|&e| e / sum).collect();
        
        let r: f64 = rng.random();
        let mut cumsum = 0.0;
        for (i, &p) in probs.iter().enumerate() {
            cumsum += p;
            if r < cumsum {
                return self.actions[i].clone();
            }
        }
        
        self.actions.last().unwrap().clone()
    }
}

/// The AGI Agent
struct AGIAgent {
    encoder: MultimodalEncoder,
    working_memory: WorkingMemory,
    long_term_memory: LongTermMemory,
    reasoning: ReasoningEngine,
    action_gen: ActionGenerator,
    current_goal: Option<Vec<f64>>,
    timestep: usize,
}

impl AGIAgent {
    fn new() -> Self {
        let d_model = 32;
        Self {
            encoder: MultimodalEncoder::new(d_model),
            working_memory: WorkingMemory::new(16),
            long_term_memory: LongTermMemory::new(100),
            reasoning: ReasoningEngine::new(d_model),
            action_gen: ActionGenerator::new(d_model),
            current_goal: None,
            timestep: 0,
        }
    }
    
    fn set_goal(&mut self, goal: &str) {
        let goal_modality = Modality::Goal(goal.to_string());
        let goal_embedding = self.encoder.encode(&goal_modality);
        self.current_goal = Some(goal_embedding);
        println!("AGI Goal set: {}", goal);
    }
    
    fn perceive(&mut self, input: Modality) -> Vec<f64> {
        let embedding = self.encoder.encode(&input);
        self.working_memory.push(input, embedding.clone());
        embedding
    }
    
    fn think(&mut self, rng: &mut impl Rng) -> String {
        // Get current state from working memory
        let goal = self.current_goal.clone().unwrap_or_else(|| vec![0.0; 32]);
        let state = self.working_memory.attend(&goal);
        
        // Recall relevant past experiences
        let past = self.long_term_memory.recall(&state, 3);
        let memory_context: Vec<f64> = if past.is_empty() {
            vec![0.0; 32]
        } else {
            let mut ctx = vec![0.0; past[0].context.len()];
            for ep in &past {
                for (i, &c) in ep.context.iter().enumerate() {
                    ctx[i] += c * ep.outcome;
                }
            }
            ctx
        };
        
        // Reason
        let reasoning_output = self.reasoning.reason(&state, &goal, &memory_context);
        
        // Generate action
        let action = self.action_gen.generate(&reasoning_output, rng);
        
        // Store experience
        self.long_term_memory.store(
            state,
            action.clone(),
            0.5, // Neutral outcome initially
            self.timestep,
        );
        
        self.timestep += 1;
        action
    }
    
    fn get_memory_stats(&self) -> (usize, usize) {
        (self.working_memory.buffer.len(), self.long_term_memory.episodes.len())
    }
}

pub fn run() {
    println!("--- Lesson 11: AGI Architecture ---");
    println!("Building a simple AGI agent with multimodal processing and memory.\n");
    
    let mut rng = rand::rng();
    let mut agent = AGIAgent::new();
    
    // Set a goal
    agent.set_goal("find food");
    
    // Simulation
    println!("\n--- AGI Simulation ---");
    
    let mut actions_taken = Vec::new();
    let mut memory_history = Vec::new();
    
    for step in 0..20 {
        // Simulate environment perception
        let perception = if step % 3 == 0 {
            Modality::Vision(vec![
                vec![0.0, 0.1, 0.0],
                vec![0.2, 0.8, 0.1], // Something interesting at center
                vec![0.0, 0.1, 0.0],
            ])
        } else if step % 3 == 1 {
            Modality::Text("empty room".to_string())
        } else {
            Modality::Text("food nearby".to_string())
        };
        
        // Agent perceives
        let _ = agent.perceive(perception.clone());
        
        // Agent thinks and acts
        let action = agent.think(&mut rng);
        
        let (wm, ltm) = agent.get_memory_stats();
        
        println!("Step {}: Perceived {:?} -> Action: {}", 
                 step, 
                 match &perception {
                     Modality::Text(t) => format!("Text(\"{}\")", t),
                     Modality::Vision(_) => "Vision".to_string(),
                     _ => "?".to_string(),
                 },
                 action);
        
        actions_taken.push((step, action.clone()));
        memory_history.push((step, wm, ltm));
    }
    
    // Print final stats
    let (wm, ltm) = agent.get_memory_stats();
    println!("\n--- Final Memory State ---");
    println!("Working Memory: {} items", wm);
    println!("Long-Term Memory: {} episodes", ltm);
    
    // Explain architecture
    println!("\n--- AGI Architecture Components ---");
    println!("1. Multimodal Encoder: Unified embedding space for text, vision, actions");
    println!("2. Working Memory: Short-term buffer with attention-based retrieval");
    println!("3. Long-Term Memory: Episodic memory for past experiences");
    println!("4. Reasoning Engine: Combines state, goal, and memory for decision making");
    println!("5. Action Generator: Produces actions based on reasoning output");
    
    // Generate visualization
    let mut viz_data = Vec::new();
    
    // Action distribution
    let mut action_counts: HashMap<String, i32> = HashMap::new();
    for (_, action) in &actions_taken {
        *action_counts.entry(action.clone()).or_insert(0) += 1;
    }
    for (action, count) in &action_counts {
        viz_data.push(json!({
            "action": action,
            "count": count,
            "type": "action_dist"
        }));
    }
    
    // Memory growth
    for (step, wm, ltm) in &memory_history {
        viz_data.push(json!({
            "step": step,
            "working_memory": wm,
            "long_term_memory": ltm,
            "type": "memory"
        }));
    }
    
    // Architecture diagram data
    let components = vec![
        ("Perception", 0, 1),
        ("Encoder", 1, 1),
        ("Working Memory", 2, 0),
        ("Long-Term Memory", 2, 2),
        ("Reasoning", 3, 1),
        ("Action Gen", 4, 1),
        ("Output", 5, 1),
    ];
    
    for (name, x, y) in components {
        viz_data.push(json!({
            "component": name,
            "x": x,
            "y": y,
            "type": "architecture"
        }));
    }
    
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "AGI Architecture Visualization",
        "vconcat": [
            {
                "title": "AGI Architecture Overview",
                "width": 500,
                "height": 200,
                "data": { "values": viz_data },
                "transform": [{ "filter": "datum.type == 'architecture'" }],
                "layer": [
                    {
                        "mark": { "type": "rect", "cornerRadius": 5, "stroke": "black" },
                        "encoding": {
                            "x": { "field": "x", "type": "quantitative", "scale": { "domain": [-0.5, 5.5] }, "axis": null },
                            "y": { "field": "y", "type": "quantitative", "scale": { "domain": [-0.5, 2.5] }, "axis": null },
                            "x2": { "value": 50 },
                            "y2": { "value": 30 },
                            "color": { "value": "#3498db" }
                        }
                    },
                    {
                        "mark": { "type": "text", "fontSize": 10, "fontWeight": "bold", "color": "white" },
                        "encoding": {
                            "x": { "field": "x", "type": "quantitative" },
                            "y": { "field": "y", "type": "quantitative" },
                            "text": { "field": "component" }
                        }
                    }
                ]
            },
            {
                "hconcat": [
                    {
                        "title": "Action Distribution",
                        "width": 250,
                        "height": 200,
                        "data": { "values": viz_data },
                        "transform": [{ "filter": "datum.type == 'action_dist'" }],
                        "mark": "bar",
                        "encoding": {
                            "x": { "field": "action", "type": "nominal", "title": "Action" },
                            "y": { "field": "count", "type": "quantitative", "title": "Count" },
                            "color": { "field": "action", "type": "nominal", "legend": null }
                        }
                    },
                    {
                        "title": "Memory Growth Over Time",
                        "width": 250,
                        "height": 200,
                        "data": { "values": viz_data },
                        "transform": [{ "filter": "datum.type == 'memory'" }],
                        "layer": [
                            {
                                "mark": { "type": "line", "color": "blue" },
                                "encoding": {
                                    "x": { "field": "step", "type": "quantitative", "title": "Step" },
                                    "y": { "field": "working_memory", "type": "quantitative", "title": "Items" }
                                }
                            },
                            {
                                "mark": { "type": "line", "color": "green" },
                                "encoding": {
                                    "x": { "field": "step", "type": "quantitative" },
                                    "y": { "field": "long_term_memory", "type": "quantitative" }
                                }
                            }
                        ]
                    }
                ]
            }
        ]
    });
    
    let filename = "lesson_11.json";
    std::fs::write(filename, spec.to_string()).unwrap();
    println!("\nVisualization saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multimodal_encoder() {
        let mut encoder = MultimodalEncoder::new(16);
        
        let text_emb = encoder.encode(&Modality::Text("find food".to_string()));
        assert_eq!(text_emb.len(), 16);
        
        let vision = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let vision_emb = encoder.encode(&Modality::Vision(vision));
        assert_eq!(vision_emb.len(), 16);
    }

    #[test]
    fn test_working_memory() {
        let mut wm = WorkingMemory::new(3);
        
        wm.push(Modality::Text("a".to_string()), vec![1.0, 0.0]);
        wm.push(Modality::Text("b".to_string()), vec![0.0, 1.0]);
        wm.push(Modality::Text("c".to_string()), vec![1.0, 1.0]);
        
        assert_eq!(wm.buffer.len(), 3);
        
        // Adding one more should remove the oldest
        wm.push(Modality::Text("d".to_string()), vec![0.0, 0.0]);
        assert_eq!(wm.buffer.len(), 3);
    }

    #[test]
    fn test_long_term_memory_recall() {
        let mut ltm = LongTermMemory::new(10);
        
        ltm.store(vec![1.0, 0.0], "action1".to_string(), 1.0, 0);
        ltm.store(vec![0.0, 1.0], "action2".to_string(), 0.5, 1);
        
        let query = vec![0.9, 0.1];
        let recalled = ltm.recall(&query, 1);
        
        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].action, "action1"); // Should recall most similar
    }

    #[test]
    fn test_agi_agent_cycle() {
        let mut agent = AGIAgent::new();
        let mut rng = rand::rng();
        
        agent.set_goal("explore");
        agent.perceive(Modality::Text("room".to_string()));
        
        let action = agent.think(&mut rng);
        assert!(!action.is_empty());
        
        let (wm, ltm) = agent.get_memory_stats();
        assert!(wm > 0);
        assert!(ltm > 0);
    }
}

