//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lessons.rs | SLAM/src/lessons.rs
//! PURPOSE: SLAM lesson definitions - structured from intuitive to advanced
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → SLAM
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! Curriculum designed for audience ranging from undergrads to professionals.
//! Each lesson starts with intuition and a demo, then builds to formal concepts.

/// Technical term that can have a popup explanation
#[derive(Clone)]
pub struct Term {
    pub word: &'static str,
    pub short: &'static str,  // One-line explanation
    pub detail: &'static str, // Full explanation for popup
}

/// Glossary of technical terms used across lessons
pub static GLOSSARY: &[Term] = &[
    Term {
        word: "sensor fusion",
        short: "Combining multiple sensors to get a better estimate",
        detail: "Each sensor has strengths and weaknesses. By combining them intelligently, \
                 we can get an estimate that's better than any single sensor alone. \
                 Like using both your eyes for depth perception.",
    },
    Term {
        word: "noise",
        short: "Random errors in sensor measurements",
        detail: "Real sensors aren't perfect. They give slightly different readings each time, \
                 even when measuring the same thing. This randomness is called noise. \
                 Think of static on a radio - the signal is there, but with interference.",
    },
    Term {
        word: "drift",
        short: "Error that accumulates over time",
        detail: "Some sensors have tiny errors that add up. If you integrate a gyroscope \
                 that's slightly off, after an hour you might think you've rotated 10° \
                 when you haven't moved at all. This accumulated error is drift.",
    },
    Term {
        word: "Gaussian",
        short: "Bell-curve shaped probability distribution",
        detail: "Also called 'normal distribution'. Most measurements cluster around the true \
                 value, with fewer measurements far away. The bell curve shape appears \
                 everywhere in nature - heights, test scores, measurement errors.",
    },
    Term {
        word: "covariance",
        short: "How much uncertainty we have",
        detail: "A number (or matrix) that describes how spread out our estimates are. \
                 High covariance = very uncertain, our guess could be way off. \
                 Low covariance = confident, we're pretty sure where it is.",
    },
    Term {
        word: "state",
        short: "Everything we want to know about the system",
        detail: "For a robot, the state might be: position (x, y), orientation (which way \
                 it's facing), and velocity (how fast it's moving). The filter's job is \
                 to estimate this state from noisy sensor data.",
    },
    Term {
        word: "particle",
        short: "One guess about what the state might be",
        detail: "Instead of tracking one estimate, we track hundreds of guesses (particles). \
                 Each particle is a hypothesis: 'maybe the robot is HERE'. Particles that \
                 match sensor readings survive; wrong guesses die off.",
    },
    Term {
        word: "landmark",
        short: "A recognizable feature in the environment",
        detail: "Something the robot can see and recognize - a door, a corner, a unique \
                 pattern. By measuring distances to known landmarks, the robot can \
                 figure out where it is (like navigating by stars).",
    },
    Term {
        word: "loop closure",
        short: "Recognizing you've returned to a place you've been before",
        detail: "When mapping, errors accumulate as you travel. But if you recognize \
                 'I've been here before!', you can correct all the accumulated drift. \
                 This 'closing the loop' snaps the whole map into consistency.",
    },
];

/// A single SLAM lesson
pub struct Lesson {
    pub id: usize,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub icon: &'static str,
    /// The hook - why should I care? (1-2 sentences)
    pub why_it_matters: &'static str,
    /// Intuitive explanation - no jargon (2-3 paragraphs)
    pub intuition: &'static str,
    /// What the demo shows
    pub demo_explanation: &'static str,
    /// Key takeaways (what should stick)
    pub key_takeaways: &'static [&'static str],
    /// For those who want to go deeper
    pub going_deeper: &'static str,
    /// Mathematical notation (optional, hidden by default)
    pub math_details: &'static str,
}

