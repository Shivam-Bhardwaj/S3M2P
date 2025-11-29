//! Lesson 10: Large Language Models (Transformers & Attention)
//!
//! The Transformer architecture revolutionized NLP. The key insight:
//! Attention allows the model to focus on relevant parts of the input.
//!
//! Key Concepts:
//! - Attention: Q @ K^T / sqrt(d) -> Softmax -> @ V
//! - Multi-Head: Multiple attention heads capture different relationships
//! - Positional Encoding: Since attention is permutation invariant

use rand::prelude::*;
use serde_json::json;
use std::collections::HashMap;

/// Token embedding and vocabulary
struct Tokenizer {
    vocab: HashMap<String, usize>,
    inverse_vocab: Vec<String>,
}

impl Tokenizer {
    fn new() -> Self {
        // Simple character-level tokenizer
        let chars = "abcdefghijklmnopqrstuvwxyz .!?,<>".chars();
        let mut vocab = HashMap::new();
        let mut inverse_vocab = Vec::new();
        
        vocab.insert("<PAD>".to_string(), 0);
        inverse_vocab.push("<PAD>".to_string());
        vocab.insert("<SOS>".to_string(), 1);
        inverse_vocab.push("<SOS>".to_string());
        vocab.insert("<EOS>".to_string(), 2);
        inverse_vocab.push("<EOS>".to_string());
        
        for (i, c) in chars.enumerate() {
            vocab.insert(c.to_string(), i + 3);
            inverse_vocab.push(c.to_string());
        }
        
        Self { vocab, inverse_vocab }
    }
    
    fn encode(&self, text: &str) -> Vec<usize> {
        let mut tokens = vec![1]; // SOS
        for c in text.chars() {
            if let Some(&id) = self.vocab.get(&c.to_string()) {
                tokens.push(id);
            }
        }
        tokens.push(2); // EOS
        tokens
    }
    
