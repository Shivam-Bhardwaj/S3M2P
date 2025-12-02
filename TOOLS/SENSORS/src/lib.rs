use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, CanvasRenderingContext2d, DeviceOrientationEvent,
    HtmlCanvasElement, HtmlVideoElement, MediaStreamConstraints,
};

const GRAPH_HISTORY_SIZE: usize = 200;

#[derive(Default)]
#[allow(dead_code)]
struct AccelData {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Default)]
#[allow(dead_code)]
struct OrientData {
    alpha: f64,
    beta: f64,
    gamma: f64,
}

struct SensorState {
    accel_history_x: Vec<f64>,
    accel_history_y: Vec<f64>,
    accel_history_z: Vec<f64>,
    current_accel: AccelData,
    current_orient: OrientData,
    sensors_available: bool,
    camera_available: bool,
}

impl SensorState {
    fn new() -> Self {
        Self {
            accel_history_x: vec![0.0; GRAPH_HISTORY_SIZE],
            accel_history_y: vec![0.0; GRAPH_HISTORY_SIZE],
            accel_history_z: vec![0.0; GRAPH_HISTORY_SIZE],
            current_accel: AccelData::default(),
            current_orient: OrientData::default(),
            sensors_available: false,
            camera_available: false,
        }
    }

    fn push_accel(&mut self, x: f64, y: f64, z: f64) {
        self.current_accel = AccelData { x, y, z };

        self.accel_history_x.remove(0);
        self.accel_history_x.push(x);

        self.accel_history_y.remove(0);
        self.accel_history_y.push(y);

        self.accel_history_z.remove(0);
        self.accel_history_z.push(z);
    }
}

thread_local! {
    static STATE: RefCell<SensorState> = RefCell::new(SensorState::new());
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    // Check for sensor support and set up
    check_sensor_support();

    // Check for camera support
    check_camera_support();

    // Set up camera button
    setup_camera_button();

    // Set up sensor permission button (for iOS)
    setup_sensor_button();

    // Start render loop
    start_render_loop();

    // Log startup
    web_sys::console::log_1(&"Sensor test initialized".into());
}

fn check_sensor_support() {
    let window = window().expect("no window");

    // Check if DeviceMotionEvent exists
    let has_motion = js_sys::Reflect::get(&window, &"DeviceMotionEvent".into())
        .map(|v| !v.is_undefined())
        .unwrap_or(false);

    let has_orientation = js_sys::Reflect::get(&window, &"DeviceOrientationEvent".into())
        .map(|v| !v.is_undefined())
        .unwrap_or(false);

    if has_motion || has_orientation {
        // Check if we need permission (iOS 13+)
        let needs_permission = check_needs_motion_permission();

        if needs_permission {
            // Show permission button
            show_sensor_permission_button();
            update_sensor_status("waiting", "Permission Required");
        } else {
            // Try to listen directly
            setup_motion_listeners();
        }
    } else {
        update_sensor_status("unavailable", "Not Supported");
        STATE.with(|s| s.borrow_mut().sensors_available = false);
    }
}

fn check_needs_motion_permission() -> bool {
    let window = window().expect("no window");

    // Check if DeviceMotionEvent.requestPermission exists (iOS 13+)
    if let Ok(motion_event) = js_sys::Reflect::get(&window, &"DeviceMotionEvent".into()) {
        if let Ok(request_perm) = js_sys::Reflect::get(&motion_event, &"requestPermission".into()) {
            return request_perm.is_function();
        }
    }
    false
}

fn show_sensor_permission_button() {
    let document = window().unwrap().document().unwrap();
    if let Some(btn) = document.get_element_by_id("sensor-btn") {
        let _ = btn.set_attribute("style", "display: block;");
    }
}