/// All SLAM lessons - ordered from simple intuition to complex algorithms
pub static LESSONS: &[Lesson] = &[
    // ═══════════════════════════════════════════════════════════════════════════
    // LESSON 0: Complementary Filter (Why Sensor Fusion?)
    // ═══════════════════════════════════════════════════════════════════════════
    Lesson {
        id: 0,
        title: "Complementary Filter",
        subtitle: "Your First Sensor Fusion",
        icon: "🔄",
        why_it_matters: "Every phone, drone, and robot uses this. It's the simplest way to \
                         combine two imperfect sensors into one reliable measurement.",
        intuition: "Imagine you're trying to measure the tilt of a balance board. You have two sensors:\n\n\
            The <strong>accelerometer</strong> measures gravity's direction. It always knows which way is 'down' - \
            great! But it's jittery. Every little vibration shows up as noise. If you're moving, \
            it can't tell gravity from acceleration.\n\n\
            The <strong>gyroscope</strong> measures how fast you're rotating. It's smooth and responds instantly \
            to motion - perfect! But it has a fatal flaw: drift. It slowly accumulates tiny errors. \
            After a few minutes, it might think you've tilted 30° when you haven't moved at all.\n\n\
            Here's the key insight: <strong>these weaknesses are opposites!</strong> The accelerometer is wrong \
            in the short term (jittery) but right in the long term (no drift). The gyroscope is \
            right in the short term (smooth) but wrong in the long term (drifts). \
            What if we trust the gyro for quick changes and the accelerometer to keep us anchored?",
        demo_explanation: "Watch the three signals:\n\n\
            • <strong>Red line</strong>: Raw accelerometer - see how jittery it is? That's noise.\n\
            • <strong>Blue line</strong>: Integrated gyroscope - smooth, but watch it slowly drift away from truth.\n\
            • <strong>Green line</strong>: Fused output - smooth AND accurate!\n\
            • <strong>Gray dashed</strong>: True angle (what we're trying to measure)\n\n\
            Adjust <strong>α (alpha)</strong> to control the blend:\n\
            • α close to 1: Trust gyro more → smoother but might drift\n\
            • α close to 0: Trust accel more → no drift but jittery\n\
            • Sweet spot (~0.96): Best of both worlds!",
        key_takeaways: &[
            "Sensors have complementary strengths - combine them!",
            "The blend parameter (α) controls the tradeoff",
            "This is the foundation of all sensor fusion",
        ],
        going_deeper: "The complementary filter is actually a combination of a high-pass filter \
                       (for the gyroscope) and a low-pass filter (for the accelerometer). \
                       The Kalman filter is the 'optimal' version that automatically adjusts \
                       the blend based on sensor uncertainties.",
        math_details: "angle = α × (angle + gyro×dt) + (1-α) × accel_angle\n\n\
                       This is equivalent to:\n\
                       • High-pass filter on gyro: responds to fast changes\n\
                       • Low-pass filter on accel: captures slow/DC component\n\n\
                       Time constant: τ = α×dt / (1-α)",
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // LESSON 1: Kalman Filter (Optimal Sensor Fusion)
    // ═══════════════════════════════════════════════════════════════════════════
    Lesson {
        id: 1,
        title: "Kalman Filter",
        subtitle: "Optimal Sensor Fusion",
        icon: "📊",
        why_it_matters: "Used in everything from GPS to SpaceX rockets. It's the mathematically \
                         optimal way to combine predictions with measurements.",
        intuition: "The complementary filter works, but we had to guess the right α value. \
            What if the sensors automatically told us how much to trust them?\n\n\
            Enter the Kalman filter. Each sensor reports not just a measurement, but also \
            how confident it is. The GPS might say 'I think you're at position 100 ± 5 meters.' \
            The wheel odometry says 'I think you moved 2 meters ± 0.1.' The filter figures out \
            the best way to combine these.\n\n\
            The magic is the <strong>Kalman Gain</strong> - it automatically calculates the optimal blend. \
            When GPS is very uncertain, the gain is low (ignore it). When odometry has drifted \
            and become uncertain, the gain is high (trust GPS more). The filter tracks its own \
            uncertainty and updates it after each step.\n\n\
            Think of it like this: you're lost in a city. Your phone's GPS says you're on Main Street \
            (but GPS is spotty here). You've been counting steps and think you walked 200 meters north. \
            A smart combination of both clues is better than trusting either alone.",
        demo_explanation: "Watch the uncertainty ellipse around the robot:\n\n\
            • <strong>Green dot</strong>: True position (hidden from the filter)\n\
            • <strong>Cyan dot + ellipse</strong>: Kalman filter estimate with uncertainty\n\
            • <strong>Yellow dots</strong>: GPS measurements (noisy but absolute)\n\n\
            Notice how the ellipse:\n\
            • <strong>Grows</strong> during prediction (we're less sure where we are)\n\
            • <strong>Shrinks</strong> after GPS update (measurement reduces uncertainty)\n\n\
            Try increasing the GPS interval - watch drift accumulate, then snap back on update!",
        key_takeaways: &[
            "The Kalman filter calculates the optimal blend automatically",
            "It tracks uncertainty and updates it over time",
            "Prediction increases uncertainty; measurements decrease it",
            "The Kalman Gain tells you how much to trust each measurement",
        ],
        going_deeper: "The Kalman filter is optimal for systems that are: (1) linear, and \
                       (2) have Gaussian (bell-curve) noise. For non-linear systems, we use \
                       the Extended Kalman Filter (EKF) or Unscented Kalman Filter (UKF). \
                       For non-Gaussian situations, particle filters are better.",
        math_details: "State: μ (mean), Σ (covariance)\n\n\
                       PREDICT:\n\
                       μ' = A×μ + B×u\n\
                       Σ' = A×Σ×A^T + Q\n\n\
                       UPDATE:\n\
                       K = Σ'×H^T × (H×Σ'×H^T + R)^(-1)  [Kalman Gain]\n\
                       μ = μ' + K×(z - H×μ')  [correct mean]\n\
                       Σ = (I - K×H)×Σ'  [reduce covariance]",
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // LESSON 2: Particle Filter (When Things Get Non-Linear)
    // ═══════════════════════════════════════════════════════════════════════════
    Lesson {
        id: 2,
        title: "Particle Filter",
        subtitle: "Monte Carlo Localization",
        icon: "🎯",
        why_it_matters: "When your robot could be in multiple places at once (like after \
                         being picked up), the Kalman filter breaks down. Particles handle this.",
        intuition: "Kalman filters assume you have ONE guess about where you are, with some \
            uncertainty around it. But what if you have NO IDEA where you are?\n\n\
            Imagine waking up in a museum with a floor plan but no memory of how you got there. \
            You could be ANYWHERE. Instead of one guess, you need thousands.\n\n\
            The particle filter represents your belief with a cloud of particles - each one is \
            a hypothesis: 'maybe I'm HERE.' As you walk and observe, particles in wrong places \
            gradually realize 'wait, I wouldn't see the T-Rex from here!' and fade away. \
            Eventually, the surviving particles cluster around your true location.\n\n\
            This is called <strong>Monte Carlo Localization</strong> - we're essentially running many \
            parallel simulations and keeping the ones that match reality.",
        demo_explanation: "Watch the particle cloud:\n\n\
            • <strong>Green triangle</strong>: True robot pose (hidden from filter)\n\
            • <strong>Orange dots</strong>: Particles (hypotheses about where robot might be)\n\
            • <strong>Cyan triangle</strong>: Estimated pose (weighted average of particles)\n\
            • <strong>Blue squares</strong>: Landmarks (known positions)\n\
            • <strong>Yellow lines</strong>: Sensor measurements to landmarks\n\n\
            The algorithm cycles through:\n\
            1. <strong>PREDICT</strong>: Move all particles with motion noise (they spread out)\n\
            2. <strong>UPDATE</strong>: Weight particles by sensor match (wrong ones get low weight)\n\
            3. <strong>RESAMPLE</strong>: Clone high-weight particles, kill low-weight ones\n\
            4. <strong>ESTIMATE</strong>: Compute weighted average\n\n\
            Use <strong>Step Mode</strong> to see each phase individually!",
        key_takeaways: &[
            "Particles represent multiple hypotheses simultaneously",
            "Natural selection: particles matching observations survive",
            "Can handle 'kidnapped robot' problem (global localization)",
            "More particles = better accuracy but higher computation",
        ],
        going_deeper: "Particle filters can handle any probability distribution and non-linear \
                       dynamics. The tradeoff is computation - you need many particles for \
                       accuracy. Techniques like adaptive resampling and importance sampling \
                       make them practical. FastSLAM uses particles for robot pose and separate \
                       Kalman filters for each landmark.",
        math_details: "For N particles with weights w_i:\n\n\
                       PREDICT: x_i' ~ p(x_t | u_t, x_i)\n\
                       UPDATE: w_i = p(z_t | x_i') × w_i, then normalize\n\
                       RESAMPLE when N_eff = 1/Σw_i² gets too low\n\
                       ESTIMATE: x̂ = Σ w_i × x_i",
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // LESSON 3: EKF SLAM (Mapping While Localizing)
    // ═══════════════════════════════════════════════════════════════════════════
    Lesson {
        id: 3,
        title: "EKF SLAM",
        subtitle: "Building the Map While You Navigate",
        icon: "🗺️",
        why_it_matters: "The chicken-and-egg problem: you need a map to localize, but need \
                         to know your location to build a map. SLAM solves both simultaneously.",
        intuition: "So far, we've assumed the robot knows where the landmarks are. But what if \
            it's exploring a completely unknown environment?\n\n\
            Imagine exploring a dark cave with a flashlight. You place glow sticks at interesting \
            spots to help navigate. But here's the problem: you're not sure exactly where YOU are, \
            so you're not sure exactly where you put the glow sticks. And you need the glow sticks \
            to figure out where you are. It's circular!\n\n\
            EKF SLAM solves this by estimating EVERYTHING at once - your position AND all the \
            landmark positions. When you see a new landmark, you add it to your map. When you \
            see a landmark again, you update your belief about where BOTH you and the landmark are.\n\n\
            The magic happens with <strong>correlations</strong>. If you're uncertain about landmark A's position, \
            and you're uncertain about your position, these uncertainties are connected. When you \
            get better information about one, it helps the other!",
        demo_explanation: "Watch the robot explore:\n\n\
            • Robot discovers landmarks and adds them to the map\n\
            • Uncertainty ellipses show how confident we are\n\
            • When robot revisits a landmark, watch the ellipses shrink!\n\n\
            The key insight: revisiting landmarks reduces uncertainty in BOTH \
            the robot pose AND other landmarks (through correlations).",
        key_takeaways: &[
            "SLAM estimates robot pose AND map simultaneously",
            "New observations create correlations between estimates",
            "Loop closure (revisiting) dramatically reduces uncertainty",
            "Computational cost grows with number of landmarks",
        ],
        going_deeper: "EKF SLAM is O(n²) per update, making it impractical for large maps. \
                       Modern alternatives include FastSLAM (particles + EKFs) and \
                       graph-based SLAM (next lesson). EKF SLAM also struggles with \
                       data association - figuring out WHICH landmark you're seeing.",
        math_details: "State vector: [robot_x, robot_y, robot_θ, lm1_x, lm1_y, lm2_x, ...]\n\n\
                       The covariance matrix tracks correlations between ALL pairs:\n\
                       Σ = [Σ_rr  Σ_rm₁  Σ_rm₂ ...]\n\
                           [Σ_m₁r Σ_m₁m₁ Σ_m₁m₂...]\n\
                           [...                    ]\n\n\
                       Observing landmark i updates ALL correlated estimates.",
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // LESSON 4: Graph SLAM (Modern Large-Scale SLAM)
    // ═══════════════════════════════════════════════════════════════════════════
    Lesson {
        id: 4,
        title: "Graph SLAM",
        subtitle: "Scaling to Real-World Maps",
        icon: "🔗",
        why_it_matters: "EKF SLAM doesn't scale. Modern self-driving cars and robots use \
                         graph-based SLAM to build maps with millions of points.",
        intuition: "EKF SLAM maintains a giant covariance matrix that grows quadratically. \
            For a map with 10,000 landmarks, that's 100 million numbers to track. Not practical.\n\n\
            Graph SLAM takes a different approach. Think of poses (where the robot was at each time) \
            as <strong>nodes</strong> in a graph. Constraints between poses (from odometry or landmark observations) \
            are <strong>edges</strong>. Each edge is like a rubber band with a preferred length.\n\n\
            The key insight: we don't need to solve this incrementally. We can collect all the \
            constraints and solve them all at once using optimization. This is MUCH more efficient \
            and lets us exploit <strong>sparsity</strong> - each pose only connects to nearby poses.\n\n\
            Loop closure becomes beautifully simple: when we recognize 'I've been here before!', \
            we add a new edge connecting the current pose to the old one. Then we re-optimize, \
            and the whole trajectory snaps into consistency, like a tensioned web.",
        demo_explanation: "Watch the pose graph:\n\n\
            • <strong>Nodes</strong>: Robot poses at each timestep\n\
            • <strong>Blue edges</strong>: Odometry constraints (sequential)\n\
            • <strong>Green edges</strong>: Loop closure constraints\n\n\
            Notice how drift accumulates without loop closure.\n\
            Click 'Add Loop Closure' when the robot returns to a previous area,\n\
            then 'Optimize' to see the graph snap into consistency!",
        key_takeaways: &[
            "Represent SLAM as a graph optimization problem",
            "Edges are constraints with uncertainty",
            "Loop closures connect distant nodes",
            "Batch optimization is more efficient than incremental updates",
            "Sparse structure enables large-scale maps",
        ],
        going_deeper: "State-of-the-art SLAM systems like ORB-SLAM, LIO-SAM, and Cartographer \
                       are all graph-based. They use sophisticated loop closure detection \
                       (bag-of-words, scan matching) and efficient solvers (g2o, GTSAM, Ceres). \
                       Multi-session SLAM and lifelong mapping are active research areas.",
        math_details: "Minimize: Σ_ij e_ij^T × Ω_ij × e_ij\n\n\
                       where e_ij = z_ij - h(x_i, x_j) is the error between \
                       expected and observed relative pose.\n\n\
                       Solved via Gauss-Newton or Levenberg-Marquardt.\n\
                       Sparse Cholesky factorization exploits graph structure.",
    },
];
