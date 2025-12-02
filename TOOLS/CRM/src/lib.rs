#![allow(unexpected_cfgs)]

use wasm_bindgen::prelude::*;
use web_sys::Window;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let _window: Window = web_sys::window().expect("no global window");

    // CRM initialization will go here
    web_sys::console::log_1(&"CRM initialized".into());

    Ok(())
}

/// Contact record
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Deal/Opportunity tracking
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Deal {
    pub id: String,
    pub title: String,
    pub contact_id: String,
    pub value: f64,
    pub stage: DealStage,
    pub probability: f32,
    pub notes: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum DealStage {
    Lead,
    Qualified,
    Proposal,
    Negotiation,
    ClosedWon,
    ClosedLost,
}

/// Interaction history
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Interaction {
    pub id: String,
    pub contact_id: String,
    pub interaction_type: InteractionType,
    pub notes: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum InteractionType {
    Email,
    Call,
    Meeting,
    Note,
    Task,
}
