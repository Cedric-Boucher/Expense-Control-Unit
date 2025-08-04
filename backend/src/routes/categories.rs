use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use crate::{middleware::AuthSession, models::category::{Category, NewCategory}, time_conversion::convert_time_to_chrono};
use futures::future::join_all;

pub fn routes() -> Router {
    Router::new().route("/api/categories", get(list_categories).post(create_category))
        .route("/api/categories/{id}", get(get_category).put(update_category).delete(delete_category))
}

async fn list_categories(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
) -> impl IntoResponse {
    let futures = sqlx::query!(
        r#"
        SELECT id, name, created_at
        FROM categories
        WHERE user_id = $1
        ORDER BY id DESC
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch transactions")
    .into_iter()
    .map(async |row| Category {
        id: row.id,
        name: row.name,
        created_at: convert_time_to_chrono(row.created_at)
    });

    let rows: Vec<Category> = join_all(futures).await;

    Json(rows)
}

pub async fn create_category(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<NewCategory>,
) -> Json<Category> {
    let record = sqlx::query!(
        r#"
        INSERT INTO categories (user_id, name)
        VALUES ($1, $2)
        RETURNING id, name, created_at
        "#,
        user.id,
        payload.name,
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert transaction");

    let result = Category {
        id: record.id,
        name: record.name,
        created_at: convert_time_to_chrono(record.created_at)
    };

    Json(result)
}

async fn get_category(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession
) -> impl IntoResponse {
    let existing = sqlx::query!(
        r#"
        SELECT id, name, created_at
        FROM categories
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch category");

    if existing.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let row = existing.unwrap();

    let category = Category {
        id: row.id,
        name: row.name,
        created_at: convert_time_to_chrono(row.created_at),
    };

    Ok(Json(category))
}

async fn update_category(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<NewCategory>,
) -> impl IntoResponse {
    let existing = sqlx::query!(
        r#"
        SELECT id FROM categories
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch category");

    if existing.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let row = sqlx::query!(
        r#"
        UPDATE categories
        SET
            name = $1
        WHERE id = $2
        RETURNING id, name, created_at
        "#,
        payload.name,
        id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to update category");

    let updated = Category {
        id: row.id,
        name: row.name,
        created_at: convert_time_to_chrono(row.created_at),
    };

    Ok(Json(updated))
}

async fn delete_category(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        DELETE FROM categories
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => Err(StatusCode::NOT_FOUND),
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
