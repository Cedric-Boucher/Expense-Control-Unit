use axum::{response::IntoResponse, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use crate::{middleware::AuthSession, models::{category::Category, transaction::{NewTransaction, Transaction}, user::User}, time_conversion::{convert_chrono_to_time, convert_time_to_chrono}};
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};
use futures::future::join_all;

pub fn routes() -> Router {
    Router::new().route("/api/transactions", get(list_transactions).post(create_transaction))
}

async fn list_transactions(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
) -> impl IntoResponse {
    let futures = sqlx::query!(
        r#"
        SELECT id, category_id, description, amount, created_at
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
    .map(async |row| Transaction {
        id: row.id,
        category: fetch_category(axum::Extension(pool.clone()), &user, row.category_id).await,
        description: row.description,
        amount: row.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(row.created_at)
    });

    let rows: Vec<Transaction> = join_all(futures).await;

    Json(rows)
}

pub async fn create_transaction(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<NewTransaction>,
) -> Json<Transaction> {
    let record = sqlx::query!(
        r#"
        INSERT INTO transactions (user_id, category_id, description, amount, created_at)
        VALUES ($1, $2, $3, $4, COALESCE($5, now()))
        RETURNING id, category_id, description, amount, created_at
        "#,
        user.id,
        payload.category_id,
        payload.description,
        BigDecimal::from_f64(payload.amount),
        payload.created_at.map(convert_chrono_to_time)
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert transaction");

    let result = Transaction {
        id: record.id,
        category: fetch_category(axum::Extension(pool), &user, record.category_id).await,
        description: record.description,
        amount: record.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(record.created_at)
    };

    Json(result)
}

async fn fetch_category(
    Extension(pool): Extension<PgPool>,
    user: &User,
    category_id: i32,
) -> Category {
    let record = sqlx::query!(
        r#"
        SELECT id, name, created_at
        FROM categories
        WHERE user_id = $1
        AND id = $2
        ORDER BY id DESC
        "#,
        user.id,
        category_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch category");
    Category {
        id: record.id,
        name: record.name,
        created_at: convert_time_to_chrono(record.created_at),
    }
}
