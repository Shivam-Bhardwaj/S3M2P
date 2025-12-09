//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: heliosphere_model.rs | HELIOS/src/heliosphere_model.rs
//! PURPOSE: Physics model for heliospheric boundaries - Parker spiral, termination shock
//! MODIFIED: 2025-12-09
//! LAYER: HELIOS (project)
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::solar_wind::{NumberTimeSeries, Vector3TimeSeries, BOLTZMANN, PROTON_MASS};
use glam::Vec3;

// Constants
// AU_TO_M removed
const KM_TO_M: f32 = 1000.0;
const NT_TO_T: f32 = 1e-9;
const CM3_TO_M3: f32 = 1e6;

pub struct HeliospherePhysicsModel {
    pub solar_wind_speed: NumberTimeSeries,
    pub solar_wind_density: NumberTimeSeries,
    pub solar_wind_temperature: NumberTimeSeries,
    pub magnetic_field_strength: Vector3TimeSeries,

    pub ism_density: f32,
    pub ism_temperature: f32,
    pub ism_velocity: Vec3,
    pub ism_magnetic_field: Vec3,
}

impl Default for HeliospherePhysicsModel {
    fn default() -> Self {
        Self::new()
    }
}

impl HeliospherePhysicsModel {
    pub fn new() -> Self {
        Self {
            solar_wind_speed: NumberTimeSeries::new(vec![0.0], vec![400.0]),
            solar_wind_density: NumberTimeSeries::new(vec![0.0], vec![5.0]),
            solar_wind_temperature: NumberTimeSeries::new(vec![0.0], vec![1.2e5]),
            magnetic_field_strength: Vector3TimeSeries::new(
                vec![0.0],
                vec![Vec3::new(0.0, 0.0, 5.0)],
            ),

            ism_density: 0.1,
            ism_temperature: 6300.0,
            ism_velocity: Vec3::new(-26.3, 0.0, 0.0),
            ism_magnetic_field: Vec3::new(0.2, 0.1, 0.1),
        }
    }

    fn calculate_ram_pressure(&self, r: f32, julian_date: f32) -> f32 {
        let v = self.solar_wind_speed.interpolate(julian_date); // km/s
        let n = self.solar_wind_density.interpolate(julian_date); // cm^-3 at 1 AU

        // Density falls as r^-2
        let density_at_r = n * (1.0 / r).powi(2);

        let v_si = v * KM_TO_M;
        let n_si = density_at_r * CM3_TO_M3; // cm^-3 to m^-3

        // P = nmv^2
        let pressure = n_si * PROTON_MASS * v_si * v_si; // Pa
        pressure * 1e9 // nPa
    }

    fn calculate_magnetic_pressure(&self, r: f32, julian_date: f32) -> f32 {
        let b1au = self.magnetic_field_strength.interpolate(julian_date);

        // Parker spiral: Br ~ r^-2, Bphi ~ r^-1
        // Assuming B1AU is roughly (Bphi, Btheta, Br) or similar.
        // JS code: Br = B1AU.z * r^-2, Bphi = B1AU.x * r^-1
        let br = b1au.z * (1.0 / r).powi(2);
        let bphi = b1au.x * (1.0 / r);

        let b_total = (br * br + bphi * bphi).sqrt() * NT_TO_T;

        let mu0 = 4.0 * std::f32::consts::PI * 1e-7;
        let pressure = (b_total * b_total) / (2.0 * mu0);
        pressure * 1e9 // nPa
    }

    fn calculate_thermal_pressure(&self, r: f32, julian_date: f32) -> f32 {
        let t = self.solar_wind_temperature.interpolate(julian_date);
        let n = self.solar_wind_density.interpolate(julian_date) * (1.0 / r).powi(2);

        let n_si = n * CM3_TO_M3;
        let pressure = 2.0 * n_si * BOLTZMANN * t; // 2* for protons+electrons
        pressure * 1e9 // nPa
    }

