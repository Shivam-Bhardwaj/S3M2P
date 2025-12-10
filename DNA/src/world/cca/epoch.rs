//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: epoch.rs | DNA/src/world/cca/epoch.rs
//! PURPOSE: Time representation for celestial mechanics
//! CREATED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Epoch
//!
//! Time representation for celestial mechanics computations. Internally stores
//! time as seconds since J2000.0 TDB (Barycentric Dynamical Time).
//!
//! ## Time Scales
//!
//! - **TDB** (Barycentric Dynamical Time) - Used for planetary ephemerides
//! - **TT** (Terrestrial Time) - Used for Earth-based observations
//! - **UTC** (Coordinated Universal Time) - Civil time with leap seconds
//! - **TAI** (International Atomic Time) - UTC without leap seconds
//!
//! ## Julian Date
//!
//! J2000.0 = 2000 January 1, 12:00:00 TT = JD 2451545.0
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use super::constants::{DAYS_PER_YEAR, J2000_EPOCH_JD, SECONDS_PER_DAY};
use std::cmp::Ordering;
use std::ops::{Add, Sub};

/// Time scale for epoch representation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TimeScale {
    /// Barycentric Dynamical Time - for planetary ephemerides
    #[default]
    TDB,
    /// Terrestrial Time - for Earth observations
    TT,
    /// International Atomic Time - continuous, no leap seconds
    TAI,
    /// Coordinated Universal Time - civil time with leap seconds
    UTC,
    /// GPS Time - continuous since 1980
    GPS,
}

/// Epoch - a point in time for celestial mechanics
///
/// Internally represented as seconds since J2000.0 TDB
#[derive(Clone, Copy, Debug, Default)]
pub struct Epoch {
    /// Seconds since J2000.0 TDB (2000 Jan 1, 12:00:00 TT)
    seconds_j2000_tdb: f64,
}

impl Epoch {
    /// Create an epoch from seconds since J2000.0 TDB
    #[inline]
    pub const fn from_seconds_j2000(seconds: f64) -> Self {
        Self {
            seconds_j2000_tdb: seconds,
        }
    }

    /// J2000.0 epoch (2000 January 1, 12:00:00 TT)
    #[inline]
    pub const fn j2000() -> Self {
        Self {
            seconds_j2000_tdb: 0.0,
        }
    }

    /// Create from Julian Date
    ///
    /// # Arguments
    /// * `jd` - Julian Date
    /// * `scale` - Time scale of the input (currently only TDB/TT fully supported)
    #[inline]
    pub fn from_jd(jd: f64, _scale: TimeScale) -> Self {
        let days_since_j2000 = jd - J2000_EPOCH_JD;
        Self {
            seconds_j2000_tdb: days_since_j2000 * SECONDS_PER_DAY,
        }
    }

    /// Create from calendar date (UTC approximation)
    ///
    /// Note: This is an approximation that ignores leap seconds and
    /// TDB-UTC differences. For high precision, use proper ephemeris data.
    pub fn from_date(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: f64) -> Self {
        let jd = gregorian_to_jd(year, month, day, hour, min, sec);
        Self::from_jd(jd, TimeScale::TDB)
    }

    /// Create from ISO 8601 string (basic parsing)
    ///
    /// Supports formats: "2024-01-15T12:00:00Z", "2024-01-15"
    pub fn from_iso(s: &str) -> Result<Self, EpochError> {
        // Basic parsing - for production, use a proper date library
        let s = s.trim();

        // Try full ISO format: YYYY-MM-DDTHH:MM:SSZ
        if let Some(t_pos) = s.find('T') {
            let date_part = &s[..t_pos];
            let time_part = s[t_pos + 1..].trim_end_matches('Z');

            let (year, month, day) = parse_date(date_part)?;
            let (hour, min, sec) = parse_time(time_part)?;

            return Ok(Self::from_date(year, month, day, hour, min, sec));
        }

        // Try date only: YYYY-MM-DD
        let (year, month, day) = parse_date(s)?;
        Ok(Self::from_date(year, month, day, 12, 0, 0.0))
    }

    /// Get seconds since J2000.0 TDB
    #[inline]
    pub fn seconds_j2000(&self) -> f64 {
        self.seconds_j2000_tdb
    }

    /// Get Julian Date (TDB scale)
    #[inline]
    pub fn to_jd(&self, _scale: TimeScale) -> f64 {
        J2000_EPOCH_JD + self.seconds_j2000_tdb / SECONDS_PER_DAY
    }

    /// Get Julian Date (shorthand for TDB)
    #[inline]
    pub fn jd(&self) -> f64 {
        self.to_jd(TimeScale::TDB)
    }

