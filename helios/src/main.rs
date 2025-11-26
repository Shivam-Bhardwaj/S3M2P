use wasm_bindgen::prelude::*;
use web_sys::window;

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
    
    // Get canvas element
    if let Some(_canvas) = document.get_element_by_id("helios-canvas") {
        log("Canvas found - ready for WebGPU rendering");
    }
}
