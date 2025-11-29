# Antimony Labs: Zero to AGI in Rust

A complete journey from basic ML concepts to AGI architecture, implemented entirely in Rust from scratch.

**You do not need to read the code.** The AI assistant handles implementation. Your job is to understand concepts through interactive visualizations.

## Quick Start

```bash
# Run all lessons and start the web server
cargo run

# Run tests (49 tests covering all components)
cargo test
```

Then open `http://localhost:3000` to explore the interactive dashboard.

## Course Structure

### Phase 1: Foundations (Lessons 0-3)
- [x] **Lesson 0: Rust Refresher** - Ownership & Borrowing visualization
- [x] **Lesson 1: Linear Regression** - Gradient descent fitting a line
- [x] **Lesson 2: Logistic Regression** - Classification with decision boundaries
- [x] **Lesson 3: Neural Networks** - XOR problem with autograd engine

### Phase 2: Deep Learning (Lessons 4-5)
- [x] **Lesson 4: CNNs** - Convolution, edge detection, feature maps
- [x] **Lesson 5: Policy Networks** - State → Action probability mapping

### Phase 3: Reinforcement Learning (Lessons 6-8)
- [x] **Lesson 6: Q-Learning** - Grid world with Q-table and ε-greedy
- [x] **Lesson 7: Policy Gradients** - Actor-Critic on CartPole
- [x] **Lesson 8: MCTS** - Monte Carlo Tree Search for Tic-Tac-Toe

### Phase 4: Towards AGI (Lessons 9-11)
- [x] **Lesson 9: AlphaZero** - Self-play with neural MCTS
- [x] **Lesson 10: LLMs** - Transformer architecture & attention
- [x] **Lesson 11: AGI Architecture** - Multimodal processing, memory, reasoning

## Key Features

- **From Scratch**: No ML libraries used - pure Rust implementations
- **Autograd Engine**: Custom automatic differentiation for backpropagation
- **Interactive Visualizations**: Vega-Lite powered charts for each lesson
- **Test-Driven**: 49 tests covering core ML algorithms
- **Three Levels**: Each lesson has Intuition, Math, and Code deep-dives

## Architecture

```
src/
├── engine/          # Autograd engine (Value, backward, gradients)
├── lessons/         # All 12 lesson implementations
├── utils/           # Visualization helpers
└── web/             # Web server (Axum) + frontend
```

## Technologies

- **Rust 2024 Edition** - Memory-safe systems programming
- **ndarray** - N-dimensional arrays for matrix operations
- **Axum** - Async web framework
- **Vega-Lite** - Declarative visualization grammar
- **KaTeX** - Math rendering for equations
