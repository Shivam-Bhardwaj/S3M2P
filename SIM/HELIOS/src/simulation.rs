// Heliosphere Simulation State - SoA Layout following too.foo patterns
// All computation in Rust, minimal JS interop

use std::f64::consts::PI;

// ============================================================================
// CONSTANTS
// ============================================================================

pub const AU_KM: f64 = 149_597_870.7;
pub const SOLAR_RADIUS_KM: f64 = 695_700.0;
pub const J2000_EPOCH: f64 = 2_451_545.0;

// Time constants
pub const EARTH_YEAR_DAYS: f64 = 365.25;
pub const SOLAR_CYCLE_DAYS: f64 = 4018.0; // ~11 years
pub const SOLAR_CYCLE_YEARS: f64 = 11.0;

// Fixed capacities - no runtime allocation
pub const MAX_PLANETS: usize = 16;
pub const MAX_MOONS: usize = 64;
pub const MAX_MISSIONS: usize = 32;
pub const MAX_ASTEROIDS: usize = 256;
pub const ORBIT_SEGMENTS: usize = 128;

// ============================================================================
// VIEW STATE
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragMode {
    None,
    Pan,   // Left-click drag: pan view
    Orbit, // Right-click or Shift+drag: orbit camera around Sun
}

#[derive(Clone, Copy, Debug)]
pub struct ViewState {
    // Camera position (AU from sun, ecliptic plane)
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64, // AU per pixel (smaller = more zoomed in)

    // 3D view angles (radians) - CAD-like orbital camera
    pub tilt: f64,     // Camera elevation angle (0 = top-down, PI/2 = edge-on)
    pub rotation: f64, // Camera rotation around Z axis (azimuth)

    // Viewport dimensions
    pub width: f64,
    pub height: f64,

    // Interaction state - CAD-like controls
    pub drag_mode: DragMode,
    pub drag_start_x: f64,
    pub drag_start_y: f64,
    pub last_center_x: f64,
    pub last_center_y: f64,
    pub last_tilt: f64,
    pub last_rotation: f64,

    // Pinch-to-zoom state
    pub pinching: bool,
    pub pinch_start_dist: f64,
    pub pinch_start_zoom: f64,
    pub pinch_center_x: f64,
    pub pinch_center_y: f64,

    // Animation
    pub auto_rotate: bool,
    pub rotation_speed: f64,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 0.02, // 1 pixel = 0.02 AU, shows inner solar system
            tilt: 0.4,  // ~23 degrees from top-down for nice 3D effect
            rotation: 0.0,
            width: 1920.0,
            height: 1080.0,
            drag_mode: DragMode::None,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            last_center_x: 0.0,
            last_center_y: 0.0,
            last_tilt: 0.4,
            last_rotation: 0.0,
            pinching: false,
            pinch_start_dist: 0.0,
            pinch_start_zoom: 0.0,
            pinch_center_x: 0.0,
            pinch_center_y: 0.0,
            auto_rotate: false,
            rotation_speed: 0.0,
        }
    }
}

impl ViewState {
    /// Check if currently in any drag mode
    #[inline]
    pub fn is_dragging(&self) -> bool {
        self.drag_mode != DragMode::None
    }

    /// Check if panning
    #[inline]
    pub fn is_panning(&self) -> bool {
        self.drag_mode == DragMode::Pan
    }

    /// Check if orbiting camera
    #[inline]
    pub fn is_orbiting(&self) -> bool {
        self.drag_mode == DragMode::Orbit
    }
}

impl ViewState {
    /// Convert 3D AU coordinates to screen pixels using isometric-style projection
    /// Returns (screen_x, screen_y, depth) where depth can be used for sorting
    #[inline]
    pub fn au_to_screen_3d(&self, x_au: f64, y_au: f64, z_au: f64) -> (f64, f64, f64) {
        // Apply camera rotation around Z axis
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        let x_rot = x_au * cos_rot - y_au * sin_rot;
        let y_rot = x_au * sin_rot + y_au * cos_rot;

        // Apply tilt (elevation angle) - foreshortens Y based on tilt
        let cos_tilt = self.tilt.cos();
        let sin_tilt = self.tilt.sin();

        // Project to screen coordinates
        // X stays the same, Y is compressed by tilt and offset by Z
        let proj_x = x_rot;
        let proj_y = y_rot * cos_tilt - z_au * sin_tilt;
        let depth = y_rot * sin_tilt + z_au * cos_tilt;

        let screen_x = (proj_x - self.center_x) / self.zoom + self.width / 2.0;
        let screen_y = (proj_y - self.center_y) / self.zoom + self.height / 2.0;

        (screen_x, screen_y, depth)
    }