    /// Get days since J2000.0
    #[inline]
    pub fn days_j2000(&self) -> f64 {
        self.seconds_j2000_tdb / SECONDS_PER_DAY
    }

    /// Get Julian centuries since J2000.0 (for precession/nutation)
    #[inline]
    pub fn centuries_j2000(&self) -> f64 {
        self.days_j2000() / (DAYS_PER_YEAR * 100.0)
    }

    /// Get year as decimal (approximate)
    #[inline]
    pub fn year(&self) -> f64 {
        2000.0 + self.days_j2000() / DAYS_PER_YEAR
    }

    /// Convert to calendar date (year, month, day, hour, min, sec)
    pub fn to_date(&self) -> (i32, u32, u32, u32, u32, f64) {
        jd_to_gregorian(self.jd())
    }

    /// Convert to ISO 8601 string
    pub fn to_iso(&self) -> String {
        let (year, month, day, hour, min, sec) = self.to_date();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:06.3}Z",
            year, month, day, hour, min, sec
        )
    }

    /// Add duration in seconds
    #[inline]
    pub fn add_seconds(&self, seconds: f64) -> Self {
        Self {
            seconds_j2000_tdb: self.seconds_j2000_tdb + seconds,
        }
    }

    /// Add duration in days
    #[inline]
    pub fn add_days(&self, days: f64) -> Self {
        self.add_seconds(days * SECONDS_PER_DAY)
    }

    /// Add duration in years (Julian years)
    #[inline]
    pub fn add_years(&self, years: f64) -> Self {
        self.add_days(years * DAYS_PER_YEAR)
    }

    /// Difference in seconds
    #[inline]
    pub fn diff_seconds(&self, other: &Self) -> f64 {
        self.seconds_j2000_tdb - other.seconds_j2000_tdb
    }

    /// Difference in days
    #[inline]
    pub fn diff_days(&self, other: &Self) -> f64 {
        self.diff_seconds(other) / SECONDS_PER_DAY
    }
}

impl PartialEq for Epoch {
    fn eq(&self, other: &Self) -> bool {
        (self.seconds_j2000_tdb - other.seconds_j2000_tdb).abs() < 1e-9
    }
}

impl Eq for Epoch {}

impl PartialOrd for Epoch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Epoch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seconds_j2000_tdb
            .partial_cmp(&other.seconds_j2000_tdb)
            .unwrap_or(Ordering::Equal)
    }
}

impl std::hash::Hash for Epoch {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash with nanosecond precision
        let nanos = (self.seconds_j2000_tdb * 1e9) as i64;
        nanos.hash(state);
    }
}

/// Duration in seconds
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Duration {
    /// Duration in seconds
    pub seconds: f64,
}

impl Duration {
    /// Create from seconds
    #[inline]
    pub const fn from_seconds(seconds: f64) -> Self {
        Self { seconds }
    }

    /// Create from days
    #[inline]
    pub fn from_days(days: f64) -> Self {
        Self {
            seconds: days * SECONDS_PER_DAY,
        }
    }

    /// Create from years
    #[inline]
    pub fn from_years(years: f64) -> Self {
        Self::from_days(years * DAYS_PER_YEAR)
    }

    /// Get duration in days
    #[inline]
    pub fn days(&self) -> f64 {
        self.seconds / SECONDS_PER_DAY
    }

    /// Get duration in years
    #[inline]
    pub fn years(&self) -> f64 {
        self.days() / DAYS_PER_YEAR
    }
}

impl Add<Duration> for Epoch {
    type Output = Epoch;

    #[inline]
    fn add(self, duration: Duration) -> Epoch {
        self.add_seconds(duration.seconds)
    }
}

impl Sub for Epoch {
    type Output = Duration;

    #[inline]
    fn sub(self, other: Epoch) -> Duration {
        Duration {
            seconds: self.diff_seconds(&other),
        }
    }
}

/// Error type for epoch parsing
#[derive(Debug)]
pub enum EpochError {
    /// Invalid format
    ParseError(String),
    /// Value out of range
    RangeError(String),
}

impl std::fmt::Display for EpochError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpochError::ParseError(s) => write!(f, "Parse error: {}", s),
            EpochError::RangeError(s) => write!(f, "Range error: {}", s),
        }
    }
}

impl std::error::Error for EpochError {}

// ============================================================================
// Helper functions for date conversion
// ============================================================================

/// Convert Gregorian date to Julian Date
fn gregorian_to_jd(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: f64) -> f64 {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };

    let a = (y / 100) as f64;
    let b = 2.0 - a + (a / 4.0).floor();

    let jd =
        (365.25 * (y + 4716) as f64).floor() + (30.6001 * (m + 1) as f64).floor() + day as f64 + b
            - 1524.5;

    // Add time of day
    jd + (hour as f64 + min as f64 / 60.0 + sec / 3600.0) / 24.0
}

