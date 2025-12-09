//! Hash-based routing for too.foo SPA navigation
//!
//! Routes are in the format `#/category` (e.g., `#/tools`, `#/sims`, `#/learn`)

use crate::bubbles::CategoryId;

/// Current route state
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Route {
    /// Home page - shows main bubbles
    Home,
    /// Category page - shows sub-bubbles for a category
    Category(CategoryId),
    /// Architecture diagram view
    Architecture,
}

impl Route {
    /// Parse the current URL hash into a Route
    pub fn from_hash(hash: &str) -> Self {
        match hash {
            "#/tools" => Route::Category(CategoryId::Tools),
            "#/sims" => Route::Category(CategoryId::Simulations),
            "#/learn" => Route::Category(CategoryId::Learn),
            "#/arch" => Route::Architecture,
            _ => Route::Home,
        }
    }

    /// Get the hash string for this route
    pub fn to_hash(&self) -> &'static str {
        match self {
            Route::Home => "",
            Route::Category(id) => id.hash_route(),
            Route::Architecture => "#/arch",
        }
    }
}

/// Get the current route from the browser URL
pub fn get_current_route() -> Route {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let hash = location.hash().unwrap_or_default();
    Route::from_hash(&hash)
}

/// Navigate to a route (updates browser URL)
pub fn navigate_to(route: Route) {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let hash = route.to_hash();
    location.set_hash(hash).ok();
}

/// Navigate back to home
pub fn navigate_home() {
    navigate_to(Route::Home);
}
