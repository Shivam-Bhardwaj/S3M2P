//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: bubbles.rs | WELCOME/src/bubbles.rs
//! PURPOSE: Bubble navigation configuration with environment-specific URLs and category definitions
//! MODIFIED: 2025-12-09
//! LAYER: WELCOME (landing)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! Bubble configuration and category data structures
//!
//! This module defines the bubble navigation system for too.foo.
//! Bubbles can be external links, direct project links, or category pages.

/// Unique identifier for each category page
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CategoryId {
    Tools,
    Simulations,
    Learn,
}

impl CategoryId {
    /// Returns the hash route for this category (e.g., "#/tools")
    pub fn hash_route(&self) -> &'static str {
        match self {
            CategoryId::Tools => "#/tools",
            CategoryId::Simulations => "#/sims",
            CategoryId::Learn => "#/learn",
        }
    }

    /// Returns human-readable title
    #[allow(dead_code)]
    pub fn title(&self) -> &'static str {
        match self {
            CategoryId::Tools => "Tools",
            CategoryId::Simulations => "Simulations",
            CategoryId::Learn => "Learn",
        }
    }
}

/// What happens when a bubble is clicked
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BubbleAction {
    /// External link (opens in new tab) - e.g., X → x.com
    External(&'static str),
    /// Direct project link (same tab) - e.g., Helios → helios.too.foo
    DirectProject(&'static str),
    /// Navigate to a category page - e.g., Tools → /#/tools
    Category(CategoryId),
}

/// A single bubble in the navigation
#[derive(Clone, Copy, Debug)]
pub struct Bubble {
    #[allow(dead_code)]
    pub id: &'static str,
    pub label: &'static str,
    #[allow(dead_code)]
    pub description: &'static str,
    pub icon: &'static str,
    pub action: BubbleAction,
}

/// A category page containing sub-bubbles
pub struct Category {
    #[allow(dead_code)]
    pub id: CategoryId,
    pub bubbles: &'static [Bubble],
}

// ============================================
// ENVIRONMENT URLS
// ============================================

#[cfg(debug_assertions)]
mod urls {
    pub const HELIOS: &str = "http://localhost:8081";
    pub const CHLADNI: &str = "http://localhost:8082";
    pub const PLL: &str = "http://localhost:8090";
    pub const SENSORS: &str = "http://localhost:8083";
    pub const AUTOCRATE: &str = "http://localhost:8084";
    pub const CRM: &str = "http://localhost:8085"; // Assuming generic port from serve-all or placeholder
    pub const POWER: &str = "http://localhost:8091";

    // Learn Bubbles
    pub const AI: &str = "http://localhost:8100";
    pub const UBUNTU: &str = "http://localhost:8101";
    pub const OPENCV: &str = "http://localhost:8102";
    pub const ARDUINO: &str = "http://localhost:8103";
    pub const ESP32: &str = "http://localhost:8104";
    pub const SWARM: &str = "http://localhost:8105";
    pub const SLAM: &str = "http://localhost:8106";

    pub const BLOG: &str = "http://localhost:8085";
    #[allow(dead_code)]
    pub const GENERIC_404: &str = "http://localhost:8080/404";
}

#[cfg(not(debug_assertions))]
mod urls {
    pub const HELIOS: &str = "https://helios.too.foo";
    pub const CHLADNI: &str = "https://chladni.too.foo";
    pub const PLL: &str = "https://pll.too.foo";
    pub const SENSORS: &str = "https://sensors.too.foo";
    pub const AUTOCRATE: &str = "https://autocrate.too.foo";
    pub const CRM: &str = "https://crm.too.foo";
    pub const POWER: &str = "https://power.too.foo";

    // Learn Bubbles - Currently pointing to 404 in prod, but let's keep them explicit
    pub const AI: &str = "https://ai.too.foo"; // or 404.too.foo
    pub const UBUNTU: &str = "https://ubuntu.too.foo";
    pub const OPENCV: &str = "https://opencv.too.foo";
    pub const ARDUINO: &str = "https://arduino.too.foo";
    pub const ESP32: &str = "https://esp32.too.foo";
    pub const SWARM: &str = "https://swarm.too.foo";
    pub const SLAM: &str = "https://slam.too.foo";

    pub const BLOG: &str = "https://blog.too.foo";
    pub const GENERIC_404: &str = "https://404.too.foo";
}

use urls::*;

// ============================================
// HOME PAGE BUBBLES
// ============================================

pub const HOME_BUBBLES: &[Bubble] = &[
    // 1. Helios (Top) - Direct project
    Bubble {
        id: "helios",
        label: "Helios",
        description: "Solar System Visualization",
        icon: "assets/islands/helios.svg",
        action: BubbleAction::DirectProject(HELIOS),
    },
    // 2. X (Top Right) - External
    Bubble {
        id: "x",
        label: "X",
        description: "@LazyShivam",
        icon: "assets/islands/x.svg",
        action: BubbleAction::External("https://x.com/LazyShivam"),
    },
    // 3. Blog (Right) - Coming soon
    Bubble {
        id: "blog",
        label: "Blog",
        description: "Technical Writing",
        icon: "assets/islands/blog.svg",
        action: BubbleAction::DirectProject(BLOG), // was 404.too.foo
    },
    // 4. Learn (Bottom Right) - Category
    Bubble {
        id: "learn",
        label: "Learn",
        description: "Tutorials & Courses",
        icon: "assets/islands/learn.svg",
        action: BubbleAction::Category(CategoryId::Learn),
    },
    // 5. Simulations (Bottom) - Category
    Bubble {
        id: "sims",
        label: "Simulations",
        description: "Interactive Demos",
        icon: "assets/islands/sims.svg",
        action: BubbleAction::Category(CategoryId::Simulations),
    },
    // 6. Tools (Bottom Left) - Category
    Bubble {
        id: "tools",
        label: "Tools",
        description: "Engineering Apps",
        icon: "assets/islands/tools.svg",
        action: BubbleAction::Category(CategoryId::Tools),
    },
    // 7. About Me (Left) - LinkedIn profile
    Bubble {
        id: "about",
        label: "About Me",
        description: "LinkedIn Profile",
        icon: "assets/islands/about.svg",
        action: BubbleAction::External("https://www.linkedin.com/in/shivambdj/"),
    },
];

// ============================================
// CATEGORY: TOOLS
// ============================================

pub const TOOLS_BUBBLES: &[Bubble] = &[
    Bubble {
        id: "pll",
        label: "PLL",
        description: "Phase Lock Loop Designer",
        icon: "assets/islands/pll.svg",
        action: BubbleAction::DirectProject(PLL),
    },
    Bubble {
        id: "sensors",
        label: "Sensors",
        description: "Mobile Sensor Testing",
        icon: "assets/islands/sensors.svg",
        action: BubbleAction::DirectProject(SENSORS), // was 404
    },
    Bubble {
        id: "autocrate",
        label: "AutoCrate",
        description: "Shipping Crate Generator",
        icon: "assets/islands/automation.svg",
        action: BubbleAction::DirectProject(AUTOCRATE), // was 404
    },
    Bubble {
        id: "crm",
        label: "CRM",
        description: "Customer Relations",
        icon: "assets/islands/crm.svg",
        action: BubbleAction::DirectProject(CRM), // was 404
    },
    Bubble {
        id: "power",
        label: "Power",
        description: "Power Circuit Designer",
        icon: "assets/islands/power.svg",
        action: BubbleAction::DirectProject(POWER), // was 404
    },
];

pub const TOOLS_CATEGORY: Category = Category {
    id: CategoryId::Tools,
    bubbles: TOOLS_BUBBLES,
};

// ============================================
// CATEGORY: SIMULATIONS
// ============================================

pub const SIMS_BUBBLES: &[Bubble] = &[
    Bubble {
        id: "chladni",
        label: "Chladni",
        description: "Wave Pattern Simulation",
        icon: "assets/islands/chladni.svg",
        action: BubbleAction::DirectProject(CHLADNI),
    },
    // Future: Boids, etc.
];

pub const SIMS_CATEGORY: Category = Category {
    id: CategoryId::Simulations,
    bubbles: SIMS_BUBBLES,
};

// ============================================
// CATEGORY: LEARN
// ============================================

pub const LEARN_BUBBLES: &[Bubble] = &[
    Bubble {
        id: "ai",
        label: "AI",
        description: "Machine Learning & Neural Networks",
        icon: "assets/islands/ai.svg",
        action: BubbleAction::DirectProject(AI), // was 404
    },
    Bubble {
        id: "ubuntu",
        label: "Ubuntu",
        description: "Linux System Administration",
        icon: "assets/islands/ubuntu.svg",
        action: BubbleAction::DirectProject(UBUNTU), // was 404
    },
    Bubble {
        id: "opencv",
        label: "OpenCV",
        description: "Computer Vision",
        icon: "assets/islands/opencv.svg",
        action: BubbleAction::DirectProject(OPENCV), // was 404
    },
    Bubble {
        id: "arduino",
        label: "Arduino",
        description: "Embedded Systems",
        icon: "assets/islands/arduino.svg",
        action: BubbleAction::DirectProject(ARDUINO), // was 404
    },
    Bubble {
        id: "esp32",
        label: "ESP32",
        description: "IoT Development",
        icon: "assets/islands/esp32.svg",
        action: BubbleAction::DirectProject(ESP32), // was 404
    },
    Bubble {
        id: "swarm",
        label: "Swarm",
        description: "Multi-Robot Coordination",
        icon: "assets/islands/swarm.svg",
        action: BubbleAction::DirectProject(SWARM), // was 404
    },
    Bubble {
        id: "slam",
        label: "SLAM",
        description: "Localization & Mapping",
        icon: "assets/islands/slam.svg",
        action: BubbleAction::DirectProject(SLAM), // was 404
    },
];

pub const LEARN_CATEGORY: Category = Category {
    id: CategoryId::Learn,
    bubbles: LEARN_BUBBLES,
};

// ============================================
// CATEGORY LOOKUP
// ============================================

/// Get category configuration by ID
pub fn get_category(id: CategoryId) -> &'static Category {
    match id {
        CategoryId::Tools => &TOOLS_CATEGORY,
        CategoryId::Simulations => &SIMS_CATEGORY,
        CategoryId::Learn => &LEARN_CATEGORY,
    }
}
