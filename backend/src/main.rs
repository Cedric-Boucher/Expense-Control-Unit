use std::{env, net::SocketAddr};

use axum::{http::{header, HeaderValue, Method}, Extension, Router};
use sqlx::migrate::Migrator;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use dotenv::dotenv;

mod routes;
mod db;
mod models;
mod passwords;
mod time_conversion;
mod middleware;
use routes::{me, transactions, signup, login, logout, categories};
use db::init_db_pool;

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db = init_db_pool().await;
    MIGRATOR.run(&db).await.expect("Failed to run migrations");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
            Method::PUT,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allow_credentials(true);

    let app = Router::new()
        .merge(me::routes())
        .merge(transactions::routes())
        .merge(categories::routes())
        .merge(signup::routes())
        .merge(login::routes())
        .merge(logout::routes())
        .layer(Extension(db))
        .layer(CookieManagerLayer::new())
        .layer(cors);

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("✅ Backend Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