    fn decode(&self, tokens: &[usize]) -> String {
        tokens.iter()
            .filter_map(|&t| {
                if t > 2 && t < self.inverse_vocab.len() {
                    Some(self.inverse_vocab[t].clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    fn vocab_size(&self) -> usize {
        self.inverse_vocab.len()
    }
}

/// Scaled Dot-Product Attention
fn attention(q: &[Vec<f64>], k: &[Vec<f64>], v: &[Vec<f64>]) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let seq_len = q.len();
    let d_k = q[0].len() as f64;
    
    // Q @ K^T
    let mut scores = vec![vec![0.0; seq_len]; seq_len];
    for i in 0..seq_len {
        for j in 0..seq_len {
            let mut dot = 0.0;
            for k_idx in 0..q[0].len() {
                dot += q[i][k_idx] * k[j][k_idx];
            }
            scores[i][j] = dot / d_k.sqrt();
        }
    }
    
    // Causal mask (for decoder)
    for i in 0..seq_len {
        for j in (i + 1)..seq_len {
            scores[i][j] = f64::NEG_INFINITY;
        }
    }
    
    // Softmax
    let mut attention_weights = vec![vec![0.0; seq_len]; seq_len];
    for i in 0..seq_len {
        let max = scores[i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp: Vec<f64> = scores[i].iter().map(|&x| (x - max).exp()).collect();
        let sum: f64 = exp.iter().sum();
        for j in 0..seq_len {
            attention_weights[i][j] = exp[j] / sum;
        }
    }
    
    // Attention @ V
    let d_v = v[0].len();
    let mut output = vec![vec![0.0; d_v]; seq_len];
    for i in 0..seq_len {
        for j in 0..seq_len {
            for d in 0..d_v {
                output[i][d] += attention_weights[i][j] * v[j][d];
            }
        }
    }
    
    (output, attention_weights)
}

/// Positional Encoding (sinusoidal)
fn positional_encoding(seq_len: usize, d_model: usize) -> Vec<Vec<f64>> {
    let mut pe = vec![vec![0.0; d_model]; seq_len];
    
    for pos in 0..seq_len {
        for i in 0..(d_model / 2) {
            let angle = pos as f64 / (10000.0_f64).powf(2.0 * i as f64 / d_model as f64);
            pe[pos][2 * i] = angle.sin();
            pe[pos][2 * i + 1] = angle.cos();
        }
    }
    
    pe
}

/// Simple Feed-Forward Network
fn feed_forward(x: &[f64], w1: &[Vec<f64>], b1: &[f64], w2: &[Vec<f64>], b2: &[f64]) -> Vec<f64> {
    let hidden_size = w1[0].len();
    let mut hidden = vec![0.0; hidden_size];
    
    // Linear 1 + ReLU
    for h in 0..hidden_size {
        let mut sum = b1[h];
        for (i, &x_val) in x.iter().enumerate() {
            sum += w1[i][h] * x_val;
        }
        hidden[h] = if sum > 0.0 { sum } else { 0.0 }; // ReLU
    }
    
    // Linear 2
    let output_size = w2[0].len();
    let mut output = vec![0.0; output_size];
    for o in 0..output_size {
        let mut sum = b2[o];
        for (h, &hid) in hidden.iter().enumerate() {
            sum += w2[h][o] * hid;
        }
        output[o] = sum;
    }
    
    output
}

/// Mini Transformer (decoder-only)
struct MiniTransformer {
    vocab_size: usize,
    d_model: usize,
    n_heads: usize,
    
    // Embedding
    embedding: Vec<Vec<f64>>,
    
    // Attention weights (simplified: one head)
    w_q: Vec<Vec<f64>>,
    w_k: Vec<Vec<f64>>,
    w_v: Vec<Vec<f64>>,
    w_o: Vec<Vec<f64>>,
    
    // FFN
    ffn_w1: Vec<Vec<f64>>,
    ffn_b1: Vec<f64>,
    ffn_w2: Vec<Vec<f64>>,
    ffn_b2: Vec<f64>,
    
    // Output projection
    output_proj: Vec<Vec<f64>>,
}

impl MiniTransformer {
    fn new(vocab_size: usize) -> Self {
        let mut rng = rand::rng();
        let d_model = 32;
        let d_ff = 64;
        
        let mut rand_matrix = |rows: usize, cols: usize| -> Vec<Vec<f64>> {
            (0..rows).map(|_| (0..cols).map(|_| rng.random_range(-0.1..0.1)).collect()).collect()
        };
        
        Self {
            vocab_size,
            d_model,
            n_heads: 1,
            embedding: rand_matrix(vocab_size, d_model),
            w_q: rand_matrix(d_model, d_model),
            w_k: rand_matrix(d_model, d_model),
            w_v: rand_matrix(d_model, d_model),
            w_o: rand_matrix(d_model, d_model),
            ffn_w1: rand_matrix(d_model, d_ff),
            ffn_b1: vec![0.0; d_ff],
            ffn_w2: rand_matrix(d_ff, d_model),
            ffn_b2: vec![0.0; d_model],
            output_proj: rand_matrix(d_model, vocab_size),
        }
    }
    
    fn embed(&self, tokens: &[usize]) -> Vec<Vec<f64>> {
        let pe = positional_encoding(tokens.len(), self.d_model);
        
        tokens.iter().enumerate().map(|(pos, &tok)| {
            self.embedding[tok].iter()
                .zip(pe[pos].iter())
                .map(|(&e, &p)| e + p)
                .collect()
        }).collect()
    }
    
    fn linear(&self, x: &[f64], w: &[Vec<f64>]) -> Vec<f64> {
        let out_size = w[0].len();
        let mut out = vec![0.0; out_size];
        for o in 0..out_size {
            for (i, &x_val) in x.iter().enumerate() {
                out[o] += x_val * w[i][o];
            }
        }
        out
    }
    
    fn forward(&self, tokens: &[usize]) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let x = self.embed(tokens);
        
        // Compute Q, K, V
        let q: Vec<Vec<f64>> = x.iter().map(|xi| self.linear(xi, &self.w_q)).collect();
        let k: Vec<Vec<f64>> = x.iter().map(|xi| self.linear(xi, &self.w_k)).collect();
        let v: Vec<Vec<f64>> = x.iter().map(|xi| self.linear(xi, &self.w_v)).collect();
        
        // Attention
        let (attn_out, attn_weights) = attention(&q, &k, &v);
        
        // Output projection + residual
        let attn_projected: Vec<Vec<f64>> = attn_out.iter().map(|a| self.linear(a, &self.w_o)).collect();
        let residual1: Vec<Vec<f64>> = x.iter().zip(attn_projected.iter())
            .map(|(xi, ai)| xi.iter().zip(ai.iter()).map(|(&x, &a)| x + a).collect())
            .collect();
        
        // FFN + residual
        let ffn_out: Vec<Vec<f64>> = residual1.iter()
            .map(|r| feed_forward(r, &self.ffn_w1, &self.ffn_b1, &self.ffn_w2, &self.ffn_b2))
            .collect();
        let residual2: Vec<Vec<f64>> = residual1.iter().zip(ffn_out.iter())
            .map(|(ri, fi)| ri.iter().zip(fi.iter()).map(|(&r, &f)| r + f).collect())
            .collect();
        
        // Output logits
        let logits: Vec<Vec<f64>> = residual2.iter()
            .map(|r| self.linear(r, &self.output_proj))
            .collect();
        
        (logits, attn_weights)
    }
    
    fn generate(&self, prompt: &[usize], max_len: usize, temperature: f64, rng: &mut impl Rng) -> Vec<usize> {
        let mut tokens = prompt.to_vec();
        
        for _ in 0..max_len {
            let (logits, _) = self.forward(&tokens);
            let last_logits = &logits[logits.len() - 1];
            
            // Temperature sampling
            let scaled: Vec<f64> = last_logits.iter().map(|&l| l / temperature).collect();
            let max_l = scaled.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let exp: Vec<f64> = scaled.iter().map(|&x| (x - max_l).exp()).collect();
            let sum: f64 = exp.iter().sum();
            let probs: Vec<f64> = exp.iter().map(|&e| e / sum).collect();
            
            // Sample
            let r: f64 = rng.random();
            let mut cumsum = 0.0;
            let mut next_token = 0;
            for (i, &p) in probs.iter().enumerate() {
                cumsum += p;
                if r < cumsum {
                    next_token = i;
                    break;
                }
            }
            
            tokens.push(next_token);
            
            // Stop at EOS
            if next_token == 2 {
                break;
            }
        }
        
        tokens
    }
}

pub fn run() {
    println!("--- Lesson 10: Large Language Models (Transformers) ---");
    
    let mut rng = rand::rng();
    let tokenizer = Tokenizer::new();
    let transformer = MiniTransformer::new(tokenizer.vocab_size());
    
    println!("Vocab size: {}", tokenizer.vocab_size());
    println!("Model dimension: {}\n", transformer.d_model);
    
    // Demo attention visualization
    println!("--- Attention Mechanism Demo ---");
    let text = "hello world";
    let tokens = tokenizer.encode(text);
    println!("Input: \"{}\"", text);
    println!("Tokens: {:?}", tokens);
    
    let (logits, attention_weights) = transformer.forward(&tokens);
    
    println!("\nAttention Pattern (which positions attend to which):");
    let token_strs: Vec<String> = tokens.iter()
        .map(|&t| tokenizer.inverse_vocab.get(t).cloned().unwrap_or("?".to_string()))
        .collect();
    
    print!("      ");
    for s in &token_strs {
        print!("{:>6}", s);
    }
    println!();
    
    for (i, row) in attention_weights.iter().enumerate() {
        print!("{:>5} ", token_strs[i]);
        for &w in row {
            if w > 0.001 {
                print!("{:>6.2}", w);
            } else {
                print!("{:>6}", "-");
            }
        }
        println!();
    }
    
    // Generation demo
    println!("\n--- Text Generation Demo ---");
    let prompts = ["the ", "hello ", "ai "];
    
    for prompt in prompts {
        print!("Prompt: \"{}\" -> ", prompt);
        let prompt_tokens = tokenizer.encode(prompt);
        let generated = transformer.generate(&prompt_tokens, 20, 1.0, &mut rng);
        let output = tokenizer.decode(&generated);
        println!("\"{}\"", output);
    }
    
    // Explain key concepts
    println!("\n--- Key Concepts ---");
    println!("1. Self-Attention: Each token attends to all previous tokens");
    println!("2. Causal Mask: Future tokens are masked (can't peek ahead)");
    println!("3. Positional Encoding: Adds position information to embeddings");
    println!("4. Residual Connections: Help gradient flow in deep networks");
    
    // Generate visualization
    let mut viz_data = Vec::new();
    
    // Attention heatmap
    for (i, row) in attention_weights.iter().enumerate() {
        for (j, &weight) in row.iter().enumerate() {
            viz_data.push(json!({
                "from": token_strs[i],
                "to": token_strs[j],
                "from_pos": i,
                "to_pos": j,
                "weight": weight,
                "type": "attention"
            }));
        }
    }
    
    // Positional encoding visualization
    let pe = positional_encoding(20, 32);
    for (pos, row) in pe.iter().enumerate() {
        for (dim, &val) in row.iter().take(8).enumerate() {
            viz_data.push(json!({
                "position": pos,
                "dimension": dim,
                "value": val,
                "type": "positional"
            }));
        }
    }
    
    let spec = json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "description": "Transformer Attention Visualization",
        "vconcat": [
            {
                "title": "Self-Attention Weights (\"hello world\")",
                "width": 400,
                "height": 400,
                "data": { "values": viz_data },
                "transform": [{ "filter": "datum.type == 'attention'" }],
                "mark": "rect",
                "encoding": {
                    "x": { 
                        "field": "to_pos", 
                        "type": "ordinal", 
                        "title": "Key Position (attends to)",
                        "axis": { "labelExpr": "datum.value" }
                    },
                    "y": { 
                        "field": "from_pos", 
                        "type": "ordinal", 
                        "title": "Query Position",
                        "sort": "descending"
                    },
                    "color": {
                        "field": "weight",
                        "type": "quantitative",
                        "scale": { "scheme": "blues" },
                        "legend": { "title": "Attention" }
                    },
                    "tooltip": [
                        { "field": "from", "title": "From" },
                        { "field": "to", "title": "To" },
                        { "field": "weight", "format": ".3f" }
                    ]
                }
            },
            {
                "title": "Positional Encoding (first 8 dimensions)",
                "width": 400,
                "height": 200,
                "data": { "values": viz_data },
                "transform": [{ "filter": "datum.type == 'positional'" }],
                "mark": "rect",
                "encoding": {
                    "x": { "field": "position", "type": "ordinal", "title": "Position" },
                    "y": { "field": "dimension", "type": "ordinal", "title": "Dimension" },
                    "color": {
                        "field": "value",
                        "type": "quantitative",
                        "scale": { "scheme": "redblue", "domain": [-1, 1] }
                    }
                }
            }
        ]
    });
    
    let filename = "lesson_10.json";
    std::fs::write(filename, spec.to_string()).unwrap();
    println!("\nVisualization saved to: {}", filename);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let tok = Tokenizer::new();
        
        let text = "hello";
        let encoded = tok.encode(text);
        let decoded = tok.decode(&encoded);
        
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_attention_sum_to_one() {
        let q = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let k = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let v = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        
        let (_, weights) = attention(&q, &k, &v);
        
        for row in &weights {
            let sum: f64 = row.iter().sum();
            assert!((sum - 1.0).abs() < 1e-6, "Attention weights should sum to 1");
        }
    }

    #[test]
    fn test_positional_encoding() {
        let pe = positional_encoding(10, 8);
        
        assert_eq!(pe.len(), 10);
        assert_eq!(pe[0].len(), 8);
        
        // Values should be in [-1, 1]
        for row in &pe {
            for &val in row {
                assert!(val >= -1.0 && val <= 1.0);
            }
        }
    }

    #[test]
    fn test_transformer_forward() {
        let tok = Tokenizer::new();
        let transformer = MiniTransformer::new(tok.vocab_size());
        
        let tokens = tok.encode("hi");
        let (logits, _) = transformer.forward(&tokens);
        
        assert_eq!(logits.len(), tokens.len());
        assert_eq!(logits[0].len(), tok.vocab_size());
    }
}

