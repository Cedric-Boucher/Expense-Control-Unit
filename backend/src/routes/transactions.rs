use axum::{routing::get, Extension, Json, Router};
use sqlx::PgPool;
use crate::models::transaction::Transaction;
use bigdecimal::ToPrimitive;

pub fn routes() -> Router {
    Router::new().route("/api/transactions", get(list_transactions))
}

async fn list_transactions(Extension(pool): Extension<PgPool>) -> Json<Vec<Transaction>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, description, amount
        FROM transactions
        ORDER BY id DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch transactions")
    .into_iter()
    .map(|row| Transaction {
        id: row.id,
        description: row.description,
        amount: row.amount.to_f64().unwrap_or(0.0),
    })
    .collect();

    Json(rows)
}
