// Client-side router for blog navigation
// Handles URL changes without full page reloads

use wasm_bindgen::prelude::*;
use web_sys::{History, Window};

#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Home,
    Post(String), // /post/{slug}
    Tag(String),  // /tag/{tag}
    Archive,      // /archive
    About,        // /about
    NotFound,
}

impl Route {
    /// Parse route from URL path
    pub fn from_path(path: &str) -> Self {
        let path = path.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        match parts.as_slice() {
            [""] | [] => Route::Home,
            ["post", slug] => Route::Post(slug.to_string()),
            ["tag", tag] => Route::Tag(tag.to_string()),
            ["archive"] => Route::Archive,
            ["about"] => Route::About,
            _ => Route::NotFound,
        }
    }

    /// Convert route to URL path
    pub fn to_path(&self) -> String {
        match self {
            Route::Home => "/".to_string(),
            Route::Post(slug) => format!("/post/{}", slug),
            Route::Tag(tag) => format!("/tag/{}", tag),
            Route::Archive => "/archive".to_string(),
            Route::About => "/about".to_string(),
            Route::NotFound => "/404".to_string(),
        }
    }
}

pub struct Router {
    window: Window,
    history: History,
    current: Route,
}

impl Router {
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let history = window.history()?;
        let location = window.location();
        let path = location.pathname()?;
        let current = Route::from_path(&path);

        Ok(Self {
            window,
            history,
            current,
        })
    }

    /// Navigate to a new route
    pub fn navigate(&mut self, route: Route) -> Result<(), JsValue> {
        let path = route.to_path();
        self.history
            .push_state_with_url(&JsValue::NULL, "", Some(&path))?;
        self.current = route;
        Ok(())
    }

    /// Get current route
    pub fn current(&self) -> &Route {
        &self.current
    }

    /// Handle browser back/forward
    pub fn sync_from_url(&mut self) -> Result<Route, JsValue> {
        let path = self.window.location().pathname()?;
        self.current = Route::from_path(&path);
        Ok(self.current.clone())
    }
}
