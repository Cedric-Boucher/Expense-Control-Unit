use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use crate::{
    middleware::AuthSession,
    models::import_payload::ImportPayload,
    time_conversion::convert_chrono_to_time
};
use sqlx::PgPool;
use bigdecimal::{BigDecimal, FromPrimitive};
use std::collections::HashMap;

pub fn routes() -> Router {
    Router::new().route("/import", post(import_data))
}

pub async fn import_data(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<ImportPayload>,
) -> StatusCode {
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let mut category_map: HashMap<String, i32> = HashMap::new();

    // Step 1: Insert categories (skip duplicates)
    for cat in payload.categories {
        let rec = sqlx::query!(
            r#"
            INSERT INTO categories (user_id, name, created_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
            user.id,
            cat.name,
            convert_chrono_to_time(cat.created_at),
        )
        .fetch_one(&mut *tx)
        .await
        .expect("Failed to insert category");

        category_map.insert(cat.name, rec.id);
    }

    // Step 2: Insert transactions
    for tx_item in payload.transactions {
        let category_id = match category_map.get(&tx_item.category.name) {
            Some(id) => *id,
            None => {
                return StatusCode::BAD_REQUEST;
            }
        };

        let amount = BigDecimal::from_f64(tx_item.amount).unwrap_or(BigDecimal::from(0));

        let _ = sqlx::query!(
            r#"
            INSERT INTO transactions (user_id, category_id, description, amount, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            user.id,
            category_id,
            tx_item.description,
            amount,
            convert_chrono_to_time(tx_item.created_at),
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert transaction");
    }

    // Commit everything
    if let Err(_) = tx.commit().await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
