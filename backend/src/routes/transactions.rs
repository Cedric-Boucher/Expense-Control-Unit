use axum::{routing::get, Extension, Json, Router};
use chrono::{DateTime, TimeZone, Utc};
use sqlx::{types::time::OffsetDateTime, PgPool};
use crate::models::transaction::{NewTransaction, Transaction};
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};

pub fn routes() -> Router {
    Router::new().route("/api/transactions", get(list_transactions).post(create_transaction))
}

async fn list_transactions(Extension(pool): Extension<PgPool>) -> Json<Vec<Transaction>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, description, amount, created_at
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
        created_at: convert_time_to_chrono(row.created_at)
    })
    .collect();

    Json(rows)
}

pub async fn create_transaction(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<NewTransaction>,
) -> Json<Transaction> {
    let record = sqlx::query!(
        r#"
        INSERT INTO transactions (description, amount)
        VALUES ($1, $2)
        RETURNING id, description, amount, created_at
        "#,
        payload.description,
        BigDecimal::from_f64(payload.amount)
    )
    .fetch_one(&pool)
    .await
    .expect("Failed te insert transaction");

    let result = Transaction {
        id: record.id,
        description: record.description,
        amount: record.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(record.created_at)
    };

    Json(result)
}

fn convert_time_to_chrono(dt: OffsetDateTime) -> DateTime<Utc> {
    let timestamp = dt.unix_timestamp();
    let nanosecond = dt.nanosecond();

    Utc.timestamp_opt(timestamp, nanosecond).unwrap()
}
