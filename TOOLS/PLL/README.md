# PLL Designer Tool

A WebAssembly-based interactive Phase Locked Loop (PLL) design tool.

## Overview
This tool provides a web interface for designing PLL frequency synthesizers. It uses the core logic from the `DNA` crate (`dna::pll`) and renders schematics, Bode plots, and phase noise profiles using HTML5 Canvas.

## Features
- **Integer-N & Fractional-N** Architecture selection.
- **Interactive Tuning**: real-time sliders for bandwidth, phase margin, and frequencies.
- **Visualizations**: 
    - Block Diagram / Schematic.
    - Open Loop Bode Plot.
    - Phase Noise Profile.
    - Transient Step Response.

## Usage
This crate compiles to WebAssembly.
```bash
wasm-pack build --target web
```
The resulting WASM is loaded by the web frontend.

## Dependencies
- `dna`: Core algorithms.
- `wasm-bindgen`: JS interoperability.
- `web-sys`: DOM and Canvas manipulation.
