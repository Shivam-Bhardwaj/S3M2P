# Deployment Guide

All frontend projects: **Rust + WASM + Trunk → Cloudflare Pages**

## Quick Deploy

```bash
# Build all
./scripts/deploy.sh all

# Build + publish one project
./scripts/deploy.sh blog --publish

# Build + publish everything
./scripts/deploy.sh all --publish
```

Requires: `npm install -g wrangler && wrangler login`

## Projects

| Project | Subdomain | Root Dir | Port (dev) |
|---------|-----------|----------|------------|
| too.foo | too.foo | `too.foo` | 8080 |
| helios | helios.too.foo | `helios` | 8081 |
| chladni | chladni.too.foo | `chladni` | 8082 |
| blog | blog.too.foo | `blog` | 8083 |
| autocrate | autocrate.too.foo | `autocrate` | 8084 |
| portfolio | portfolio.too.foo | `portfolio` | 8085 |
| ML | learn.too.foo | `ML` | 3000 (native) |

## Cloudflare Pages Setup (per project)

1. **Dashboard** → Pages → Create Project → Connect Git
2. **Build Settings**:
   - Framework: `None`
   - Build command: `trunk build --release`
   - Output directory: `dist`
   - Root directory: `{project_folder}`
3. **Environment Variables**:
   ```
   WASM_BINDGEN_VERSION=0.2.93
   ```

## DNS (Cloudflare)

Add CNAME records:
```
helios    → {pages-project}.pages.dev
chladni   → {pages-project}.pages.dev
blog      → {pages-project}.pages.dev
autocrate → {pages-project}.pages.dev
portfolio → {pages-project}.pages.dev
learn     → {vps-ip or tunnel}
```

## Local Dev

```bash
# Run any project locally
trunk serve {project}/index.html --open

# Examples
trunk serve too.foo/index.html --open
trunk serve chladni/index.html --open
trunk serve blog/index.html --open
```

## ML (learn.too.foo)

Native Rust server, not WASM. Deploy to VPS:

```bash
cd ML
cargo build --release
# Copy binary to VPS
scp target/release/antimony-labs user@vps:/opt/
# Run with systemd or supervisor
```

Or use Cloudflare Tunnel:
```bash
cloudflared tunnel --url http://localhost:3000
```

## Backend (Storage Server)

Deployed at `144.126.145.3` for Helios data.

For HTTPS, use Cloudflare Tunnel or Proxy:
```bash
cloudflared tunnel --url http://localhost:3000
# Maps to data.too.foo
```
