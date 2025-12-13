//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lessons.rs | SLAM/src/lessons.rs
//! PURPOSE: SLAM lesson definitions and curriculum structure
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → SLAM
//! ═══════════════════════════════════════════════════════════════════════════════

/// A single SLAM lesson
pub struct Lesson {
    pub id: usize,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub intuition: &'static str,
    pub key_concepts: &'static [&'static str],
}

/// All SLAM lessons
pub static LESSONS: &[Lesson] = &[
    Lesson {
        id: 0,
        title: "Particle Filter",
        subtitle: "Monte Carlo Localization",
        icon: "🎯",
        description: "Estimate robot position using a cloud of weighted particles. Each particle represents a hypothesis about where the robot might be.",
        intuition: "Imagine dropping thousands of tiny robots across a map. As the real robot moves and senses, particles that match its observations survive while others fade away. The surviving particles converge on the true position.",
        key_concepts: &["Particles", "Weights", "Resampling", "Motion Model", "Sensor Model"],
    },
    Lesson {
        id: 1,
        title: "Kalman Filter",
        subtitle: "Optimal State Estimation",
        icon: "📊",
        description: "Combine noisy sensor measurements with motion predictions to optimally estimate state. The workhorse of robotics estimation.",
        intuition: "Like tracking a flying ball - you predict where it will be, observe where it appears, and blend both estimates weighted by their certainty.",
        key_concepts: &["Prediction", "Update", "Covariance", "Kalman Gain", "Gaussian"],
    },
    Lesson {
        id: 2,
        title: "EKF SLAM",
        subtitle: "Simultaneous Localization & Mapping",
        icon: "🗺️",
        description: "Build a map while simultaneously localizing within it. Solve the chicken-and-egg problem of needing a map to localize, but needing position to map.",
        intuition: "Explore a dark room with a flashlight. As you move, you build a mental map of obstacles while tracking your own position relative to them.",
        key_concepts: &["State Augmentation", "Landmark Association", "Loop Closure", "Uncertainty"],
    },
    Lesson {
        id: 3,
        title: "Graph SLAM",
        subtitle: "Pose Graph Optimization",
        icon: "🔗",
        description: "Represent SLAM as a graph optimization problem. Nodes are poses, edges are constraints from odometry and observations.",
        intuition: "Connect poses with rubber bands representing constraints. The optimal solution minimizes the total energy of stretched bands.",
        key_concepts: &["Pose Graph", "Constraints", "Optimization", "Bundle Adjustment"],
    },
];
