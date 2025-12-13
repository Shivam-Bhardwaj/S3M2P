//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: routing.rs | LEARN/learn_web/src/routing.rs
//! PURPOSE: Hash-based routing for LEARN apps
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_web
//! ═══════════════════════════════════════════════════════════════════════════════

use wasm_bindgen::prelude::*;

/// Route enum for LEARN apps
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// Home page - lesson list
    Home,
    /// Lesson view with lesson index
    Lesson(usize),
}

impl Route {
    /// Parse a hash string into a Route
    pub fn from_hash(hash: &str) -> Self {
        let hash = hash.trim_start_matches('#');

        if hash.is_empty() || hash == "/" {
            return Route::Home;
        }

        // Parse "#/lesson/N" format
        if let Some(rest) = hash.strip_prefix("/lesson/") {
            if let Ok(idx) = rest.parse::<usize>() {
                return Route::Lesson(idx);
            }
        }

        // Legacy format: "#N" (just a number)
        if let Ok(idx) = hash.parse::<usize>() {
            return Route::Lesson(idx);
        }

        Route::Home
    }

    /// Convert Route to hash string
    pub fn to_hash(self) -> String {
        match self {
            Route::Home => String::new(),
            Route::Lesson(idx) => format!("#/lesson/{}", idx),
        }
    }
}

/// Get the current route from the browser URL
pub fn get_current_route() -> Route {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return Route::Home,
    };

    let location = window.location();
    let hash = location.hash().unwrap_or_default();
    Route::from_hash(&hash)
}

/// Navigate to a route (updates browser URL)
pub fn navigate_to(route: Route) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    let location = window.location();
    let hash = route.to_hash();
    let _ = location.set_hash(&hash);
}

/// Navigate to home
pub fn navigate_home() {
    navigate_to(Route::Home);
}

/// Navigate to a specific lesson
pub fn navigate_to_lesson(idx: usize) {
    navigate_to(Route::Lesson(idx));
}

/// Setup hash change listener
///
/// The callback is called with the new route whenever the hash changes.
pub fn setup_routing<F>(mut on_route_change: F) -> Result<(), JsValue>
where
    F: FnMut(Route) + 'static,
{
    let window = web_sys::window().ok_or("No window")?;

    let closure = Closure::wrap(Box::new(move || {
        let route = get_current_route();
        on_route_change(route);
    }) as Box<dyn FnMut()>);

    window.add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

/// Setup routing with initial render
///
/// Calls the callback immediately with the current route, then on every hash change.
pub fn setup_routing_with_initial<F>(mut on_route_change: F) -> Result<(), JsValue>
where
    F: FnMut(Route) + Clone + 'static,
{
    // Initial render
    let route = get_current_route();
    on_route_change(route);

    // Setup listener for future changes
    setup_routing(on_route_change)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_from_hash() {
        assert_eq!(Route::from_hash(""), Route::Home);
        assert_eq!(Route::from_hash("#"), Route::Home);
        assert_eq!(Route::from_hash("#/"), Route::Home);
        assert_eq!(Route::from_hash("#/lesson/0"), Route::Lesson(0));
        assert_eq!(Route::from_hash("#/lesson/5"), Route::Lesson(5));
        assert_eq!(Route::from_hash("#5"), Route::Lesson(5));
    }

    #[test]
    fn test_route_to_hash() {
        assert_eq!(Route::Home.to_hash(), "");
        assert_eq!(Route::Lesson(0).to_hash(), "#/lesson/0");
        assert_eq!(Route::Lesson(10).to_hash(), "#/lesson/10");
    }
}
