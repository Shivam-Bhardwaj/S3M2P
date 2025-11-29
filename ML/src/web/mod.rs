pub mod comments;

use axum::{
    routing::{get, post},
    Router,
    response::{Html, Json},
    extract::{State, Path},
};
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use serde::Deserialize;

#[derive(Clone)]
struct AppState {
    comment_store: comments::CommentStore,
}

#[derive(Deserialize)]
struct NewCommentRequest {
    author: String,
    content: String,
    lesson_id: String,
}

pub async fn start_server() {
    let comment_store = comments::CommentStore::new("comments.json");
    let state = AppState { comment_store };

    // Build our application with a route
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/comments/{lesson_id}", get(get_comments_handler))
        .route("/api/comments", post(add_comment_handler))
        .nest_service("/static", ServeDir::new(".")) // Serve current directory for images
        .with_state(state);

    // Address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> Html<String> {
    // Read from disk for hot-reload during development
    let html = std::fs::read_to_string("src/web/index.html")
        .unwrap_or_else(|_| include_str!("index.html").to_string());
    Html(html)
}

async fn get_comments_handler(
    State(state): State<AppState>,
    Path(lesson_id): Path<String>,
) -> Json<Vec<comments::Comment>> {
    let comments = state.comment_store.get_comments(&lesson_id);
    Json(comments)
}

async fn add_comment_handler(
    State(state): State<AppState>,
    Json(payload): Json<NewCommentRequest>,
) -> Json<comments::Comment> {
    let comment = state.comment_store.add_comment(
        payload.lesson_id, 
        payload.author, 
        payload.content
    );
    Json(comment)
}
