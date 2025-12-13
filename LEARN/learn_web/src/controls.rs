//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: controls.rs | LEARN/learn_web/src/controls.rs
//! PURPOSE: Control binding utilities for sliders and buttons
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_web
//! ═══════════════════════════════════════════════════════════════════════════════

use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlInputElement};

/// Wire a range input (slider) to a callback
///
/// The callback receives the new value whenever the input changes.
pub fn wire_range<F>(id: &str, mut callback: F) -> Result<(), JsValue>
where
    F: FnMut(f32) + 'static,
{
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let element = document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", id)))?;

    let input: HtmlInputElement = element.dyn_into()?;

    // Clone the ID to move into the closure
    let id_owned = id.to_string();

    let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
        if let Ok(input) = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&id_owned)
            .unwrap()
            .dyn_into::<HtmlInputElement>()
        {
            if let Ok(value) = input.value().parse::<f32>() {
                callback(value);
            }
        }
    }) as Box<dyn FnMut(_)>);

    input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

/// Wire a button to a callback
pub fn wire_button<F>(id: &str, mut callback: F) -> Result<(), JsValue>
where
    F: FnMut() + 'static,
{
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let element = document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Button '{}' not found", id)))?;

    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
        e.prevent_default();
        callback();
    }) as Box<dyn FnMut(_)>);

    element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

/// Wire all controls in a container using data-param attributes
///
/// Elements with `data-param="name"` will be wired to call `on_change(name, value)`.
pub fn wire_param_controls<F>(container_id: &str, on_change: F) -> Result<(), JsValue>
where
    F: FnMut(&str, f32) + Clone + 'static,
{
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let container = document
        .get_element_by_id(container_id)
        .ok_or_else(|| JsValue::from_str(&format!("Container '{}' not found", container_id)))?;

    // Find all elements with data-param attribute using query_selector_all on Document
    let inputs = document.query_selector_all(&format!("#{} [data-param]", container_id))?;

    for i in 0..inputs.length() {
        if let Some(node) = inputs.get(i) {
            let element: Element = node.dyn_into()?;
            if let Some(param_name) = element.get_attribute("data-param") {
                let param_name_clone = param_name.clone();
                let mut on_change_clone = on_change.clone();

                let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    if let Some(el) = document.query_selector(&format!("[data-param='{}']", param_name_clone)).ok().flatten() {
                        if let Ok(input) = el.dyn_into::<HtmlInputElement>() {
                            if let Ok(value) = input.value().parse::<f32>() {
                                on_change_clone(&param_name_clone, value);
                            }
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                element.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
                closure.forget();
            }
        }
    }

    // Drop container since we don't need it after querying
    let _ = container;

    Ok(())
}

/// Set the value of a range input
pub fn set_range_value(id: &str, value: f32) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let element = document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", id)))?;

    let input: HtmlInputElement = element.dyn_into()?;
    input.set_value(&value.to_string());

    Ok(())
}

/// Get the current value of a range input
pub fn get_range_value(id: &str) -> Result<f32, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let element = document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", id)))?;

    let input: HtmlInputElement = element.dyn_into()?;
    input
        .value()
        .parse::<f32>()
        .map_err(|_| JsValue::from_str("Invalid number"))
}