    pub fn calculate_ism_pressure(&self) -> (f32, f32, f32) {
        // Ram
        let v_ism = self.ism_velocity.length() * KM_TO_M;
        let n_ism = self.ism_density * CM3_TO_M3;
        let ram = n_ism * PROTON_MASS * v_ism * v_ism * 1e9;

        // Thermal
        let thermal = 2.0 * n_ism * BOLTZMANN * self.ism_temperature * 1e9;

        // Magnetic
        let b_ism = self.ism_magnetic_field.length() * NT_TO_T;
        let mu0 = 4.0 * std::f32::consts::PI * 1e-7;
        let magnetic = (b_ism * b_ism) / (2.0 * mu0) * 1e9;

        (ram, thermal, magnetic)
    }

    pub fn calculate_termination_shock(&self, theta: f32, phi: f32, julian_date: f32) -> f32 {
        // Estimate TS distance based on Ram Pressure scaling
        // R_ts scales as sqrt(RamPressure)
        let v = self.solar_wind_speed.interpolate(julian_date);
        let n = self.solar_wind_density.interpolate(julian_date);

        // Base reference: v=400 km/s, n=5 cm^-3 -> R_ts ~ 90 AU
        let base_pressure_proxy = 400.0 * 400.0 * 5.0;
        let current_pressure_proxy = v * v * n;

        let mut r = 90.0 * (current_pressure_proxy / base_pressure_proxy).sqrt();

        // Asymmetry
        // JS: noseDirection = (-1, 0, 0)
        // direction = (sin theta cos phi, sin theta sin phi, cos theta)
        let dir = Vec3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );

        let nose_dir = Vec3::new(-1.0, 0.0, 0.0);
        let dot = dir.dot(nose_dir);

        if dot > 0.5 {
            r *= 0.94;
        } else if dot < -0.3 {
            r *= 2.2;
        } else {
            r *= 1.1;
        }

        r
    }

    pub fn calculate_heliopause(&self, theta: f32, phi: f32, julian_date: f32) -> f32 {
        let ts_dist = self.calculate_termination_shock(theta, phi, julian_date);
        let mut r = ts_dist + 30.0;

        let (ism_ram, ism_thermal, ism_mag) = self.calculate_ism_pressure();
        let total_ism_pressure = ism_ram + ism_thermal + ism_mag;

        for _ in 0..20 {
            let sw_ram = self.calculate_ram_pressure(r, julian_date);
            let sw_mag = self.calculate_magnetic_pressure(r, julian_date);
            let sw_therm = self.calculate_thermal_pressure(r, julian_date);

            let total_sw = sw_ram + sw_mag + sw_therm;
            let ratio = total_sw / total_ism_pressure;

            if (ratio - 1.0).abs() < 0.01 {
                break;
            }

            r *= ratio.powf(0.3);
        }

        // Asymmetry override (from JS code)
        let dir = Vec3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );
        let nose_dir = Vec3::new(-1.0, 0.0, 0.0);
        let dot = dir.dot(nose_dir);

        if dot > 0.5 {
            r = 121.0;
        } else if dot < -0.3 {
            r = 300.0 + 50.0 * dot.abs();
        } else {
            r = 140.0;
        }

        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_pressure_balance() {
        let model = HeliospherePhysicsModel::new();
        let (ram, thermal, mag) = model.calculate_ism_pressure();

        assert!(ram > 0.0);
        assert!(thermal > 0.0);
        assert!(mag > 0.0);
    }

    #[test]
    fn test_boundary_calculations() {
        let model = HeliospherePhysicsModel::new();

        // Nose direction (theta = 90 deg, phi = 180 deg -> x = -1)
        // wait, JS nose was (-1,0,0).
        // Spherical to cartesian: x = sin theta cos phi.
        // sin(PI/2) = 1. cos(PI) = -1. -> x = -1. Correct.
        let theta = PI / 2.0;
        let phi = PI;

        let ts = model.calculate_termination_shock(theta, phi, 0.0);
        let hp = model.calculate_heliopause(theta, phi, 0.0);

        assert!(ts > 0.0);
        assert!(hp > ts);
        // Voyager 1 TS ~ 94 AU, HP ~ 121 AU
        assert!((ts - 80.0).abs() < 40.0); // Wide margin because of simple model
        assert!((hp - 121.0).abs() < 1.0); // Hardcoded override in JS logic
    }
}
