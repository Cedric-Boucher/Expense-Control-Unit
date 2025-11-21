use std::{env, net::SocketAddr};

use axum::{Extension, Router};
use sqlx::migrate::Migrator;
use tower_cookies::CookieManagerLayer;
use tower_http::compression::{CompressionLayer, CompressionLevel, predicate::{Predicate, DefaultPredicate, SizeAbove}};
use dotenv::dotenv;

mod routes;
mod db;
mod models;
mod passwords;
mod time_conversion;
mod middleware;

use routes::{me, transactions, signup, login, logout, categories, import};
use db::init_db_pool;

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() {
    dotenv().ok();

    const MIN_COMPRESSION_SIZE: u16 = 1000; // bytes
    let compression_predicate = Predicate::and(
        DefaultPredicate::new(),
        SizeAbove::new(MIN_COMPRESSION_SIZE)
    );
    let compression_level = CompressionLevel::Default;

    let db = init_db_pool().await;
    MIGRATOR.run(&db).await.expect("Failed to run migrations");

    let api_routes = Router::new()
        .merge(me::routes())
        .merge(transactions::routes())
        .merge(categories::routes())
        .merge(signup::routes())
        .merge(login::routes())
        .merge(logout::routes())
        .merge(import::routes());

    let app = Router::new()
        .merge(api_routes)
        .layer(Extension(db))
        .layer(CookieManagerLayer::new())
        .layer(
            CompressionLayer::new()
                .quality(compression_level)
                .compress_when(compression_predicate)
        );

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("✅ Backend Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
