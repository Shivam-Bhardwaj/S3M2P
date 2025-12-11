//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: main.rs | HELIOS/server/src/main.rs
//! PURPOSE: Axum HTTP server for on-demand star tile streaming (Google Earth-style)
//! LAYER: HELIOS Server
//! ═══════════════════════════════════════════════════════════════════════════════

mod db;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use dna::spatial::SpatialKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

use db::{StarData, StarDatabase};

/// Server state shared across handlers
#[derive(Clone)]
struct AppState {
    db: Arc<StarDatabase>,
}

/// Tile data response (sent to WASM client)
#[derive(Debug, Serialize, Deserialize)]
struct TileData {
    face: u8,
    level: u8,
    x: u32,
    y: u32,
    stars: Vec<StarEntry>,
}

/// Compact star entry for network transfer
#[derive(Debug, Serialize, Deserialize)]
struct StarEntry {
    hip_id: i32,
    name: String,
    ra: f64,
    dec: f64,
    magnitude: f64,
    color_bv: f64,
    constellation: String,
}

impl From<StarData> for StarEntry {
    fn from(s: StarData) -> Self {
        Self {
            hip_id: s.hip_id,
            name: s.name,
            ra: s.ra,
            dec: s.dec,
            magnitude: s.magnitude,
            color_bv: s.color_bv,
            constellation: s.constellation,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting HELIOS streaming server...");

    // Get database URL from environment or use default
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let default_url = "postgresql://root:iouiouiou@144.126.145.3/stars";
        warn!("DATABASE_URL not set, using default: {}", default_url);
        default_url.to_string()
    });

    // Connect to database
    let db: Arc<StarDatabase> = Arc::new(StarDatabase::new(&db_url).await?);
    info!("Database connection established");

    // Test database connection
    db.health_check().await?;
    info!("Database health check passed");

    let state = AppState { db };

    // Build router
    let app = Router::new()
        .route("/api/tiles/stars/:face/:level/:x/:y", get(get_tile))
        .route("/api/health", get(health_check))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers(tower_http::cors::Any),
        )
        .with_state(state);

    // Bind server
    let addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8090".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server listening on http://{}", addr);
    info!("Endpoints:");
    info!("  GET /api/tiles/stars/{{face}}/{{level}}/{{x}}/{{y}}");
    info!("  GET /api/health");

    axum::serve(listener, app).await?;

    Ok(())
}

/// GET /api/tiles/stars/:face/:level/:x/:y
///
/// Query database for stars within the specified spatial tile
async fn get_tile(
    State(state): State<AppState>,
    Path((face, level, x, y)): Path<(u8, u8, u32, u32)>,
) -> Result<Json<TileData>, AppError> {
    // Construct SpatialKey from URL params
    let key = SpatialKey::new(face, level, x, y);

    info!(
        "Request: tile face={} level={} coords=({},{})",
        face, level, x, y
    );

    // Query database
    let stars = state.db.query_tile(key).await?;

    info!("Returning {} stars for tile", stars.len());

    // Convert to response
    Ok(Json(TileData {
        face,
        level,
        x,
        y,
        stars: stars.into_iter().map(|s| s.into()).collect(),
    }))
}

/// GET /api/health
///
/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    state.db.health_check().await?;

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        database: "connected".to_string(),
    }))
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    database: String,
}

/// Error handling
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        warn!("Request error: {}", self.0);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            Json(serde_json::json!({
                "error": self.0.to_string()
            })),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