fn setup_sensor_button() {
    let document = window().unwrap().document().unwrap();

    if let Some(btn) = document.get_element_by_id("sensor-btn") {
        let closure = Closure::wrap(Box::new(move || {
            request_motion_permission();
        }) as Box<dyn Fn()>);

        let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

fn request_motion_permission() {
    let window = window().expect("no window");

    // iOS 13+ permission request
    if let Ok(motion_event) = js_sys::Reflect::get(&window, &"DeviceMotionEvent".into()) {
        if let Ok(request_perm) = js_sys::Reflect::get(&motion_event, &"requestPermission".into()) {
            if request_perm.is_function() {
                let func: js_sys::Function = request_perm.unchecked_into();
                if let Ok(promise) = func.call0(&motion_event) {
                    let promise: js_sys::Promise = promise.unchecked_into();

                    let on_granted = Closure::wrap(Box::new(move |result: JsValue| {
                        let result_str = result.as_string().unwrap_or_default();
                        if result_str == "granted" {
                            setup_motion_listeners();
                            hide_sensor_permission_button();
                        } else {
                            update_sensor_status("error", "Permission Denied");
                        }
                    }) as Box<dyn FnMut(JsValue)>);

                    let on_error = Closure::wrap(Box::new(move |_: JsValue| {
                        update_sensor_status("error", "Permission Error");
                    }) as Box<dyn FnMut(JsValue)>);

                    let _ = promise.then2(&on_granted, &on_error);
                    on_granted.forget();
                    on_error.forget();
                }
            }
        }
    }
}

fn hide_sensor_permission_button() {
    let document = window().unwrap().document().unwrap();
    if let Some(btn) = document.get_element_by_id("sensor-btn") {
        let _ = btn.set_attribute("style", "display: none;");
    }
}

fn setup_motion_listeners() {
    let window = window().expect("no window");

    // Device Motion (accelerometer) - use JsValue since typed accessors aren't fully available
    let motion_closure = Closure::wrap(Box::new(move |event: JsValue| {
        // Access accelerationIncludingGravity via js_sys::Reflect
        if let Ok(accel) = js_sys::Reflect::get(&event, &"accelerationIncludingGravity".into()) {
            if !accel.is_null() && !accel.is_undefined() {
                let x = js_sys::Reflect::get(&accel, &"x".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let y = js_sys::Reflect::get(&accel, &"y".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let z = js_sys::Reflect::get(&accel, &"z".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                STATE.with(|s| {
                    let mut state = s.borrow_mut();
                    state.push_accel(x, y, z);
                    state.sensors_available = true;
                });

                update_accel_display(x, y, z);
            }
        }
    }) as Box<dyn Fn(JsValue)>);

    let _ = window.add_event_listener_with_callback(
        "devicemotion",
        motion_closure.as_ref().unchecked_ref(),
    );
    motion_closure.forget();

    // Device Orientation
    let orientation_closure = Closure::wrap(Box::new(move |event: DeviceOrientationEvent| {
        let alpha = event.alpha().unwrap_or(0.0);
        let beta = event.beta().unwrap_or(0.0);
        let gamma = event.gamma().unwrap_or(0.0);

        STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.current_orient = OrientData { alpha, beta, gamma };
        });

        update_orientation_display(alpha, beta, gamma);
    }) as Box<dyn Fn(DeviceOrientationEvent)>);

    let _ = window.add_event_listener_with_callback(
        "deviceorientation",
        orientation_closure.as_ref().unchecked_ref(),
    );
    orientation_closure.forget();

    update_sensor_status("available", "Active");
    STATE.with(|s| s.borrow_mut().sensors_available = true);
}

fn check_camera_support() {
    let window = window().expect("no window");
    let navigator = window.navigator();

    // Check if mediaDevices exists
    if let Some(_) = navigator.media_devices().ok() {
        update_camera_status("waiting", "Ready");
        STATE.with(|s| s.borrow_mut().camera_available = true);
    } else {
        update_camera_status("unavailable", "Not Supported");
        update_camera_placeholder("Camera not supported on this device");
        STATE.with(|s| s.borrow_mut().camera_available = false);
    }
}

fn setup_camera_button() {
    let document = window().unwrap().document().unwrap();

    if let Some(btn) = document.get_element_by_id("camera-btn") {
        let closure = Closure::wrap(Box::new(move || {
            wasm_bindgen_futures::spawn_local(async {
                request_camera().await;
            });
        }) as Box<dyn Fn()>);

        let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

async fn request_camera() {
    let window = window().expect("no window");
    let navigator = window.navigator();

    if let Ok(media_devices) = navigator.media_devices() {
        let constraints = MediaStreamConstraints::new();
        constraints.set_video(&JsValue::TRUE);
        constraints.set_audio(&JsValue::FALSE);

        match media_devices.get_user_media_with_constraints(&constraints) {
            Ok(promise) => {
                match JsFuture::from(promise).await {
                    Ok(stream) => {
                        let stream: web_sys::MediaStream = stream.unchecked_into();

                        let document = window.document().unwrap();
                        if let Some(video) = document.get_element_by_id("camera-video") {
                            let video: HtmlVideoElement = video.unchecked_into();
                            video.set_src_object(Some(&stream));
                            let _ = video.play();
                        }

                        // Hide placeholder via style
                        if let Some(placeholder) = document.get_element_by_id("camera-placeholder") {
                            let _ = placeholder.set_attribute("style", "display: none;");
                        }

                        update_camera_status("available", "Streaming");
                    }
                    Err(_) => {
                        update_camera_status("error", "Permission Denied");
                        update_camera_placeholder("Camera permission denied");
                    }
                }
            }
            Err(_) => {
                update_camera_status("error", "Error");
                update_camera_placeholder("Failed to access camera");
            }
        }
    }
}

fn update_camera_placeholder(message: &str) {
    let document = window().unwrap().document().unwrap();
    if let Some(text) = document.get_element_by_id("camera-status-text") {
        text.set_text_content(Some(message));
    }
}

fn update_accel_display(x: f64, y: f64, z: f64) {
    let document = window().unwrap().document().unwrap();

    if let Some(el) = document.get_element_by_id("accel-x") {
        el.set_text_content(Some(&format!("{:.2}", x)));
    }
    if let Some(el) = document.get_element_by_id("accel-y") {
        el.set_text_content(Some(&format!("{:.2}", y)));
    }
    if let Some(el) = document.get_element_by_id("accel-z") {
        el.set_text_content(Some(&format!("{:.2}", z)));
    }
}

fn update_orientation_display(alpha: f64, beta: f64, gamma: f64) {
    let document = window().unwrap().document().unwrap();

    if let Some(el) = document.get_element_by_id("orient-alpha") {
        el.set_text_content(Some(&format!("{:.0}°", alpha)));
    }
    if let Some(el) = document.get_element_by_id("orient-beta") {
        el.set_text_content(Some(&format!("{:.0}°", beta)));
    }
    if let Some(el) = document.get_element_by_id("orient-gamma") {
        el.set_text_content(Some(&format!("{:.0}°", gamma)));
    }
}

fn update_sensor_status(status: &str, text: &str) {
    let document = window().unwrap().document().unwrap();

    if let Some(dot) = document.get_element_by_id("sensor-dot") {
        let _ = dot.set_attribute("class", &format!("status-dot {}", status));
    }
    if let Some(el) = document.get_element_by_id("sensor-status") {
        el.set_text_content(Some(text));
        let _ = el.set_attribute("class", &format!("status-text {}", status));
    }
}

fn update_camera_status(status: &str, text: &str) {
    let document = window().unwrap().document().unwrap();

    if let Some(dot) = document.get_element_by_id("camera-dot") {
        let _ = dot.set_attribute("class", &format!("status-dot {}", status));
    }
    if let Some(el) = document.get_element_by_id("camera-status") {
        el.set_text_content(Some(text));
        let _ = el.set_attribute("class", &format!("status-text {}", status));
    }
}

fn start_render_loop() {
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        render_graph();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

fn render_graph() {
    let document = window().unwrap().document().unwrap();

    let canvas: HtmlCanvasElement = match document.get_element_by_id("accel-graph") {
        Some(el) => el.unchecked_into(),
        None => return,
    };

    // Get canvas element dimensions via computed style or direct attributes
    let width = canvas.client_width() as u32;
    let height = canvas.client_height() as u32;

    if width == 0 || height == 0 {
        return;
    }

    // Set canvas internal resolution
    let dpr = window().unwrap().device_pixel_ratio();
    canvas.set_width((width as f64 * dpr) as u32);
    canvas.set_height((height as f64 * dpr) as u32);

    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .unchecked_into();

    let _ = ctx.scale(dpr, dpr);

    let w = width as f64;
    let h = height as f64;

    // Clear
    ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.3)"));
    ctx.fill_rect(0.0, 0.0, w, h);

    // Draw grid
    ctx.set_stroke_style(&JsValue::from_str("rgba(0, 255, 255, 0.1)"));
    ctx.set_line_width(1.0);

    // Horizontal center line
    ctx.begin_path();
    ctx.move_to(0.0, h / 2.0);
    ctx.line_to(w, h / 2.0);
    ctx.stroke();

    // Draw data
    STATE.with(|s| {
        let state = s.borrow();

        let scale = h / 40.0; // ±20 m/s² range
        let center = h / 2.0;
        let step = w / (GRAPH_HISTORY_SIZE as f64 - 1.0);

        // X axis (red)
        draw_line(&ctx, &state.accel_history_x, step, center, scale, "#ff6b6b");

        // Y axis (teal)
        draw_line(&ctx, &state.accel_history_y, step, center, scale, "#4ecdc4");

        // Z axis (yellow)
        draw_line(&ctx, &state.accel_history_z, step, center, scale, "#ffe66d");
    });

    // Legend
    ctx.set_font("12px 'JetBrains Mono', monospace");

    ctx.set_fill_style(&JsValue::from_str("#ff6b6b"));
    let _ = ctx.fill_text("X", 10.0, 16.0);

    ctx.set_fill_style(&JsValue::from_str("#4ecdc4"));
    let _ = ctx.fill_text("Y", 30.0, 16.0);

    ctx.set_fill_style(&JsValue::from_str("#ffe66d"));
    let _ = ctx.fill_text("Z", 50.0, 16.0);
}

fn draw_line(ctx: &CanvasRenderingContext2d, data: &[f64], step: f64, center: f64, scale: f64, color: &str) {
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(2.0);
    ctx.begin_path();

    for (i, &val) in data.iter().enumerate() {
        let x = i as f64 * step;
        let y = center - val * scale;

        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }

    ctx.stroke();
}
