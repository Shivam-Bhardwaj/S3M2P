//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: animation.rs | LEARN/learn_web/src/animation.rs
//! PURPOSE: Animation loop using requestAnimationFrame
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_web
//! ═══════════════════════════════════════════════════════════════════════════════

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Animation loop using requestAnimationFrame
///
/// Provides start/stop/pause controls and calls a callback each frame
/// with the delta time since the last frame.
pub struct AnimationLoop {
    callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    running: Rc<RefCell<bool>>,
    paused: Rc<RefCell<bool>>,
    frame_id: Rc<RefCell<i32>>,
}

impl AnimationLoop {
    /// Create a new animation loop
    ///
    /// The callback receives delta time in seconds since the last frame.
    pub fn new<F>(mut on_frame: F) -> Self
    where
        F: FnMut(f64) + 'static,
    {
        let callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let running = Rc::new(RefCell::new(false));
        let paused = Rc::new(RefCell::new(false));
        let frame_id = Rc::new(RefCell::new(0));

        let callback_clone = callback.clone();
        let running_clone = running.clone();
        let paused_clone = paused.clone();
        let frame_id_clone = frame_id.clone();

        let window = web_sys::window().unwrap();
        let performance = window.performance().unwrap();
        let last_time = Rc::new(RefCell::new(performance.now()));

        *callback.borrow_mut() = Some(Closure::new(move || {
            if !*running_clone.borrow() {
                return;
            }

            let now = performance.now();
            let dt = if *paused_clone.borrow() {
                0.0
            } else {
                (now - *last_time.borrow()) / 1000.0
            };
            *last_time.borrow_mut() = now;

            // Call user callback with delta time
            on_frame(dt);

            // Schedule next frame
            let id = web_sys::window()
                .unwrap()
                .request_animation_frame(
                    callback_clone
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap();
            *frame_id_clone.borrow_mut() = id;
        }));

        Self {
            callback,
            running,
            paused,
            frame_id,
        }
    }

    /// Start the animation loop
    pub fn start(&self) {
        if *self.running.borrow() {
            return;
        }

        *self.running.borrow_mut() = true;
        *self.paused.borrow_mut() = false;

        let id = web_sys::window()
            .unwrap()
            .request_animation_frame(
                self.callback
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
        *self.frame_id.borrow_mut() = id;
    }

    /// Stop the animation loop
    pub fn stop(&self) {
        *self.running.borrow_mut() = false;
        let _ = web_sys::window()
            .unwrap()
            .cancel_animation_frame(*self.frame_id.borrow());
    }

    /// Pause the animation (loop continues but dt is 0)
    pub fn pause(&self) {
        *self.paused.borrow_mut() = true;
    }

    /// Resume the animation
    pub fn resume(&self) {
        *self.paused.borrow_mut() = false;
    }

    /// Toggle pause state
    pub fn toggle_pause(&self) {
        let mut paused = self.paused.borrow_mut();
        *paused = !*paused;
    }

    /// Check if animation is currently paused
    pub fn is_paused(&self) -> bool {
        *self.paused.borrow()
    }

    /// Check if animation is running
    pub fn is_running(&self) -> bool {
        *self.running.borrow()
    }
}
