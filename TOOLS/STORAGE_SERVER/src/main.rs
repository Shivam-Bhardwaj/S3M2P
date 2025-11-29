use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;
use tower_http::cors::CorsLayer;
use tracing::info;
// Reserved for future batch endpoint implementation
#[allow(unused_imports)]
use dna::spatial::{DataLayer, SpatialKey};

#[derive(Clone)]
struct AppState {
    data_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let data_path = PathBuf::from(data_dir);

    if !data_path.exists() {
        fs::create_dir_all(&data_path).await?;
    }

    let state = AppState {
        data_dir: data_path,
    };

    // CORS: Allow everything for now (Development)
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route("/v1/health", get(health_check))
        .route("/v1/chunk/:layer/:face/:level/:x/:y", get(get_chunk))
        .route("/v1/batch", post(batch_get_chunks))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[derive(Deserialize)]
struct ChunkParams {
    layer: String,
    face: u8,
    level: u8,
    x: u32,
    y: u32,
}

async fn get_chunk(State(state): State<AppState>, Path(params): Path<ChunkParams>) -> Response {
    // Construct path: data/{layer}/{face}/{level}/{x}_{y}.bin
    let mut path = state.data_dir.clone();
    path.push(&params.layer);
    path.push(params.face.to_string());
    path.push(params.level.to_string());
    // Using a sharded directory structure or flat file?
    // For millions of files, we want to avoid huge directories.
    // Maybe x/y.bin?
    path.push(params.x.to_string());
    path.push(format!("{}.bin", params.y));

    match fs::read(&path).await {
        Ok(data) => (StatusCode::OK, data).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BatchRequest {
    layer: String,
    keys: Vec<u64>, // Raw SpatialKey values
}

// Simple multipart-like response for batching could be implemented here
// For now, just returning a JSON array of base64 or similar?
// Binary streaming is better for "fastest".
// Let's keep it simple for the prototype: A stream of length-prefixed chunks.
async fn batch_get_chunks(
    State(_state): State<AppState>,
    Json(_req): Json<BatchRequest>,
) -> impl IntoResponse {
    // TODO: Implement efficient batch retrieval
    StatusCode::NOT_IMPLEMENTED
}