/// Convert Julian Date to Gregorian date
fn jd_to_gregorian(jd: f64) -> (i32, u32, u32, u32, u32, f64) {
    let z = (jd + 0.5).floor() as i64;
    let f = jd + 0.5 - z as f64;

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

    // Extract time of day
    let day_fraction = f * 24.0;
    let hour = day_fraction.floor() as u32;
    let min_fraction = (day_fraction - hour as f64) * 60.0;
    let min = min_fraction.floor() as u32;
    let sec = (min_fraction - min as f64) * 60.0;

    (year, month, day, hour, min, sec)
}

/// Parse date string "YYYY-MM-DD"
fn parse_date(s: &str) -> Result<(i32, u32, u32), EpochError> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return Err(EpochError::ParseError(format!(
            "Invalid date format: {}",
            s
        )));
    }

    let year = parts[0]
        .parse::<i32>()
        .map_err(|_| EpochError::ParseError("Invalid year".into()))?;
    let month = parts[1]
        .parse::<u32>()
        .map_err(|_| EpochError::ParseError("Invalid month".into()))?;
    let day = parts[2]
        .parse::<u32>()
        .map_err(|_| EpochError::ParseError("Invalid day".into()))?;

    if !(1..=12).contains(&month) {
        return Err(EpochError::RangeError("Month must be 1-12".into()));
    }
    if !(1..=31).contains(&day) {
        return Err(EpochError::RangeError("Day must be 1-31".into()));
    }

    Ok((year, month, day))
}

/// Parse time string "HH:MM:SS" or "HH:MM:SS.sss"
fn parse_time(s: &str) -> Result<(u32, u32, f64), EpochError> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 {
        return Err(EpochError::ParseError(format!(
            "Invalid time format: {}",
            s
        )));
    }

    let hour = parts[0]
        .parse::<u32>()
        .map_err(|_| EpochError::ParseError("Invalid hour".into()))?;
    let min = parts[1]
        .parse::<u32>()
        .map_err(|_| EpochError::ParseError("Invalid minute".into()))?;
    let sec = if parts.len() > 2 {
        parts[2]
            .parse::<f64>()
            .map_err(|_| EpochError::ParseError("Invalid second".into()))?
    } else {
        0.0
    };

    if hour >= 24 {
        return Err(EpochError::RangeError("Hour must be 0-23".into()));
    }
    if min >= 60 {
        return Err(EpochError::RangeError("Minute must be 0-59".into()));
    }
    if sec >= 60.0 {
        return Err(EpochError::RangeError("Second must be 0-59".into()));
    }

    Ok((hour, min, sec))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_j2000() {
        let j2000 = Epoch::j2000();
        assert_eq!(j2000.seconds_j2000(), 0.0);
        assert!((j2000.jd() - J2000_EPOCH_JD).abs() < EPSILON);
    }

    #[test]
    fn test_from_jd() {
        let epoch = Epoch::from_jd(J2000_EPOCH_JD + 1.0, TimeScale::TDB);
        assert!((epoch.days_j2000() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_from_date() {
        // J2000.0 is 2000 Jan 1, 12:00:00
        let epoch = Epoch::from_date(2000, 1, 1, 12, 0, 0.0);
        assert!(epoch.seconds_j2000().abs() < 1.0); // Should be very close to 0
    }

    #[test]
    fn test_from_iso() {
        let epoch = Epoch::from_iso("2024-01-15T12:00:00Z").unwrap();
        let (year, month, day, hour, min, _sec) = epoch.to_date();

        assert_eq!(year, 2024);
        assert_eq!(month, 1);
        assert_eq!(day, 15);
        assert_eq!(hour, 12);
        assert_eq!(min, 0);
    }

    #[test]
    fn test_to_iso_roundtrip() {
        let original = Epoch::from_date(2024, 6, 15, 10, 30, 45.123);
        let iso = original.to_iso();
        let parsed = Epoch::from_iso(&iso).unwrap();

        assert!((original.seconds_j2000() - parsed.seconds_j2000()).abs() < 0.01);
    }

    #[test]
    fn test_add_days() {
        let epoch = Epoch::j2000();
        let later = epoch.add_days(1.0);

        assert!((later.days_j2000() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_duration() {
        let e1 = Epoch::j2000();
        let e2 = e1.add_days(10.0);
        let duration = e2 - e1;

        assert!((duration.days() - 10.0).abs() < EPSILON);
    }

    #[test]
    fn test_ordering() {
        let e1 = Epoch::j2000();
        let e2 = e1.add_days(1.0);

        assert!(e1 < e2);
        assert!(e2 > e1);
        assert_eq!(e1, e1);
    }
}