    /// Convert AU coordinates to screen pixels (2D, ignores Z)
    #[inline]
    pub fn au_to_screen(&self, x_au: f64, y_au: f64) -> (f64, f64) {
        let (sx, sy, _) = self.au_to_screen_3d(x_au, y_au, 0.0);
        (sx, sy)
    }

    /// Convert screen pixels to AU coordinates (assumes Z=0 plane)
    #[inline]
    pub fn screen_to_au(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        // Inverse projection (simplified, assumes z=0)
        let proj_x = (screen_x - self.width / 2.0) * self.zoom + self.center_x;
        let proj_y = (screen_y - self.height / 2.0) * self.zoom + self.center_y;

        // Inverse tilt
        let cos_tilt = self.tilt.cos();
        let y_rot = proj_y / cos_tilt;
        let x_rot = proj_x;

        // Inverse rotation
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        let x_au = x_rot * cos_rot + y_rot * sin_rot;
        let y_au = -x_rot * sin_rot + y_rot * cos_rot;

        (x_au, y_au)
    }

    /// Check if AU position is visible on screen (with margin)
    #[inline]
    pub fn is_visible(&self, x_au: f64, y_au: f64, margin_au: f64) -> bool {
        let half_w = self.width / 2.0 * self.zoom + margin_au;
        let half_h = self.height / 2.0 * self.zoom + margin_au;

        (x_au - self.center_x).abs() < half_w && (y_au - self.center_y).abs() < half_h
    }

    /// Check if 3D position is visible
    #[inline]
    pub fn is_visible_3d(&self, x_au: f64, y_au: f64, z_au: f64, margin_au: f64) -> bool {
        let (sx, sy, _) = self.au_to_screen_3d(x_au, y_au, z_au);
        sx > -margin_au / self.zoom
            && sx < self.width + margin_au / self.zoom
            && sy > -margin_au / self.zoom
            && sy < self.height + margin_au / self.zoom
    }

    /// Get visible AU range
    pub fn visible_range(&self) -> (f64, f64, f64, f64) {
        let half_w = self.width / 2.0 * self.zoom;
        let half_h = self.height / 2.0 * self.zoom;
        (
            self.center_x - half_w,
            self.center_y - half_h,
            self.center_x + half_w,
            self.center_y + half_h,
        )
    }

    /// Zoom level for LOD decisions
    pub fn lod_level(&self) -> u8 {
        if self.zoom < 0.001 {
            4
        }
        // Extreme close-up
        else if self.zoom < 0.01 {
            3
        }
        // Planet detail
        else if self.zoom < 0.1 {
            2
        }
        // Inner system
        else if self.zoom < 1.0 {
            1
        }
        // Outer system
        else {
            0
        } // Heliosphere scale
    }

    /// Set camera tilt angle (0 = top-down, PI/2 = edge-on)
    pub fn set_tilt(&mut self, tilt: f64) {
        self.tilt = tilt.clamp(0.0, PI * 0.45); // Max ~80 degrees
    }

    /// Set camera rotation around Z axis
    pub fn set_rotation(&mut self, rotation: f64) {
        self.rotation = rotation % (2.0 * PI);
    }
}

// ============================================================================
// ORBITAL ELEMENTS (Keplerian)
// ============================================================================

#[derive(Clone, Copy, Debug, Default)]
pub struct OrbitalElements {
    pub a: f64,     // Semi-major axis (AU)
    pub e: f64,     // Eccentricity
    pub i: f64,     // Inclination (radians)
    pub omega: f64, // Longitude of ascending node (radians)
    pub w: f64,     // Argument of perihelion (radians)
    pub m0: f64,    // Mean anomaly at epoch (radians)
    pub n: f64,     // Mean motion (radians/day)
}

