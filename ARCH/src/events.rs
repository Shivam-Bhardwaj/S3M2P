//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: events.rs | ARCH/src/events.rs
//! PURPOSE: Event handling with delegation pattern for ARCH file explorer
//! MODIFIED: 2025-12-09
//! LAYER: ARCH (architecture explorer)
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::{render, LineAction, APP};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, MouseEvent};

pub fn setup_events(container_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let container = document
        .get_element_by_id(container_id)
        .ok_or("Container not found")?;

    // Click delegation
    {
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            if let Some(target) = event.target() {
                if let Ok(element) = target.dyn_into::<Element>() {
                    // Find closest .tree-line or button ancestor
                    if let Some(line_el) = find_closest(&element, "tree-line") {
                        handle_line_click(&line_el);
                    } else if let Some(button_el) = find_closest(&element, "file-viewer__close") {
                        handle_close_click(&button_el);
                    } else if let Some(button_el) = find_closest(&element, "file-viewer__nav-btn") {
                        handle_nav_click(&button_el);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        container.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Resize handler
    {
        let closure = Closure::wrap(Box::new(move || {
            APP.with(|app| {
                if let Some(ref mut state) = *app.borrow_mut() {
                    state.handle_resize();
                }
            });
            render();
        }) as Box<dyn FnMut()>);

        window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Keyboard navigation
    {
        use crate::ViewMode;
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            APP.with(|app| {
                if let Some(ref mut state) = *app.borrow_mut() {
                    // Only handle arrow keys when file viewer is open
                    if matches!(state.view_mode, ViewMode::FileViewer { .. }) {
                        match event.key().as_str() {
                            "ArrowLeft" => {
                                state.navigate(&LineAction::PreviousFile);
                                render();
                                event.prevent_default();
                            }
                            "ArrowRight" => {
                                state.navigate(&LineAction::NextFile);
                                render();
                                event.prevent_default();
                            }
                            _ => {}
                        }
                    }
                }
            });
        }) as Box<dyn FnMut(_)>);

        window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

fn find_closest(element: &Element, class_name: &str) -> Option<Element> {
    let mut current = Some(element.clone());

    while let Some(el) = current {
        if el.class_list().contains(class_name) {
            return Some(el);
        }
        current = el.parent_element();
    }

    None
}

fn handle_line_click(line_el: &Element) {
    let action = line_el
        .get_attribute("data-action")
        .unwrap_or_default();

    APP.with(|app| {
        if let Some(ref mut state) = *app.borrow_mut() {
            match action.as_str() {
                "back" => state.navigate(&LineAction::Back),
                "folder" => {
                    if let Some(target) = line_el.get_attribute("data-target") {
                        state.navigate(&LineAction::EnterFolder(target));
                    }
                }
                "file" => {
                    if let Some(path) = line_el.get_attribute("data-path") {
                        state.navigate(&LineAction::SelectFile(path));
                    }
                }
                _ => {}
            }
        }
    });

    render();
}

fn handle_close_click(_button_el: &Element) {
    APP.with(|app| {
        if let Some(ref mut state) = *app.borrow_mut() {
            state.close_file_viewer();
        }
    });

    render();
}

fn handle_nav_click(button_el: &Element) {
    let action = button_el
        .get_attribute("data-action")
        .unwrap_or_default();

    APP.with(|app| {
        if let Some(ref mut state) = *app.borrow_mut() {
            match action.as_str() {
                "next-file" => state.navigate(&LineAction::NextFile),
                "previous-file" => state.navigate(&LineAction::PreviousFile),
                _ => {}
            }
        }
    });

    render();
}
