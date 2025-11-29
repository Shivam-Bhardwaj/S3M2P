use glam::Vec3;
use std::f32::consts::PI;

// Solar wind constants
pub const FAST_WIND_SPEED: f32 = 750.0; // km/s
pub const SLOW_WIND_SPEED: f32 = 400.0; // km/s
pub const TYPICAL_DENSITY: f32 = 5.0; // protons/cm³ at 1 AU
pub const TYPICAL_TEMPERATURE: f32 = 1.2e5; // K at 1 AU
pub const CARRINGTON_PERIOD: f32 = 27.2753; // days
pub const SIDEREAL_PERIOD: f32 = 25.38; // days
pub const SOLAR_RADIUS: f32 = 6.96e5; // km
pub const AU_KM: f32 = 1.496e8; // km
pub const PROTON_MASS: f32 = 1.673e-27; // kg
pub const BOLTZMANN: f32 = 1.38e-23; // J/K

#[derive(Clone, Debug)]
pub struct NumberTimeSeries {
    epochs: Vec<f32>,
    values: Vec<f32>,
}

impl NumberTimeSeries {
    pub fn new(epochs: Vec<f32>, values: Vec<f32>) -> Self {
        Self { epochs, values }
    }

    pub fn interpolate(&self, time: f32) -> f32 {
        if self.epochs.is_empty() {
            return 0.0;
        }
        if self.epochs.len() == 1 {
            return self.values[0];
        }

        let idx = match self
            .epochs
            .binary_search_by(|t| t.partial_cmp(&time).unwrap())
        {
            Ok(i) => i,
            Err(i) => i,
        };

        if idx == 0 {
            return self.values[0];
        }
        if idx >= self.epochs.len() {
            return *self.values.last().unwrap();
        }

        let t0 = self.epochs[idx - 1];
        let t1 = self.epochs[idx];
        let v0 = self.values[idx - 1];
        let v1 = self.values[idx];

        let t = (time - t0) / (t1 - t0);
        v0 + (v1 - v0) * t
    }
}

#[derive(Clone, Debug)]
pub struct Vector3TimeSeries {
    epochs: Vec<f32>,
    values: Vec<Vec3>,
}

impl Vector3TimeSeries {
    pub fn new(epochs: Vec<f32>, values: Vec<Vec3>) -> Self {
        Self { epochs, values }
    }

    pub fn interpolate(&self, time: f32) -> Vec3 {
        if self.epochs.is_empty() {
            return Vec3::ZERO;
        }
        if self.epochs.len() == 1 {
            return self.values[0];
        }

        let idx = match self
            .epochs
            .binary_search_by(|t| t.partial_cmp(&time).unwrap())
        {
            Ok(i) => i,
            Err(i) => i,
        };

        if idx == 0 {
            return self.values[0];
        }
        if idx >= self.epochs.len() {
            return *self.values.last().unwrap();
        }

        let t0 = self.epochs[idx - 1];
        let t1 = self.epochs[idx];
        let v0 = self.values[idx - 1];
        let v1 = self.values[idx];

        let t = (time - t0) / (t1 - t0);
        v0.lerp(v1, t)
    }
}

pub struct ParkerSpiral {
    solar_rotation_rate: f32,  // rad/s
    solar_magnetic_field: f32, // Tesla
}

impl Default for ParkerSpiral {
    fn default() -> Self {
        Self::new()
    }
}

impl ParkerSpiral {
    pub fn new() -> Self {
        let solar_rotation_rate = 2.0 * PI / (SIDEREAL_PERIOD * 24.0 * 3600.0);
        let solar_magnetic_field = 1e-4; // 1 Gauss = 10^-4 Tesla
        Self {
            solar_rotation_rate,
            solar_magnetic_field,
        }
    }

    pub fn get_spiral_angle(&self, r_au: f32, solar_wind_speed_km_s: f32) -> f32 {
        let r_meters = r_au * AU_KM * 1000.0;
        let v_meters = solar_wind_speed_km_s * 1000.0;

        let tan_psi = (self.solar_rotation_rate * r_meters) / v_meters;
        tan_psi.atan()
    }

    pub fn get_magnetic_field(&self, position_au: Vec3, solar_wind_speed_km_s: f32) -> Vec3 {
        let r = position_au.length();
        if r < 1e-6 {
            return Vec3::ZERO;
        }

        let theta = (position_au.z / r).acos(); // Co-latitude
        let phi = position_au.y.atan2(position_au.x); // Azimuth

        let spiral_angle = self.get_spiral_angle(r, solar_wind_speed_km_s);

        // B0 scales as r^-2
        let r_sun_au = SOLAR_RADIUS / AU_KM;
        let b0 = self.solar_magnetic_field * (r_sun_au / r).powi(2);

        let br = b0;
        let b_phi = -b0 * r * spiral_angle.sin() / spiral_angle.cos(); // simplified

        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        let bx = br * sin_theta * cos_phi + b_phi * (-sin_phi);
        let by = br * sin_theta * sin_phi + b_phi * cos_phi;
        let bz = br * cos_theta;

        Vec3::new(bx, by, bz) * 1e9 // Convert to nT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parker_spiral_angle() {
        let ps = ParkerSpiral::new();
        // At 1 AU with 400 km/s wind
        let angle = ps.get_spiral_angle(1.0, 400.0);
        let angle_deg = angle * 180.0 / PI;

        assert!(
            angle_deg > 40.0 && angle_deg < 50.0,
            "Expected ~45 deg, got {}",
            angle_deg
        );
    }

    #[test]
    fn test_magnetic_field_direction() {
        let ps = ParkerSpiral::new();
        let pos = Vec3::new(1.0, 0.0, 0.0);
        let b = ps.get_magnetic_field(pos, 400.0);

        assert!(b.z.abs() < 1e-6);
        assert!(b.x.abs() > 1e-9);
        assert!(b.y.abs() > 1e-9);
    }

    #[test]
    fn test_timeseries_interpolation() {
        let ts = NumberTimeSeries::new(vec![0.0, 10.0], vec![100.0, 200.0]);
        assert_eq!(ts.interpolate(0.0), 100.0);
        assert_eq!(ts.interpolate(10.0), 200.0);
        assert_eq!(ts.interpolate(5.0), 150.0);
    }

    #[test]
    fn test_vector_timeseries_interpolation() {
        let ts = Vector3TimeSeries::new(
            vec![0.0, 10.0],
            vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0)],
        );
        let v = ts.interpolate(5.0);
        assert!((v.x - 5.0).abs() < 1e-6);
        assert!((v.y - 5.0).abs() < 1e-6);
    }
}
