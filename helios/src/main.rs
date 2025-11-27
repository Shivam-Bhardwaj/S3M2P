mod streaming;
use streaming::StreamController;
use glam::Vec3;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement};
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, DeviceDescriptor, Features, Instance, InstanceDescriptor,
    Limits, LoadOp, Operations, PowerPreference, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, TextureViewDescriptor,
};
use std::rc::Rc;
use std::cell::RefCell;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

fn main() {
    console_error_panic_hook::set_once();
    
    // Spawn async block
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = run().await {
            error(&format!("Application error: {:?}", e));
        }
    });
}

async fn run() -> Result<(), String> {
    let window = window().ok_or("No window found")?;
    let document = window.document().ok_or("No document found")?;
    
    log("Helios initialized");
    
    // Configuration
    // In production (Cloudflare Pages), this should be set to "https://data.too.foo" or similar
    // to avoid Mixed Content errors (HTTPS frontend -> HTTP backend).
    const BACKEND_URL: Option<&str> = option_env!("BACKEND_URL");
    let server_url = BACKEND_URL.unwrap_or("http://144.126.145.3:3000").to_string();
    
    log(&format!("Connecting to storage backend: {}", server_url));
    
    // Initialize Streaming Controller
    let streamer = Rc::new(RefCell::new(StreamController::new(server_url)));
    
    let canvas = document
        .get_element_by_id("helios-canvas")
        .ok_or("Canvas not found")?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "Element is not a canvas")?;

    log("Canvas found - Initializing WebGPU...");

    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
        .map_err(|e| format!("Failed to create surface: {:?}", e))?;

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or("No suitable GPU adapter found")?;

    log(&format!("Adapter found: {:?}", adapter.get_info()));

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                required_features: Features::empty(),
                required_limits: Limits::downlevel_webgl2_defaults(),
                label: None,
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .map_err(|e| format!("Failed to create device: {:?}", e))?;

    let surface_config = surface
        .get_default_config(&adapter, canvas.width(), canvas.height())
        .ok_or("Failed to get surface config")?;
    
    surface.configure(&device, &surface_config);

    log("WebGPU initialized successfully. Starting render loop.");

    // Animation Loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let device = Rc::new(device);
    let queue = Rc::new(queue);
    let surface = Rc::new(surface);
    
    // Simple camera state
    let mut camera_angle: f32 = 0.0;

    let window_clone = window.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        // Update Camera
        camera_angle += 0.005;
        let view_pos = Vec3::new(camera_angle.sin() * 10.0, 0.0, camera_angle.cos() * 10.0);
        let view_dir = -view_pos.normalize();
        
        // Update Streamer
        streamer.borrow().update(
            view_pos,
            view_dir,
            std::f32::consts::PI / 3.0 
        );

        // Render
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return, // Skip frame on error
        };
        
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.1, // Dark blue background
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            // In a real implementation, we would draw the boid/star buffers here
            // using data from streamer.borrow().store
        }

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        // Request next frame
        window_clone
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }));

    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");

    Ok(())
}
