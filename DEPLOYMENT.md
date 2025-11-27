# Deployment Guide: The Infinite Resolution Stack

This project uses a hybrid architecture:
1.  **Frontend**: Rust + WASM (Helios) -> Deployed on **Cloudflare Pages**.
2.  **Backend**: Rust (Storage Server) -> Deployed on **High-Performance VPS**.

## 1. Backend (Storage Server)
The server is currently deployed at `144.126.145.3`.

### SSL/HTTPS Requirement
Since Cloudflare Pages serves the frontend via **HTTPS**, the browser will block requests to an **HTTP** backend (Mixed Content Error).
**You must secure the backend.**

#### Option A: Cloudflare Tunnel (Recommended)
1.  Install `cloudflared` on the VPS.
2.  Run: `cloudflared tunnel --url http://localhost:3000`
3.  This gives you a `https://....trycloudflare.com` URL (or map it to `data.too.foo`).
4.  Use this URL in the frontend build.

#### Option B: Cloudflare Proxy (DNS)
1.  Change the server to listen on Port 80 or 8080 (Cloudflare compatible ports).
2.  Add a DNS `A` record: `data.too.foo` -> `144.126.145.3` (Proxied / Orange Cloud).
3.  Cloudflare handles SSL.

## 2. Frontend (Helios)
The frontend is built using `Trunk`.

### Build for Production
Pass the secure backend URL during the build:

```bash
# Example
export BACKEND_URL="https://data.too.foo"
trunk build --release
```

### Deploy to Cloudflare Pages
1.  Connect your GitHub repo to Cloudflare Pages.
2.  **Build Settings**:
    *   **Framework**: None (or Rust)
    *   **Build Command**: `trunk build --release`
    *   **Output Directory**: `dist`
    *   **Root Directory**: `helios`
3.  **Environment Variables** (in Cloudflare Dashboard):
    *   `BACKEND_URL`: `https://data.too.foo` (or your secure backend URL)
    *   `WASM_BINDGEN_VERSION`: `0.2.93` (matches Cargo.toml)

## 3. Project Structure
*   `antimony-core`: Shared physics & spatial library.
*   `storage-server`: The high-performance binary data server (VPS).
*   `helios`: The WASM visualization client (Cloudflare Pages).
*   `too.foo`: The landing page (Cloudflare Pages).

