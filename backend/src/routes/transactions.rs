use axum::{response::IntoResponse, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use crate::{middleware::AuthSession, models::transaction::{NewTransaction, Transaction}, time_conversion::{convert_chrono_to_time, convert_time_to_chrono}};
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};

pub fn routes() -> Router {
    Router::new().route("/api/transactions", get(list_transactions).post(create_transaction))
}

async fn list_transactions(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
) -> impl IntoResponse {
    let rows: Vec<Transaction> = sqlx::query!(
        r#"
        SELECT id, description, amount, created_at
        FROM transactions
        WHERE user_id = $1
        ORDER BY id DESC
        "#,
        user.id
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
    AuthSession(user): AuthSession,
    Json(payload): Json<NewTransaction>,
) -> Json<Transaction> {
    let record = sqlx::query!(
        r#"
        INSERT INTO transactions (user_id, description, amount, created_at)
        VALUES ($1, $2, $3, COALESCE($4, now()))
        RETURNING id, description, amount, created_at
        "#,
        user.id,
        payload.description,
        BigDecimal::from_f64(payload.amount),
        payload.created_at.map(convert_chrono_to_time)
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert transaction");

    let result = Transaction {
        id: record.id,
        description: record.description,
        amount: record.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(record.created_at)
    };

    Json(result)
}
