# Learn - ML Fundamentals Platform

Interactive ML learning platform built with Rust/WASM. Client-side rendering, no server needed.

## Build & Run

```bash
trunk serve learn/index.html --open
trunk build --release learn/index.html
```

## Architecture

```
learn/
  src/
    lib.rs       # App state, WASM entry
    lessons.rs   # Lesson definitions (12 lessons)
    render.rs    # DOM rendering
  index.html     # Entry point with KaTeX
```

## Curriculum

### Phase 1: Foundations (0-3)
- Rust Refresher
- Linear Regression
- Logistic Regression
- Neural Networks

### Phase 2: Deep Learning (4-5)
- CNNs
- Policy Networks

### Phase 3: Reinforcement Learning (6-8)
- Q-Learning
- Policy Gradients
- MCTS

### Phase 4: Advanced (9-11)
- AlphaZero
- LLMs
- System Architecture

## Features

- **Static site** - No server, deploys to Cloudflare Pages
- **KaTeX** - Math rendering for equations
- **Interactive demos** - Canvas-based visualizations (per lesson)
- **Responsive** - Works on mobile

## Implementation Status

- [x] Lesson structure and content
- [x] Home page with phase sections
- [x] Individual lesson view
- [x] KaTeX math rendering
- [ ] Interactive visualizations per lesson
- [ ] Progress tracking (localStorage)
- [ ] Code examples with syntax highlighting

## Content Source

Adapted from ML/ folder (antimony-labs). Original has:
- Full implementations in Rust
- Axum web server
- Vega-Lite visualizations

This WASM version focuses on teaching concepts client-side.
