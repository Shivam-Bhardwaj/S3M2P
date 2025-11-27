use wasm_bindgen::prelude::*;
use web_sys::window;
use antimony_core::{HeliosphereSurface, HeliosphereParameters, HeliosphereMorphology};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn main() {
    console_error_panic_hook::set_once();
    
    let window = window().unwrap();
    let document = window.document().unwrap();
    
    log("Helios initialized");
    
    // Test Heliosphere Logic
    log("Testing Heliosphere Surface logic from antimony-core...");
    
    let params = HeliosphereParameters::new(
        120.0, // r_hp_nose
        0.8,   // r_ts_over_hp
        vec![1.0, 0.0, 0.0], // nose_vec
        0.1,   // ism_rho
        6000.0, // ism_t
        0.5,   // ism_b
        2e-14, // sw_mdot
        400.0, // sw_v
        HeliosphereMorphology::Cometary,
        vec![1.0, 2.5, 0.5], // shape_params
    );
    
    let surface = HeliosphereSurface::new(params);
    
    // Test nose direction (theta=PI/2, phi=0 corresponds to X axis? No, theta=0 is Z, theta=PI/2 is XY plane. Phi=0 is X)
    // In the code: 
    // dx = sin_theta * cos_phi
    // dy = sin_theta * sin_phi
    // dz = cos_theta
    // If theta=PI/2, phi=0 => dx=1, dy=0, dz=0. This aligns with nose_vec=[1,0,0].
    // alpha should be 0 (upwind).
    
    let r_nose = surface.heliopause_radius(std::f32::consts::PI / 2.0, 0.0);
    log(&format!("Heliosphere radius at nose (theta=PI/2, phi=0): {:.2} AU", r_nose));
    
    // Test tail direction (theta=PI/2, phi=PI) => dx=-1
    let r_tail = surface.heliopause_radius(std::f32::consts::PI / 2.0, std::f32::consts::PI);
    log(&format!("Heliosphere radius at tail (theta=PI/2, phi=PI): {:.2} AU", r_tail));

    // Get canvas element
    if let Some(_canvas) = document.get_element_by_id("helios-canvas") {
        log("Canvas found - ready for WebGPU rendering");
    }
}