impl OrbitalElements {
    pub fn new(
        a: f64,
        e: f64,
        i_deg: f64,
        omega_deg: f64,
        w_deg: f64,
        l0_deg: f64,
        period_years: f64,
    ) -> Self {
        let i = i_deg.to_radians();
        let omega = omega_deg.to_radians();
        let w = w_deg.to_radians();
        let m0 = (l0_deg - w_deg - omega_deg).to_radians();
        let n = 2.0 * PI / (period_years * 365.25);
        Self {
            a,
            e,
            i,
            omega,
            w,
            m0,
            n,
        }
    }

    /// Calculate true anomaly and distance at Julian date
    #[inline]
    fn solve_kepler(&self, jd: f64) -> (f64, f64) {
        let days = jd - J2000_EPOCH;
        let m = (self.m0 + self.n * days) % (2.0 * PI);

        // Kepler solver (5 iterations is enough for e < 0.3)
        let mut e_anom = m + self.e * m.sin();
        for _ in 0..5 {
            let delta = (e_anom - self.e * e_anom.sin() - m) / (1.0 - self.e * e_anom.cos());
            e_anom -= delta;
        }

        // True anomaly
        let sqrt_term = ((1.0 + self.e) / (1.0 - self.e)).sqrt();
        let true_anom = 2.0 * (sqrt_term * (e_anom / 2.0).tan()).atan();

        // Distance
        let r = self.a * (1.0 - self.e * e_anom.cos());

        (true_anom, r)
    }

    /// Calculate full 3D (x, y, z) position at Julian date
    /// Uses proper orbital mechanics with inclination and longitude of ascending node
    #[inline]
    pub fn position_3d(&self, jd: f64) -> (f64, f64, f64) {
        let (true_anom, r) = self.solve_kepler(jd);

        // Position in orbital plane (perifocal coordinates)
        let angle = true_anom + self.w;
        let x_orb = r * angle.cos();
        let y_orb = r * angle.sin();

        // Transform from orbital plane to ecliptic coordinates
        // Using rotation matrices for Omega (longitude of ascending node) and i (inclination)
        let cos_omega = self.omega.cos();
        let sin_omega = self.omega.sin();
        let cos_i = self.i.cos();
        let sin_i = self.i.sin();

        // Apply rotations: first around orbital plane, then inclination, then ascending node
        let x = x_orb * cos_omega - y_orb * cos_i * sin_omega;
        let y = x_orb * sin_omega + y_orb * cos_i * cos_omega;
        let z = y_orb * sin_i;

        (x, y, z)
    }

    /// Calculate 3D position for a given true anomaly (for orbit path drawing)
    #[inline]
    pub fn position_3d_at_anomaly(&self, true_anom: f64) -> (f64, f64, f64) {
        // Distance at this true anomaly
        let r = self.a * (1.0 - self.e * self.e) / (1.0 + self.e * true_anom.cos());

        // Position in orbital plane
        let angle = true_anom + self.w;
        let x_orb = r * angle.cos();
        let y_orb = r * angle.sin();

        // Transform to ecliptic coordinates
        let cos_omega = self.omega.cos();
        let sin_omega = self.omega.sin();
        let cos_i = self.i.cos();
        let sin_i = self.i.sin();

        let x = x_orb * cos_omega - y_orb * cos_i * sin_omega;
        let y = x_orb * sin_omega + y_orb * cos_i * cos_omega;
        let z = y_orb * sin_i;

        (x, y, z)
    }

    /// Calculate (x, y) position at Julian date (2D ecliptic projection - legacy)
    #[inline]
    pub fn position_2d(&self, jd: f64) -> (f64, f64) {
        let (x, y, _z) = self.position_3d(jd);
        (x, y)
    }
}

// ============================================================================
// SIMULATION STATE - SoA LAYOUT
// ============================================================================

pub struct SimulationState {
    // === TIME ===
    pub julian_date: f64,
    pub time_scale: f64, // Days per second (1.0 = real-time, 365.25 = 1 year/sec)
    pub paused: bool,

