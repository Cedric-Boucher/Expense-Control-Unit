use std::{env, net::SocketAddr};

use axum::{Extension, Router};
use tower_http::cors::{CorsLayer, Any};
use dotenv::dotenv;

mod routes;
mod db;
mod models;
use routes::{health, transactions};
use db::init_db_pool;


#[tokio::main]
async fn main() {
    dotenv().ok();

    let db = init_db_pool().await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(health::routes())
        .merge(transactions::routes())
        .layer(Extension(db))
        .layer(cors);

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("✅ Backend Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
