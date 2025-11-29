// DOM rendering for blog
// Generates HTML elements from blog data

use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement};

use crate::{BlogIndex, Post, PostMeta};

pub struct BlogRenderer {
    document: Document,
    root: Element,
}

impl BlogRenderer {
    pub fn new(root_id: &str) -> Result<Self, JsValue> {
        let document = web_sys::window()
            .ok_or("No window")?
            .document()
            .ok_or("No document")?;

        let root = document
            .get_element_by_id(root_id)
            .ok_or("Root element not found")?;

        Ok(Self { document, root })
    }

    /// Clear the root element
    pub fn clear(&self) {
        self.root.set_inner_html("");
    }

    /// Render the home page with recent posts
    pub fn render_home(&self, index: &BlogIndex) -> Result<(), JsValue> {
        self.clear();

        let container = self.create_element("div", "blog-home")?;

        // Header
        let header = self.create_element("header", "blog-header")?;
        header.set_inner_html(
            r#"
            <h1>Blog</h1>
            <p class="subtitle">Thoughts on code, robotics, and the universe</p>
        "#,
        );
        container.append_child(&header)?;

        // Recent posts
        let posts_section = self.create_element("section", "recent-posts")?;
        let posts_html: String = index
            .recent(10)
            .iter()
            .map(|meta| self.render_post_card(meta))
            .collect();
        posts_section.set_inner_html(&format!("<h2>Recent Posts</h2>{}", posts_html));
        container.append_child(&posts_section)?;

        // Tags sidebar
        let tags_section = self.create_element("aside", "tags-sidebar")?;
        let tags: String = index
            .all_tags()
            .iter()
            .map(|tag| format!(r#"<a href="/tag/{}" class="tag">#{}</a>"#, tag, tag))
            .collect::<Vec<_>>()
            .join(" ");
        tags_section.set_inner_html(&format!(
            "<h3>Tags</h3><div class='tag-cloud'>{}</div>",
            tags
        ));
        container.append_child(&tags_section)?;

        self.root.append_child(&container)?;
        Ok(())
    }

    /// Render a single post
    pub fn render_post(&self, post: &Post) -> Result<(), JsValue> {
        self.clear();

        let article = self.create_element("article", "blog-post")?;

        let meta = &post.meta;
        let content_html = post.render_html();
        let reading_time = post.reading_time();

        let tags_html: String = meta
            .tags
            .iter()
            .map(|tag| format!(r#"<a href="/tag/{}" class="tag">#{}</a>"#, tag, tag))
            .collect::<Vec<_>>()
            .join(" ");

        let ai_badge = if meta.ai_generated {
            r#"<span class="ai-badge">AI Generated</span>"#
        } else {
            ""
        };

        article.set_inner_html(&format!(
            r#"
            <header class="post-header">
                <h1>{}</h1>
                <div class="post-meta">
                    <time>{}</time>
                    <span class="reading-time">{} min read</span>
                    {}
                </div>
                <div class="post-tags">{}</div>
            </header>
            <div class="post-content">
                {}
            </div>
            <footer class="post-footer">
                <a href="/" class="back-link">← Back to Blog</a>
            </footer>
        "#,
            meta.title, meta.date, reading_time, ai_badge, tags_html, content_html
        ));

        self.root.append_child(&article)?;
        Ok(())
    }

    /// Render a post card for listings
    fn render_post_card(&self, meta: &PostMeta) -> String {
        let tags: String = meta
            .tags
            .iter()
            .take(3)
            .map(|t| format!(r#"<span class="card-tag">{}</span>"#, t))
            .collect::<Vec<_>>()
            .join("");

        format!(
            r#"
            <article class="post-card">
                <a href="/post/{}">
                    <h3>{}</h3>
                    <time>{}</time>
                    <p>{}</p>
                    <div class="card-tags">{}</div>
                </a>
            </article>
        "#,
            meta.slug, meta.title, meta.date, meta.summary, tags
        )
    }

    /// Render posts filtered by tag
    pub fn render_tag(&self, tag: &str, index: &BlogIndex) -> Result<(), JsValue> {
        self.clear();

        let container = self.create_element("div", "blog-tag-page")?;

        let posts = index.by_tag(tag);
        let posts_html: String = posts
            .iter()
            .map(|meta| self.render_post_card(meta))
            .collect();

        container.set_inner_html(&format!(
            r#"
            <header class="tag-header">
                <h1>#{}</h1>
                <p>{} posts</p>
                <a href="/" class="back-link">← All Posts</a>
            </header>
            <section class="tag-posts">
                {}
            </section>
        "#,
            tag,
            posts.len(),
            posts_html
        ));

        self.root.append_child(&container)?;
        Ok(())
    }

    /// Render 404 page
    pub fn render_404(&self) -> Result<(), JsValue> {
        self.clear();
        let container = self.create_element("div", "not-found")?;
        container.set_inner_html(
            r#"
            <h1>404</h1>
            <p>Post not found</p>
            <a href="/">← Back to Blog</a>
        "#,
        );
        self.root.append_child(&container)?;
        Ok(())
    }

    fn create_element(&self, tag: &str, class: &str) -> Result<Element, JsValue> {
        let el = self.document.create_element(tag)?;
        el.set_class_name(class);
        Ok(el)
    }
}
