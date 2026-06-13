use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use crate::{middleware::AuthSession, models::tag::{Tag, NewTag}, time_conversion::convert_time_to_chrono};

pub fn routes() -> Router {
    Router::new()
        .route("/tags", get(list_tags).post(create_tag))
        .route("/tags/{id}", get(get_tag).put(update_tag).delete(delete_tag))
}

async fn list_tags(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
) -> impl IntoResponse {
    let rows: Vec<Tag> = sqlx::query!(
        r#"
        SELECT id, name, created_at
        FROM tags
        WHERE user_id = $1
        ORDER BY name ASC
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch tags")
    .into_iter()
    .map(|row| Tag {
        id: row.id,
        name: row.name,
        created_at: convert_time_to_chrono(row.created_at),
    })
    .collect();

    Json(rows)
}

pub async fn create_tag(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<NewTag>,
) -> Result<Json<Tag>, StatusCode> {
    let trimmed_name = payload.name.trim();
    if trimmed_name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let duplicate_check = sqlx::query!(
        r#"
        SELECT id FROM tags 
        WHERE user_id = $1 AND LOWER(name) = LOWER($2)
        LIMIT 1
        "#,
        user.id,
        trimmed_name
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to check for duplicate tag");

    if duplicate_check.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let record = sqlx::query!(
        r#"
        INSERT INTO tags (user_id, name)
        VALUES ($1, $2)
        RETURNING id, name, created_at
        "#,
        user.id,
        trimmed_name
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert tag");

    let result = Tag {
        id: record.id,
        name: record.name,
        created_at: convert_time_to_chrono(record.created_at),
    };

    Ok(Json(result))
}

async fn get_tag(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
) -> impl IntoResponse {
    let existing = sqlx::query!(
        r#"
        SELECT id, name, created_at
        FROM tags
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch tag");

    if existing.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let row = existing.unwrap();
    let tag = Tag {
        id: row.id,
        name: row.name,
        created_at: convert_time_to_chrono(row.created_at),
    };

    Ok(Json(tag))
}

async fn update_tag(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<NewTag>,
) -> impl IntoResponse {
    let trimmed_name = payload.name.trim();
    if trimmed_name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    let duplicate_check = sqlx::query!(
        r#"
        SELECT id FROM tags 
        WHERE user_id = $1 AND LOWER(name) = LOWER($2) AND id != $3
        LIMIT 1
        "#,
        user.id,
        trimmed_name,
        id
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("Failed to check for duplicate tag");

    if duplicate_check.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let row = sqlx::query!(
        r#"
        UPDATE tags 
        SET name = $1
        WHERE id = $2 AND user_id = $3
        RETURNING id, name, created_at
        "#,
        trimmed_name,
        id,
        user.id
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("Failed to update tag");

    if row.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let row = row.unwrap();
    tx.commit().await.expect("Failed to commit transaction");

    let updated = Tag {
        id: row.id,
        name: row.name,
        created_at: convert_time_to_chrono(row.created_at),
    };

    Ok(Json(updated))
}

async fn delete_tag(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        DELETE FROM tags
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
