use axum::{http::StatusCode, response::IntoResponse};

use axum::{Router, routing::get};

pub fn routes() -> Router {
    Router::new().route("/api/health", get(health_check))
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Backend is healthy")
}
