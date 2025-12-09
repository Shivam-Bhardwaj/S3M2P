use std::f32::consts::PI;
use wasm_bindgen::prelude::*;

// Re-export types for WASM
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeliosphereMorphology {
    Cometary = 0,
    Croissant = 1,
    Bubble = 2,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct HeliosphereParameters {
    // Nose radius (upwind direction) in AU
    pub r_hp_nose: f32,

    // Termination shock to heliopause ratio (typically 0.75-0.85)
    pub r_ts_over_hp: f32,

    // Direction of ISM inflow (unit vector in HEE_J2000)
    // Storing as [x, y, z] for easier WASM interop
    pub nose_vec: Vec<f32>,

    // ISM conditions
    pub ism_rho: f32, // density (particles/cm³)
    pub ism_t: f32,   // temperature (K)
    pub ism_b: f32,   // magnetic field strength (nT)

    // Solar wind conditions
    pub sw_mdot: f32, // mass loss rate (proxy)
    pub sw_v: f32,    // wind speed (km/s)

    // Morphology
    pub morphology: HeliosphereMorphology,

    // Shape coefficients (morphology-dependent)
    pub shape_params: Vec<f32>,
}

#[wasm_bindgen]
impl HeliosphereParameters {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        r_hp_nose: f32,
        r_ts_over_hp: f32,
        nose_vec: Vec<f32>,
        ism_rho: f32,
        ism_t: f32,
        ism_b: f32,
        sw_mdot: f32,
        sw_v: f32,
        morphology: HeliosphereMorphology,
        shape_params: Vec<f32>,
    ) -> Self {
        Self {
            r_hp_nose,
            r_ts_over_hp,
            nose_vec,
            ism_rho,
            ism_t,
            ism_b,
            sw_mdot,
            sw_v,
            morphology,
            shape_params,
        }
    }
}

#[wasm_bindgen]
pub struct HeliosphereSurface {
    params: HeliosphereParameters,
}

#[wasm_bindgen]
impl HeliosphereSurface {
    #[wasm_bindgen(constructor)]
    pub fn new(params: HeliosphereParameters) -> Self {
        Self { params }
    }

    /// Calculate heliopause radius at given spherical angles (radians)
    /// theta: Polar angle (0 at +Z, PI at -Z)
    /// phi: Azimuthal angle (0 at +X, PI/2 at +Y)
    pub fn heliopause_radius(&self, theta: f32, phi: f32) -> f32 {
        // Cosine of angle from nose direction
        // Nose is along -nose_vec (ISM flows toward Sun)
        // nose_vec is assumed to be normalized
        let nx = self.params.nose_vec.first().unwrap_or(&1.0);
        let ny = self.params.nose_vec.get(1).unwrap_or(&0.0);
        let nz = self.params.nose_vec.get(2).unwrap_or(&0.0);

        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        // Direction vector in Cartesian
        let dx = sin_theta * cos_phi;
        let dy = sin_theta * sin_phi;
        let dz = cos_theta;

        // Angle from nose (0 = upwind, PI = downwind)
        // Dot product: d . n
        // Note: JS code used -nose_vec
        let cos_alpha = -(dx * nx + dy * ny + dz * nz);
        let alpha = cos_alpha.clamp(-1.0, 1.0).acos();

        let r_nose = self.params.r_hp_nose;

        match self.params.morphology {
            HeliosphereMorphology::Cometary => self.cometary_shape(alpha, r_nose),
            HeliosphereMorphology::Croissant => self.croissant_shape(alpha, r_nose, theta),
            HeliosphereMorphology::Bubble => self.bubble_shape(alpha, r_nose),
        }
    }

    pub fn termination_shock_radius(&self, theta: f32, phi: f32) -> f32 {
        let hp_radius = self.heliopause_radius(theta, phi);
        hp_radius * self.params.r_ts_over_hp
    }

    fn cometary_shape(&self, alpha: f32, r_nose: f32) -> f32 {
        let a0 = self.params.shape_params.first().unwrap_or(&1.0);
        let a1 = self.params.shape_params.get(1).unwrap_or(&2.5);
        let a2 = self.params.shape_params.get(2).unwrap_or(&0.5);

        let cos_alpha = alpha.cos();
        let factor = a0 + a1 * cos_alpha + a2 * cos_alpha * cos_alpha;

        r_nose * factor.max(0.1)
    }

    fn croissant_shape(&self, alpha: f32, r_nose: f32, theta: f32) -> f32 {
        let asymmetry = self.params.shape_params.first().unwrap_or(&1.5);
        let flattening = self.params.shape_params.get(1).unwrap_or(&0.7);
        let tail_spread = self.params.shape_params.get(2).unwrap_or(&0.3);

        let base_radius = self.cometary_shape(alpha, r_nose);

        let latitude_factor = 1.0 - flattening * theta.sin().powi(2);

        let tail_factor = if alpha > PI / 2.0 {
            1.0 + tail_spread * (2.0 * (alpha - PI / 2.0)).sin()
        } else {
            1.0
        };

        base_radius * asymmetry * latitude_factor * tail_factor
    }

    fn bubble_shape(&self, alpha: f32, r_nose: f32) -> f32 {
        let asphericity = self.params.shape_params.first().unwrap_or(&0.1);
        let factor = 1.0 + asphericity * alpha.cos();
        r_nose * factor
    }

    pub fn update_parameters(&mut self, params: HeliosphereParameters) {
        self.params = params;
    }

    pub fn get_parameters(&self) -> HeliosphereParameters {
        self.params.clone()
    }

