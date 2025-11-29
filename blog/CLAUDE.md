# Blog - Markdown Blog Engine

Rust/WASM blog engine with markdown support and AI-generated content indicators.

## Build & Run

```bash
trunk serve blog/index.html --open
trunk build --release blog/index.html
```

## Architecture

```
blog/
  src/
    lib.rs       # Core types, markdown parsing
    router.rs    # Client-side routing
    render.rs    # DOM rendering
  posts/         # Markdown blog posts
  index.html     # Entry point
```

## Writing Posts

Posts are markdown files in `posts/` with YAML frontmatter:

```markdown
---
title: "Post Title"
slug: "post-slug"
date: "2025-01-15"
tags: [rust, wasm, tutorial]
summary: "Brief description for listings"
draft: false
ai_generated: true
---

# Post Content

Your markdown content here...
```

### Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| title | Yes | Display title |
| slug | Yes | URL path (/post/{slug}) |
| date | Yes | Publication date (YYYY-MM-DD) |
| tags | No | Array of tags for filtering |
| summary | No | Short description for cards |
| draft | No | If true, not shown in listings |
| ai_generated | No | Shows AI badge if true |

## Routes

- `/` - Home page with recent posts
- `/post/{slug}` - Individual post
- `/tag/{tag}` - Posts filtered by tag
- `/archive` - All posts
- `/about` - About page

## Markdown Features

Supported via pulldown-cmark:
- Headers, lists, blockquotes
- Code blocks with syntax highlighting
- Tables
- Footnotes
- Task lists
- Strikethrough

## Implementation Status

- [x] Markdown parsing with frontmatter
- [x] Client-side router
- [x] Post rendering
- [x] Tag filtering
- [ ] Full post index loading
- [ ] Search functionality
- [ ] RSS feed generation
- [ ] Syntax highlighting (hljs integration)
- [ ] Comments (via external service)

## AI-Generated Content

Posts can be marked as AI-generated in frontmatter. These will:
- Display an "AI Generated" badge
- Be clearly distinguished in listings
- Follow same quality standards
