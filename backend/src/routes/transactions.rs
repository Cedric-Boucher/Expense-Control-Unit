use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use crate::{middleware::AuthSession, models::{category::Category, transaction::{NewTransaction, Transaction}, user::User}, time_conversion::{convert_chrono_to_time, convert_time_to_chrono}};
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};
use futures::future::join_all;

pub fn routes() -> Router {
    Router::new().route("/api/transactions", get(list_transactions).post(create_transaction))
        .route("/api/transactions/{id}", get(get_transaction).put(update_transaction))
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
        ORDER BY created_at DESC
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

async fn get_transaction(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession
) -> impl IntoResponse {
    let existing = sqlx::query!(
        r#"
        SELECT id, description, amount, created_at, category_id
        FROM transactions
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch transaction");

    if existing.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let row = existing.unwrap();

    let category = fetch_category(Extension(pool), &user, row.category_id).await;

    let transaction = Transaction {
        id: row.id,
        description: row.description,
        amount: row.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(row.created_at),
        category,
    };

    Ok(Json(transaction))
}

async fn update_transaction(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<NewTransaction>,
) -> impl IntoResponse {
    let existing = sqlx::query!(
        r#"
        SELECT id FROM transactions
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch transaction");

    if existing.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let row = sqlx::query!(
        r#"
        UPDATE transactions
        SET
            description = $1,
            amount = $2,
            created_at = COALESCE($3, created_at),
            category_id = $4
        WHERE id = $5
        RETURNING id, description, amount, created_at, category_id
        "#,
        payload.description,
        BigDecimal::from_f64(payload.amount),
        payload.created_at.map(convert_chrono_to_time),
        payload.category_id,
        id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to update transaction");

    let category = fetch_category(Extension(pool), &user, row.category_id).await;

    let updated = Transaction {
        id: row.id,
        description: row.description,
        amount: row.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(row.created_at),
        category,
    };

    Ok(Json(updated))
}
