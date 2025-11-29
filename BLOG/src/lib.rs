// Blog - Markdown-based blog engine
// AI-assisted content, rendered in Rust/WASM
#![allow(unexpected_cfgs)]

use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub mod render;
pub mod router;

/// Blog post metadata (frontmatter)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostMeta {
    pub title: String,
    pub slug: String,
    pub date: String,
    pub tags: Vec<String>,
    pub summary: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub ai_generated: bool,
}

/// Full blog post with content
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub meta: PostMeta,
    pub content: String, // Raw markdown
}

impl Post {
    /// Render markdown content to HTML
    pub fn render_html(&self) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(&self.content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    }

    /// Extract reading time estimate
    pub fn reading_time(&self) -> u32 {
        let word_count = self.content.split_whitespace().count();
        ((word_count as f32 / 200.0).ceil() as u32).max(1) // 200 WPM average
    }
}

/// Blog index (list of all posts)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlogIndex {
    pub posts: Vec<PostMeta>,
}

impl BlogIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter posts by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&PostMeta> {
        self.posts
            .iter()
            .filter(|p| !p.draft && p.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Get recent posts
    pub fn recent(&self, count: usize) -> Vec<&PostMeta> {
        self.posts.iter().filter(|p| !p.draft).take(count).collect()
    }

    /// Get all unique tags
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .posts
            .iter()
            .flat_map(|p| p.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }
}

/// Blog state
#[derive(Default)]
pub struct Blog {
    pub index: BlogIndex,
    pub current_post: Option<Post>,
}

impl Blog {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Parse markdown frontmatter (YAML-style)
pub fn parse_frontmatter(content: &str) -> Option<(PostMeta, String)> {
    if !content.starts_with("---") {
        return None;
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }

    let frontmatter = parts[1].trim();
    let body = parts[2].trim();

    // Simple key-value parsing (for full YAML, use serde_yaml)
    let mut meta = PostMeta {
        title: String::new(),
        slug: String::new(),
        date: String::new(),
        tags: Vec::new(),
        summary: String::new(),
        draft: false,
        ai_generated: false,
    };

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            match key {
                "title" => meta.title = value.to_string(),
                "slug" => meta.slug = value.to_string(),
                "date" => meta.date = value.to_string(),
                "summary" => meta.summary = value.to_string(),
                "draft" => meta.draft = value == "true",
                "ai_generated" => meta.ai_generated = value == "true",
                "tags" => {
                    meta.tags = value
                        .trim_matches(|c| c == '[' || c == ']')
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .collect();
                }
                _ => {}
            }
        }
    }

    Some((meta, body.to_string()))
}

/// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Blog engine initialized".into());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: "Test Post"
slug: "test-post"
date: "2025-01-15"
tags: [rust, wasm]
summary: "A test post"
---

# Hello World

This is the content.
"#;

        let (meta, body) = parse_frontmatter(content).unwrap();
        assert_eq!(meta.title, "Test Post");
        assert_eq!(meta.slug, "test-post");
        assert!(body.contains("Hello World"));
    }

    #[test]
    fn test_render_html() {
        let post = Post {
            meta: PostMeta {
                title: "Test".into(),
                slug: "test".into(),
                date: "2025-01-15".into(),
                tags: vec![],
                summary: "".into(),
                draft: false,
                ai_generated: false,
            },
            content: "# Heading\n\nParagraph with **bold**.".into(),
        };

        let html = post.render_html();
        assert!(html.contains("<h1>Heading</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }
}