    // Mesh generation helper
    // Returns a flat vector of [x, y, z, nx, ny, nz, ...]
    pub fn generate_mesh_data(
        &self,
        theta_steps: usize,
        phi_steps: usize,
        is_termination_shock: bool,
    ) -> Vec<f32> {
        let mut data = Vec::with_capacity((theta_steps + 1) * (phi_steps + 1) * 6);

        for i in 0..=theta_steps {
            let theta = (i as f32 / theta_steps as f32) * PI;

            for j in 0..=phi_steps {
                let phi = (j as f32 / phi_steps as f32) * 2.0 * PI;

                let r = if is_termination_shock {
                    self.termination_shock_radius(theta, phi)
                } else {
                    self.heliopause_radius(theta, phi)
                };

                let sin_theta = theta.sin();
                let x = r * sin_theta * phi.cos();
                let y = r * sin_theta * phi.sin();
                let z = r * theta.cos();

                // Approximate normal (simple sphere normal for now, could improve)
                // True normal requires derivatives
                let nx = sin_theta * phi.cos();
                let ny = sin_theta * phi.sin();
                let nz = theta.cos();

                data.push(x);
                data.push(y);
                data.push(z);
                data.push(nx);
                data.push(ny);
                data.push(nz);
            }
        }
        data
    }

    pub fn generate_indices(&self, theta_steps: usize, phi_steps: usize) -> Vec<u32> {
        let mut indices = Vec::new();
        for i in 0..theta_steps {
            for j in 0..phi_steps {
                let a = (i * (phi_steps + 1) + j) as u32;
                let b = a + (phi_steps + 1) as u32;

                indices.push(a);
                indices.push(b);
                indices.push(a + 1);

                indices.push(b);
                indices.push(b + 1);
                indices.push(a + 1);
            }
        }
        indices
    }
}

#[wasm_bindgen]
pub fn interpolate_parameters(
    params0: &HeliosphereParameters,
    params1: &HeliosphereParameters,
    t: f32,
) -> HeliosphereParameters {
    let t = t.clamp(0.0, 1.0);

    let r_hp_nose = params0.r_hp_nose * (1.0 - t) + params1.r_hp_nose * t;
    let r_ts_over_hp = params0.r_ts_over_hp * (1.0 - t) + params1.r_ts_over_hp * t;

    let n0 = &params0.nose_vec;
    let n1 = &params1.nose_vec;

    let nx = n0.first().unwrap_or(&1.0) * (1.0 - t) + n1.first().unwrap_or(&1.0) * t;
    let ny = n0.get(1).unwrap_or(&0.0) * (1.0 - t) + n1.get(1).unwrap_or(&0.0) * t;
    let nz = n0.get(2).unwrap_or(&0.0) * (1.0 - t) + n1.get(2).unwrap_or(&0.0) * t;

    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    let nose_vec = vec![nx / len, ny / len, nz / len];

    let ism_rho = params0.ism_rho * (1.0 - t) + params1.ism_rho * t;
    let ism_t = params0.ism_t * (1.0 - t) + params1.ism_t * t;
    let ism_b = params0.ism_b * (1.0 - t) + params1.ism_b * t;
    let sw_mdot = params0.sw_mdot * (1.0 - t) + params1.sw_mdot * t;
    let sw_v = params0.sw_v * (1.0 - t) + params1.sw_v * t;

    let morphology = if t < 0.5 {
        params0.morphology
    } else {
        params1.morphology
    };

    let max_params = params0.shape_params.len().max(params1.shape_params.len());
    let mut shape_params = Vec::with_capacity(max_params);

    for i in 0..max_params {
        let p0 = params0.shape_params.get(i).unwrap_or(&0.0);
        let p1 = params1.shape_params.get(i).unwrap_or(&0.0);
        shape_params.push(p0 * (1.0 - t) + p1 * t);
    }

    HeliosphereParameters {
        r_hp_nose,
        r_ts_over_hp,
        nose_vec,
        ism_rho,
        ism_t,
        ism_b,
        sw_mdot,
        sw_v,
        morphology,
        shape_params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_default_params() -> HeliosphereParameters {
        HeliosphereParameters::new(
            100.0,
            0.8,
            vec![1.0, 0.0, 0.0],
            1.0,
            1.0,
            1.0,
            1.0,
            1.0,
            HeliosphereMorphology::Bubble,
            vec![0.1],
        )
    }

    #[test]
    fn test_heliosphere_parameters_creation() {
        let params = create_default_params();
        assert_eq!(params.r_hp_nose, 100.0);
        assert_eq!(params.morphology, HeliosphereMorphology::Bubble);
    }

    #[test]
    fn test_heliopause_radius_nose() {
        let params = create_default_params();
        let surface = HeliosphereSurface::new(params);

        // At theta=pi/2, phi=pi (nose direction if nose_vec is +X and we look from origin towards -X)
        let r = surface.heliopause_radius(PI / 2.0, PI);
        // Bubble shape at nose (alpha=0) is r_nose * (1 + param)
        // 100 * (1 + 0.1 * 1) = 110
        assert!((r - 110.0).abs() < 0.01);
    }

    #[test]
    fn test_termination_shock_ratio() {
        let params = create_default_params();
        let surface = HeliosphereSurface::new(params);

        let r_hp = surface.heliopause_radius(PI / 2.0, PI);
        let r_ts = surface.termination_shock_radius(PI / 2.0, PI);

        assert!((r_ts / r_hp - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_interpolation() {
        let p0 = create_default_params();
        let mut p1 = create_default_params();
        p1.r_hp_nose = 200.0;

        let p_mid = interpolate_parameters(&p0, &p1, 0.5);
        assert!((p_mid.r_hp_nose - 150.0).abs() < 0.01);
    }
}
