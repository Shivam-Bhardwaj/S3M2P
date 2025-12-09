# AutoCrate

AutoCrate is a tool for generating ASTM-standard shipping crate designs.

## Overview
This tool is a Rust/WASM implementation of a crate design utility. It leverages core logic from the `DNA` crate (`dna::autocrate`) to calculate:
- Optimal lumber dimensions.
- Skid count and spacing.
- Weight limits and center of gravity.

## Features
- **ASTM Compliance**: Generates designs compliant with standard shipping specs.
- **Material Optimization**: Minimizes lumber waste.
- **Export**: Can export designs to PDF or Gerber (via `DNA`).

## Usage
The crate compiles to WebAssembly for use in a web frontend:
```bash
wasm-pack build --target web
```
