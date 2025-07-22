use std::{env, net::SocketAddr};

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router, Json};
use tower_http::cors::{CorsLayer, Any};
use dotenv::dotenv;
use serde::Serialize;

#[derive(Serialize)]
struct Transaction {
    id: u32,
    description: String,
    amount: i32
}

async fn list_transactions() -> Json<Vec<Transaction>> {
    let sample_data = vec![
        Transaction { id: 1, description: "Sample".into(), amount: -156 },
        Transaction { id: 2, description: "Test".into(), amount: 843 },
    ];

    Json(sample_data)
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/transactions", get(list_transactions))
        .layer(cors);

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("✅ Backend Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Backend is healthy")
}
