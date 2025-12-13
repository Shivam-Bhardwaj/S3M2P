//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: dom.rs | LEARN/learn_web/src/dom.rs
//! PURPOSE: DOM utility functions
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_web
//! ═══════════════════════════════════════════════════════════════════════════════

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;

/// Get an element by ID and cast to a specific type
pub fn get_element<T: JsCast>(id: &str) -> Result<T, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", id)))?
        .dyn_into::<T>()
        .map_err(|_| JsValue::from_str(&format!("Element '{}' has wrong type", id)))
}

/// Get an element by ID as generic Element
pub fn get_element_by_id(id: &str) -> Result<Element, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", id)))
}

/// Set the text content of an element
pub fn set_text(id: &str, text: &str) -> Result<(), JsValue> {
    let element = get_element_by_id(id)?;
    element.set_text_content(Some(text));
    Ok(())
}

/// Set the inner HTML of an element
pub fn set_html(id: &str, html: &str) -> Result<(), JsValue> {
    let element = get_element_by_id(id)?;
    element.set_inner_html(html);
    Ok(())
}

/// Set an attribute on an element
pub fn set_attribute(id: &str, name: &str, value: &str) -> Result<(), JsValue> {
    let element = get_element_by_id(id)?;
    element.set_attribute(name, value)
}

/// Remove an attribute from an element
pub fn remove_attribute(id: &str, name: &str) -> Result<(), JsValue> {
    let element = get_element_by_id(id)?;
    element.remove_attribute(name)
}

/// Add a class to an element
pub fn add_class(id: &str, class: &str) -> Result<(), JsValue> {
    let element = get_element_by_id(id)?;
    element.class_list().add_1(class)
}

/// Remove a class from an element
pub fn remove_class(id: &str, class: &str) -> Result<(), JsValue> {
    let element = get_element_by_id(id)?;
    element.class_list().remove_1(class)
}

/// Toggle a class on an element
pub fn toggle_class(id: &str, class: &str) -> Result<bool, JsValue> {
    let element = get_element_by_id(id)?;
    element.class_list().toggle(class)
}

/// Show an element (remove hidden/display:none)
pub fn show(id: &str) -> Result<(), JsValue> {
    let element: web_sys::HtmlElement = get_element(id)?;
    element.style().set_property("display", "")
}

/// Hide an element (set display:none)
pub fn hide(id: &str) -> Result<(), JsValue> {
    let element: web_sys::HtmlElement = get_element(id)?;
    element.style().set_property("display", "none")
}

/// Set visibility of an element
pub fn set_visible(id: &str, visible: bool) -> Result<(), JsValue> {
    if visible {
        show(id)
    } else {
        hide(id)
    }
}

/// Create a new element
pub fn create_element(tag: &str) -> Result<Element, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    document.create_element(tag)
}

/// Append a child element to a parent
pub fn append_child(parent_id: &str, child: &Element) -> Result<(), JsValue> {
    let parent = get_element_by_id(parent_id)?;
    parent.append_child(child)?;
    Ok(())
}

/// Query selector on document
pub fn query_selector(selector: &str) -> Result<Option<Element>, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    document.query_selector(selector)
}
