use std::sync::{Arc, Mutex};
use antimony_core::spatial::{SpatialStore, DataLayer};
use glam::Vec3;

// Placeholder for actual star data
#[derive(Clone, Debug)]
pub struct StarChunk {
    pub raw_data: Vec<u8>,
}

pub struct StreamController {
    store: Arc<Mutex<SpatialStore<StarChunk>>>,
    server_url: String,
}

impl StreamController {
    pub fn new(server_url: String) -> Self {
        Self {
            store: Arc::new(Mutex::new(SpatialStore::new(10))),
            server_url,
        }
    }

    pub fn update(&self, view_pos: Vec3, view_dir: Vec3, fov: f32) {
        let mut store = self.store.lock().unwrap();
        
        // Query visible keys with LOD bias 1.0
        let (loaded, missing) = store.query_visible_keys(
            view_pos, 
            view_dir, 
            fov, 
            1.0, 
            DataLayer::Stars
        );
        
        // Process missing keys
        for key in missing {
            // Spawn fetch task
            let url = format!("{}/v1/chunk/stars/{}/{}/{}/{}", 
                self.server_url, 
                key.face(), 
                key.level(), 
                key.coords().0, 
                key.coords().1
            );
            
            let store_ref = self.store.clone();
            let key_copy = key;
            
            wasm_bindgen_futures::spawn_local(async move {
                // Log fetch attempt
                web_sys::console::log_1(&format!("Fetching chunk: {:?}", key_copy).into());
                
                // Mock fetch for now or use reqwest
                match reqwest::get(&url).await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            if let Ok(bytes) = resp.bytes().await {
                                let mut store = store_ref.lock().unwrap();
                                store.insert(key_copy, vec![StarChunk { raw_data: bytes.to_vec() }]);
                                web_sys::console::log_1(&format!("Loaded chunk: {:?}", key_copy).into());
                            }
                        } else {
                            web_sys::console::warn_1(&format!("Failed to fetch chunk: {:?}", resp.status()).into());
                        }
                    },
                    Err(e) => {
                        web_sys::console::error_1(&format!("Network error: {:?}", e).into());
                    }
                }
            });
        }
        
        // Visualize loaded chunks (Debug)
        if !loaded.is_empty() {
            web_sys::console::log_1(&format!("Visible chunks: {}", loaded.len()).into());
        }
    }
}