    // === PLANETS (SoA) ===
    pub planet_count: usize,
    pub planet_names: [&'static str; MAX_PLANETS],
    pub planet_orbits: [OrbitalElements; MAX_PLANETS],
    pub planet_radii_km: [f64; MAX_PLANETS],
    pub planet_colors: [&'static str; MAX_PLANETS],
    pub planet_has_rings: [bool; MAX_PLANETS],

    // Pre-computed positions (updated each frame) - now with Z for 3D
    pub planet_x: [f64; MAX_PLANETS],
    pub planet_y: [f64; MAX_PLANETS],
    pub planet_z: [f64; MAX_PLANETS],

    // Pre-computed orbit paths (updated on zoom change) - now with 3D (x, y, z per segment)
    pub orbit_paths: [[f64; ORBIT_SEGMENTS * 3]; MAX_PLANETS],
    pub orbit_dirty: bool,

    // === MISSIONS (SoA) ===
    pub mission_count: usize,
    pub mission_names: [&'static str; MAX_MISSIONS],
    pub mission_colors: [&'static str; MAX_MISSIONS],
    pub mission_x: [f64; MAX_MISSIONS],
    pub mission_y: [f64; MAX_MISSIONS],
    pub mission_active: [bool; MAX_MISSIONS],

    // Mission trajectory data (simplified waypoints)
    pub mission_waypoints: [[(f64, f64, f64); 8]; MAX_MISSIONS], // (jd, x, y)
    pub mission_waypoint_counts: [usize; MAX_MISSIONS],

    // === VIEW STATE ===
    pub view: ViewState,

    // === HELIOSPHERE ===
    pub termination_shock_au: f64,
    pub heliopause_au: f64,
    pub bow_shock_au: f64,

    // === SOLAR CYCLE ===
    // Solar cycle is ~11 years (4018 days)
    // Phase 0.0 = solar minimum, 0.5 = solar maximum
    pub solar_cycle_phase: f64,
    // Base reference date for solar cycle (Solar Cycle 25 minimum: Dec 2019)
    pub solar_cycle_ref_jd: f64,

    // === SCRATCH BUFFERS (avoid allocation) ===
    scratch_visible: [bool; MAX_PLANETS],

    // === FRAME STATS ===
    pub frame_count: u64,
    pub last_fps_update: f64,
    pub fps: f64,
}

impl SimulationState {
    pub fn new() -> Self {
        // Default: 25% solar cycle per second = ~2.75 years/sec = ~1004.5 days/sec
        // This means one full solar cycle takes 4 seconds - nice pulsation speed
        let quarter_solar_cycle_per_sec = SOLAR_CYCLE_DAYS / 4.0; // ~1004.5 days/sec
        let mut state = Self {
            julian_date: J2000_EPOCH + 8766.0,       // ~2024
            time_scale: quarter_solar_cycle_per_sec, // 25% solar cycle per second
            paused: false,

            planet_count: 0,
            planet_names: [""; MAX_PLANETS],
            planet_orbits: [OrbitalElements::default(); MAX_PLANETS],
            planet_radii_km: [0.0; MAX_PLANETS],
            planet_colors: [""; MAX_PLANETS],
            planet_has_rings: [false; MAX_PLANETS],
            planet_x: [0.0; MAX_PLANETS],
            planet_y: [0.0; MAX_PLANETS],
            planet_z: [0.0; MAX_PLANETS],
            orbit_paths: [[0.0; ORBIT_SEGMENTS * 3]; MAX_PLANETS],
            orbit_dirty: true,

            mission_count: 0,
            mission_names: [""; MAX_MISSIONS],
            mission_colors: [""; MAX_MISSIONS],
            mission_x: [0.0; MAX_MISSIONS],
            mission_y: [0.0; MAX_MISSIONS],
            mission_active: [false; MAX_MISSIONS],
            mission_waypoints: [[(0.0, 0.0, 0.0); 8]; MAX_MISSIONS],
            mission_waypoint_counts: [0; MAX_MISSIONS],

            view: ViewState::default(),

            termination_shock_au: 94.0,
            heliopause_au: 121.0,
            bow_shock_au: 230.0,

            // Solar Cycle 25 minimum was around December 2019 (JD 2458849)
            solar_cycle_phase: 0.0,
            solar_cycle_ref_jd: 2458849.0,

            scratch_visible: [false; MAX_PLANETS],

            frame_count: 0,
            last_fps_update: 0.0,
            fps: 60.0,
        };

        state.init_solar_system();
        state.init_missions();
        state
    }

    fn init_solar_system(&mut self) {
        // Sun is implicit at (0, 0)

        // Mercury
        self.add_planet(
            "Mercury",
            OrbitalElements::new(0.387, 0.206, 7.0, 48.3, 29.1, 252.3, 0.241),
            2439.7,
            "#B5B5B5",
            false,
        );

        // Venus
        self.add_planet(
            "Venus",
            OrbitalElements::new(0.723, 0.007, 3.4, 76.7, 54.9, 182.0, 0.615),
            6051.8,
            "#E6C87A",
            false,
        );

        // Earth
        self.add_planet(
            "Earth",
            OrbitalElements::new(1.000, 0.017, 0.0, 174.9, 288.1, 100.5, 1.0),
            6371.0,
            "#6B93D6",
            false,
        );

        // Mars
        self.add_planet(
            "Mars",
            OrbitalElements::new(1.524, 0.093, 1.85, 49.6, 286.5, 355.5, 1.881),
            3389.5,
            "#C1440E",
            false,
        );

        // Jupiter
        self.add_planet(
            "Jupiter",
            OrbitalElements::new(5.203, 0.048, 1.3, 100.5, 273.9, 34.4, 11.86),
            69911.0,
            "#D4A57A",
            false,
        );

        // Saturn (with rings)
        self.add_planet(
            "Saturn",
            OrbitalElements::new(9.537, 0.054, 2.5, 113.7, 339.4, 50.0, 29.46),
            58232.0,
            "#E3D4AD",
            true,
        );

        // Uranus
        self.add_planet(
            "Uranus",
            OrbitalElements::new(19.19, 0.047, 0.8, 74.0, 97.0, 313.2, 84.01),
            25362.0,
            "#B5E3E3",
            true,
        );

        // Neptune
        self.add_planet(
            "Neptune",
            OrbitalElements::new(30.07, 0.009, 1.8, 131.8, 276.3, 304.9, 164.8),
            24622.0,
            "#5B7FDE",
            false,
        );
    }

    fn add_planet(
        &mut self,
        name: &'static str,
        orbit: OrbitalElements,
        radius_km: f64,
        color: &'static str,
        has_rings: bool,
    ) {
        if self.planet_count >= MAX_PLANETS {
            return;
        }
        let i = self.planet_count;
        self.planet_names[i] = name;
        self.planet_orbits[i] = orbit;
        self.planet_radii_km[i] = radius_km;
        self.planet_colors[i] = color;
        self.planet_has_rings[i] = has_rings;
        self.planet_count += 1;
    }

    fn init_missions(&mut self) {
        // Voyager 1
        self.add_mission(
            "Voyager 1",
            "#FFD700",
            &[
                (2443391.5, 1.0, 0.0),    // Launch 1977
                (2444200.0, 5.2, 1.0),    // Jupiter 1979
                (2444600.0, 9.5, 3.0),    // Saturn 1980
                (2451545.0, 75.0, 20.0),  // 2000
                (2460676.0, 163.0, 45.0), // 2025
            ],
        );

        // Voyager 2
        self.add_mission(
            "Voyager 2",
            "#00CED1",
            &[
                (2443375.5, 1.0, 0.0),
                (2444100.0, 5.2, -1.0),
                (2444700.0, 9.5, -3.0),
                (2445700.0, 19.2, -8.0),
                (2446400.0, 30.0, -12.0),
                (2460676.0, 137.0, -50.0),
            ],
        );

        // New Horizons
        self.add_mission(
            "New Horizons",
            "#FF6347",
            &[
                (2453755.5, 1.0, 0.0),   // Launch 2006
                (2454159.0, 5.2, 0.5),   // Jupiter 2007
                (2457216.0, 33.0, 5.0),  // Pluto 2015
                (2460676.0, 58.0, 10.0), // 2025
            ],
        );

        // Parker Solar Probe
        self.add_mission(
            "Parker Solar",
            "#FF4500",
            &[
                (2458340.5, 1.0, 0.0), // Launch 2018
                (2458800.0, 0.17, 0.0),
                (2459200.0, 0.05, 0.0),
                (2460000.0, 0.046, 0.0), // Closest approach
            ],
        );
    }

    fn add_mission(
        &mut self,
        name: &'static str,
        color: &'static str,
        waypoints: &[(f64, f64, f64)],
    ) {
        if self.mission_count >= MAX_MISSIONS {
            return;
        }
        let i = self.mission_count;
        self.mission_names[i] = name;
        self.mission_colors[i] = color;
        self.mission_active[i] = true;

        let count = waypoints.len().min(8);
        for j in 0..count {
            self.mission_waypoints[i][j] = waypoints[j];
        }
        self.mission_waypoint_counts[i] = count;
        self.mission_count += 1;
    }

    /// Main update - call once per frame
    pub fn update(&mut self, dt: f64) {
        if !self.paused {
            // time_scale is in days/second, dt is in seconds
            // So time_scale * dt gives us the change in julian_date (days)
            self.julian_date += self.time_scale * dt;
        }

        // Update solar cycle phase (11 year cycle = ~4018 days)
        self.update_solar_cycle();

        // Update planet positions (full 3D)
        for i in 0..self.planet_count {
            let (x, y, z) = self.planet_orbits[i].position_3d(self.julian_date);
            self.planet_x[i] = x;
            self.planet_y[i] = y;
            self.planet_z[i] = z;
        }

        // Update mission positions
        for i in 0..self.mission_count {
            if !self.mission_active[i] {
                continue;
            }
            let (x, y) = self.interpolate_mission(i);
            self.mission_x[i] = x;
            self.mission_y[i] = y;
        }

        // Rebuild orbit paths if needed (zoom changed or first frame)
        if self.orbit_dirty {
            self.rebuild_orbit_paths();
            self.orbit_dirty = false;
        }

        self.frame_count += 1;
    }

    /// Update solar cycle phase and heliosphere boundaries
    fn update_solar_cycle(&mut self) {
        const SOLAR_CYCLE_DAYS: f64 = 4018.0; // ~11 years

        // Calculate phase (0.0 to 1.0, where 0.5 is solar maximum)
        let days_since_min = self.julian_date - self.solar_cycle_ref_jd;
        let cycle_position = (days_since_min / SOLAR_CYCLE_DAYS).rem_euclid(1.0);
        self.solar_cycle_phase = cycle_position;

        // Heliosphere expansion/contraction based on solar activity
        // At solar maximum, solar wind pressure is higher, pushing boundaries out
        // At solar minimum, boundaries contract
        // Using a smooth sinusoidal variation
        let activity = (cycle_position * 2.0 * std::f64::consts::PI).sin();
        let activity_factor = 0.5 + 0.5 * activity; // 0.0 to 1.0

        // Termination shock: 85 AU (min) to 100 AU (max)
        self.termination_shock_au = 85.0 + 15.0 * activity_factor;

        // Heliopause: 110 AU (min) to 130 AU (max)
        self.heliopause_au = 110.0 + 20.0 * activity_factor;

        // Bow shock: 200 AU (min) to 250 AU (max)
        self.bow_shock_au = 200.0 + 50.0 * activity_factor;
    }

    /// Get solar cycle phase description
    pub fn solar_cycle_name(&self) -> &'static str {
        let phase = self.solar_cycle_phase;
        if phase < 0.125 || phase >= 0.875 {
            "Solar Min"
        } else if phase < 0.375 {
            "Rising"
        } else if phase < 0.625 {
            "Solar Max"
        } else {
            "Declining"
        }
    }

    /// Get current date as a year with decimal
    pub fn current_year(&self) -> f64 {
        2000.0 + (self.julian_date - J2000_EPOCH) / 365.25
    }

    fn interpolate_mission(&self, idx: usize) -> (f64, f64) {
        let count = self.mission_waypoint_counts[idx];
        if count == 0 {
            return (0.0, 0.0);
        }

        let jd = self.julian_date;
        let wps = &self.mission_waypoints[idx];

        // Before first waypoint
        if jd <= wps[0].0 {
            return (wps[0].1, wps[0].2);
        }

        // Find bracketing waypoints
        for i in 0..count - 1 {
            if jd >= wps[i].0 && jd <= wps[i + 1].0 {
                let t = (jd - wps[i].0) / (wps[i + 1].0 - wps[i].0);
                return (
                    wps[i].1 + t * (wps[i + 1].1 - wps[i].1),
                    wps[i].2 + t * (wps[i + 1].2 - wps[i].2),
                );
            }
        }

        // Extrapolate from last segment
        let last = count - 1;
        if last > 0 {
            let dt = wps[last].0 - wps[last - 1].0;
            let t = (jd - wps[last].0) / dt;
            (
                wps[last].1 + t * (wps[last].1 - wps[last - 1].1),
                wps[last].2 + t * (wps[last].2 - wps[last - 1].2),
            )
        } else {
            (wps[0].1, wps[0].2)
        }
    }

    fn rebuild_orbit_paths(&mut self) {
        for p in 0..self.planet_count {
            let orbit = &self.planet_orbits[p];
            for i in 0..ORBIT_SEGMENTS {
                // True anomaly around the orbit
                let true_anom = 2.0 * PI * (i as f64) / (ORBIT_SEGMENTS as f64);
                // Get full 3D position using proper orbital mechanics
                let (x, y, z) = orbit.position_3d_at_anomaly(true_anom);
                self.orbit_paths[p][i * 3] = x;
                self.orbit_paths[p][i * 3 + 1] = y;
                self.orbit_paths[p][i * 3 + 2] = z;
            }
        }
    }

    /// Mark orbits for rebuild (call on zoom change)
    pub fn mark_orbits_dirty(&mut self) {
        self.orbit_dirty = true;
    }

    // === VIEW CONTROL ===

    pub fn set_viewport(&mut self, width: f64, height: f64) {
        self.view.width = width;
        self.view.height = height;
    }

    pub fn zoom_to(&mut self, level: f64) {
        // Allow very small zoom for planet close-ups (10^-8 allows planet-scale viewing)
        self.view.zoom = level.clamp(0.00000001, 10.0);
        self.orbit_dirty = true;
    }

    pub fn zoom_by(&mut self, factor: f64) {
        self.zoom_to(self.view.zoom * factor);
    }

    pub fn pan_by(&mut self, dx_screen: f64, dy_screen: f64) {
        self.view.center_x -= dx_screen * self.view.zoom;
        self.view.center_y -= dy_screen * self.view.zoom;
    }

    pub fn focus_on_planet(&mut self, idx: usize) {
        if idx < self.planet_count {
            // Check if we're already focused on this planet (within tolerance)
            let dx = (self.view.center_x - self.planet_x[idx]).abs();
            let dy = (self.view.center_y - self.planet_y[idx]).abs();
            let already_focused = dx < 0.01 && dy < 0.01;

            self.view.center_x = self.planet_x[idx];
            self.view.center_y = self.planet_y[idx];

            // Only zoom if not already focused on this planet
            if !already_focused {
                // NASA Eyes style: planet fills ~60% of screen
                // Calculate zoom so planet appears large and prominent
                // We want the planet to be about 200-300 pixels in radius on screen
                // zoom = AU per pixel, so smaller zoom = more zoomed in
                let radius_au = self.planet_radii_km[idx] / AU_KM;

                // Target: planet radius should be ~150 pixels on screen
                // screen_radius = radius_au / zoom => zoom = radius_au / screen_radius
                let target_screen_radius = 150.0; // pixels
                let target_zoom = radius_au / target_screen_radius;

                // Clamp to reasonable bounds
                self.zoom_to(target_zoom.max(0.00000001).min(0.01));
            }
        }
    }

    pub fn focus_on_sun(&mut self) {
        self.view.center_x = 0.0;
        self.view.center_y = 0.0;
        self.zoom_to(0.001);
    }

    pub fn view_inner_system(&mut self) {
        self.view.center_x = 0.0;
        self.view.center_y = 0.0;
        self.zoom_to(0.01); // Shows Mercury to Mars
    }

    pub fn view_outer_system(&mut self) {
        self.view.center_x = 0.0;
        self.view.center_y = 0.0;
        self.zoom_to(0.15); // Shows out to Neptune
    }

    pub fn view_heliosphere(&mut self) {
        // Position sun at 2/3 of screen to show heliosphere with direction of motion
        // Sun moves through interstellar medium roughly in the +X direction (towards "nose")
        // Desktop (landscape): sun at 2/3 from left, shows tail to right
        // Mobile (portrait): sun at 2/3 from bottom, shows tail above
        let is_portrait = self.view.height > self.view.width;

        if is_portrait {
            // Mobile: sun at 2/3 from bottom (1/3 from top)
            // Offset in Y to show more of the tail above
            self.view.center_x = 0.0;
            self.view.center_y = -self.bow_shock_au * 0.3; // Shift view down, sun appears higher
        } else {
            // Desktop: sun at 2/3 from left (1/3 from right)
            // Offset in X to show more of the tail to the right
            self.view.center_x = self.bow_shock_au * 0.3; // Shift view right, sun appears left
            self.view.center_y = 0.0;
        }
        self.zoom_to(1.2); // Shows full heliosphere including bow shock
    }

    // === TIME CONTROL ===

    pub fn set_time_scale(&mut self, days_per_second: f64) {
        self.time_scale = days_per_second.clamp(-365250.0, 365250.0);
    }

    /// Set time scale in Earth years per second
    pub fn set_time_scale_years(&mut self, years_per_second: f64) {
        self.set_time_scale(years_per_second * EARTH_YEAR_DAYS);
    }

    /// Set time scale in solar cycles per second
    pub fn set_time_scale_solar_cycles(&mut self, cycles_per_second: f64) {
        self.set_time_scale(cycles_per_second * SOLAR_CYCLE_DAYS);
    }

    /// Get time scale in Earth years per second
    pub fn time_scale_years(&self) -> f64 {
        self.time_scale / EARTH_YEAR_DAYS
    }

    /// Get time scale in solar cycles per second
    pub fn time_scale_solar_cycles(&self) -> f64 {
        self.time_scale / SOLAR_CYCLE_DAYS
    }

    /// Format time scale as human-readable string
    /// Shows percentage of solar cycle (11 years) per second
    pub fn time_scale_str(&self) -> String {
        if self.paused {
            return "Paused".to_string();
        }

        let abs_ts = self.time_scale.abs();
        let sign = if self.time_scale < 0.0 { "-" } else { "" };

        // Show as percentage of solar cycle (11 years)
        let solar_cycle_percent = (abs_ts / SOLAR_CYCLE_DAYS) * 100.0;

        if solar_cycle_percent >= 100.0 {
            // Multiple solar cycles per second
            let cycles = solar_cycle_percent / 100.0;
            format!("{}{}x ☉cycle/s", sign, cycles as i32)
        } else if solar_cycle_percent >= 1.0 {
            // Percentage of solar cycle - main display mode
            format!("{}{:.0}% ☉cycle/s", sign, solar_cycle_percent)
        } else {
            // Very slow - show in years
            let years = abs_ts / EARTH_YEAR_DAYS;
            if years >= 0.1 {
                format!("{}{:.1} yr/s", sign, years)
            } else if abs_ts >= 1.0 {
                format!("{}{:.0} d/s", sign, abs_ts)
            } else {
                format!("{}{:.1}x", sign, abs_ts)
            }
        }
    }

    /// Step time scale up by multiplier (for speed control buttons)
    pub fn speed_up(&mut self, factor: f64) {
        let new_scale = self.time_scale * factor;
        self.set_time_scale(new_scale);
    }

    /// Step time scale down by divisor
    pub fn slow_down(&mut self, factor: f64) {
        let new_scale = self.time_scale / factor;
        self.set_time_scale(new_scale);
    }

    pub fn set_date(&mut self, year: i32, month: u32, day: u32) {
        self.julian_date = jd_from_date(year, month, day);
    }

    pub fn get_date(&self) -> (i32, u32, u32) {
        date_from_jd(self.julian_date)
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Reset 3D view to default
    pub fn reset_3d_view(&mut self) {
        self.view.tilt = 0.4;
        self.view.rotation = 0.0;
        self.mark_orbits_dirty();
    }
}

// ============================================================================
// TIME UTILITIES
// ============================================================================

pub fn jd_from_date(year: i32, month: u32, day: u32) -> f64 {
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;

    day as f64 + ((153 * m + 2) / 5) as f64 + (365 * y) as f64 + (y / 4) as f64 - (y / 100) as f64
        + (y / 400) as f64
        - 32045.0
}

pub fn date_from_jd(jd: f64) -> (i32, u32, u32) {
    let z = jd.floor() as i64;
    let a = if z < 2299161 {
        z
    } else {
        let alpha = ((z as f64 - 1867216.25) / 36524.25).floor() as i64;
        z + 1 + alpha - alpha / 4
    };

    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d - (30.6001 * e as f64).floor() as i64) as u32;
    let month = if e < 14 { e - 1 } else { e - 13 } as u32;
    let year = if month > 2 { c - 4716 } else { c - 4715 } as i32;

    (year, month, day)
}
