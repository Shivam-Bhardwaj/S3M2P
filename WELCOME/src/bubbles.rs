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
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub action: BubbleAction,
}

/// A category page containing sub-bubbles
pub struct Category {
    pub id: CategoryId,
    pub bubbles: &'static [Bubble],
}

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
        action: BubbleAction::DirectProject("https://helios.too.foo"),
    },
    // 2. X (Top Right) - External
    Bubble {
        id: "x",
        label: "X",
        description: "@LazyShivam",
        icon: "assets/islands/x.svg",
        action: BubbleAction::External("https://x.com/LazyShivam"),
    },
    // 3. Blog (Right) - Direct project (for now, could become category later)
    Bubble {
        id: "blog",
        label: "Blog",
        description: "Technical Writing",
        icon: "assets/islands/blog.svg",
        action: BubbleAction::DirectProject("https://blog.too.foo"),
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
    // 7. About Me (Left) - Direct project
    Bubble {
        id: "about",
        label: "About Me",
        description: "Coming Soon",
        icon: "assets/islands/about.svg",
        action: BubbleAction::DirectProject("https://portfolio.too.foo"),
    },
];

// ============================================
// CATEGORY: TOOLS
// ============================================

pub const TOOLS_BUBBLES: &[Bubble] = &[
    Bubble {
        id: "sensors",
        label: "Sensors",
        description: "Mobile Sensor Testing",
        icon: "assets/islands/sensors.svg",
        action: BubbleAction::DirectProject("https://sensors.too.foo"),
    },
    Bubble {
        id: "autocrate",
        label: "AutoCrate",
        description: "Shipping Crate Generator",
        icon: "assets/islands/automation.svg",
        action: BubbleAction::DirectProject("https://autocrate.too.foo"),
    },
    Bubble {
        id: "crm",
        label: "CRM",
        description: "Customer Relations",
        icon: "assets/islands/tools.svg",
        action: BubbleAction::DirectProject("https://crm.too.foo"),
    },
    Bubble {
        id: "pll",
        label: "PLL",
        description: "Phase Lock Loop Designer",
        icon: "assets/islands/tools.svg",
        action: BubbleAction::DirectProject("https://pll.too.foo"),
    },
    Bubble {
        id: "power",
        label: "Power",
        description: "Power Circuit Designer",
        icon: "assets/islands/tools.svg",
        action: BubbleAction::DirectProject("https://power.too.foo"),
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
        icon: "assets/islands/sims.svg",
        action: BubbleAction::DirectProject("https://chladni.too.foo"),
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
        icon: "assets/islands/learn.svg",
        action: BubbleAction::DirectProject("https://ai.too.foo"),
    },
    Bubble {
        id: "ubuntu",
        label: "Ubuntu",
        description: "Linux System Administration",
        icon: "assets/islands/learn.svg",
        action: BubbleAction::DirectProject("https://ubuntu.too.foo"),
    },
    Bubble {
        id: "opencv",
        label: "OpenCV",
        description: "Computer Vision",
        icon: "assets/islands/learn.svg",
        action: BubbleAction::DirectProject("https://opencv.too.foo"),
    },
    Bubble {
        id: "arduino",
        label: "Arduino",
        description: "Embedded Systems",
        icon: "assets/islands/learn.svg",
        action: BubbleAction::DirectProject("https://arduino.too.foo"),
    },
    Bubble {
        id: "esp32",
        label: "ESP32",
        description: "IoT Development",
        icon: "assets/islands/learn.svg",
        action: BubbleAction::DirectProject("https://esp32.too.foo"),
    },
    Bubble {
        id: "swarm",
        label: "Swarm",
        description: "Multi-Robot Coordination",
        icon: "assets/islands/learn.svg",
        action: BubbleAction::DirectProject("https://swarm.too.foo"),
    },
    Bubble {
        id: "slam",
        label: "SLAM",
        description: "Localization & Mapping",
        icon: "assets/islands/learn.svg",
        action: BubbleAction::DirectProject("https://slam.too.foo"),
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
