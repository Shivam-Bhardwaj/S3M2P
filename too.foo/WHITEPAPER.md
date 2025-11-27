# TOO.FOO: The Infinite Resolution Engine

## Abstract
Too.foo is a research-grade platform dedicated to the next generation of internet architecture: **Data-Driven Rendering**. We believe the future of the web lies not in static assets or client-side processing, but in the seamless streaming of massive, server-processed datasets that are rendered instantly on the client.

## Mission
To build the infrastructure for "Infinite Resolution" simulations. We aim to decouple simulation complexity from rendering latency. Whether it's a galaxy-scale simulation or a molecular dynamics model, the user should experience it in real-time, limited only by bandwidth and display resolution, not by local compute power.

## Architecture: The Spatial Streaming Protocol

### 1. Server-Side Heavy Lifting
Instead of running physics or LLM inference on the user's device, we utilize powerful GPU clusters in data centers.
*   **Simulation**: Rust-based physics engines (Antimony Core) run continuous simulations (N-body, Fluid Dynamics, Plasma).
*   **Spatial Indexing**: Data is dynamically indexed into a **Cube Sphere Quadtree** (Spherical Coordinate System).
*   **Storage**: Processed chunks are stored in a high-performance binary format, indexed by `(Face, Level, X, Y)`.

### 2. The Retrieval Engine (Antimony Core)
The core library (`antimony-core`) implements a sophisticated caching and retrieval logic:
*   **LOD (Level of Detail)**: The client requests data based on its camera position and Field of View (FOV).
*   **Frustum Culling**: Only chunks visible to the user are fetched.
*   **Predictive Prefetching**: The engine anticipates user movement to stream data before it's needed.

### 3. Client-Side Rendering (Helios)
The frontend (`helios.too.foo`) is a lightweight WASM + WebGPU application.
*   **Zero-Copy Rendering**: Binary data from the server is mapped directly to GPU buffers.
*   **Compute Shaders**: Final visual effects (bloom, trails) are applied locally.

## Projects

### HELIOS (helios.too.foo)
A sun-centric simulation of the heliosphere.
*   **Scale**: From the solar surface to the termination shock (100+ AU).
*   **Data**: Real-time streaming of solar wind plasma density, magnetic fields, and stellar positions.
*   **Tech**: Rust, WebGPU, Antimony Spatial Store.

### CAM
[Coming Soon] A camera-first interface for exploring latent spaces in generative models.

### ML
[Coming Soon] Machine learning integration for predictive simulation steering.

## Technology Stack
*   **Language**: Rust (Server & Client via WASM)
*   **Graphics**: WebGPU (wgpu)
*   **Network**: HTTP/3, WebSocket, Custom Binary Protocol
*   **Compute**: CUDA / WGPU Compute Shaders

---
*Built by Curious. Powered by Rust.*

