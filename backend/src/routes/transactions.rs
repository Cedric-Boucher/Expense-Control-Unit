use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use std::collections::HashMap;
use crate::{
    middleware::AuthSession, 
    models::{category::Category, tag::Tag, transaction::{NewTransaction, Transaction}, user::User}, 
    time_conversion::{convert_chrono_to_time, convert_time_to_chrono}
};
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};

pub fn routes() -> Router {
    Router::new().route("/transactions", get(list_transactions).post(create_transaction))
        .route("/transactions/{id}", get(get_transaction).put(update_transaction).delete(delete_transaction))
}

async fn list_transactions(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
) -> impl IntoResponse {
    let tags_records = sqlx::query!(
        r#"
        SELECT tt.transaction_id, t.id, t.name, t.created_at, t.closing_date
        FROM tags t
        JOIN transaction_tags tt ON t.id = tt.tag_id
        WHERE t.user_id = $1
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut tags_by_transaction: HashMap<i32, Vec<Tag>> = HashMap::new();
    for row in tags_records {
        tags_by_transaction.entry(row.transaction_id).or_default().push(Tag {
            id: row.id,
            name: row.name,
            created_at: convert_time_to_chrono(row.created_at),
            closing_date: row.closing_date.map(convert_time_to_chrono)
        });
    }

    let rows: Vec<Transaction> = sqlx::query!(
        r#"
        SELECT
            transactions.id as transaction_id,
            categories.id as category_id,
            transactions.description as transaction_description,
            categories.name as category_name,
            categories.is_asset as category_is_asset,
            amount,
            transactions.created_at as transaction_created_at,
            categories.created_at as category_created_at,
            categories.parent_id
        FROM transactions
        JOIN categories ON transactions.category_id = categories.id
        WHERE transactions.user_id = $1
        ORDER BY transactions.created_at DESC
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch transactions")
    .into_iter()
    .map(|row| Transaction {
        id: row.transaction_id,
        category: Category {
            id: row.category_id,
            name: row.category_name,
            parent_id: row.parent_id,
            is_asset: row.category_is_asset,
            created_at: convert_time_to_chrono(row.category_created_at)
        },
        tags: tags_by_transaction.remove(&row.transaction_id).unwrap_or_default(),
        description: row.transaction_description,
        amount: row.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(row.transaction_created_at)
    })
    .collect(); 

    Json(rows)
}

pub async fn create_transaction(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<NewTransaction>,
) -> Json<Transaction> {
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

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
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to insert transaction");

    for tag_id in &payload.tag_ids {
        sqlx::query!(
            r#"
            INSERT INTO transaction_tags (user_id, transaction_id, tag_id)
            VALUES ($1, $2, $3)
            "#,
            user.id,
            record.id,
            tag_id
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to link tag to transaction");
    }

    tx.commit().await.expect("Failed to commit transaction");

    let category = fetch_category(&pool, &user, record.category_id).await;
    let tags = fetch_transaction_tags(&pool, &user, record.id).await;

    let result = Transaction {
        id: record.id,
        category,
        tags,
        description: record.description,
        amount: record.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(record.created_at)
    };

    Json(result)
}

async fn fetch_category(
    pool: &PgPool,
    user: &User,
    category_id: i32,
) -> Category {
    let record = sqlx::query!(
        r#"
        SELECT id, name, is_asset, created_at, parent_id
        FROM categories
        WHERE user_id = $1
        AND id = $2
        ORDER BY id DESC
        "#,
        user.id,
        category_id
    )
    .fetch_one(pool)
    .await
    .expect("Failed to fetch category");

    Category {
        id: record.id,
        name: record.name,
        parent_id: record.parent_id, 
        is_asset: record.is_asset,
        created_at: convert_time_to_chrono(record.created_at),
    }
}

async fn fetch_transaction_tags(
    pool: &PgPool,
    user: &User,
    transaction_id: i32,
) -> Vec<Tag> {
    sqlx::query!(
        r#"
        SELECT t.id, t.name, t.created_at, t.closing_date
        FROM tags t
        JOIN transaction_tags tt ON t.id = tt.tag_id
        WHERE tt.transaction_id = $1 AND t.user_id = $2
        ORDER BY t.name ASC
        "#,
        transaction_id,
        user.id
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| Tag {
        id: row.id,
        name: row.name,
        created_at: convert_time_to_chrono(row.created_at),
        closing_date: row.closing_date.map(convert_time_to_chrono)
    })
    .collect()
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

    let category = fetch_category(&pool, &user, row.category_id).await;
    let tags = fetch_transaction_tags(&pool, &user, row.id).await;

    let transaction = Transaction {
        id: row.id,
        description: row.description,
        amount: row.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(row.created_at),
        category,
        tags,
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

    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    let row = sqlx::query!(
        r#"
        UPDATE transactions
        SET
            description = $1,
            amount = $2,
            created_at = COALESCE($3, created_at),
            category_id = $4
        WHERE id = $5 AND user_id = $6
        RETURNING id, description, amount, created_at, category_id
        "#,
        payload.description,
        BigDecimal::from_f64(payload.amount),
        payload.created_at.map(convert_chrono_to_time),
        payload.category_id,
        id,
        user.id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to update transaction");

    // Clear existing tags and re-insert the new ones
    sqlx::query!("DELETE FROM transaction_tags WHERE transaction_id = $1 AND user_id = $2", id, user.id)
        .execute(&mut *tx)
        .await
        .expect("Failed to clear old tags");

    for tag_id in &payload.tag_ids {
        sqlx::query!(
            r#"
            INSERT INTO transaction_tags (user_id, transaction_id, tag_id) 
            SELECT $3, $1, id FROM tags WHERE id = $2 AND user_id = $3
            "#,
            id,
            tag_id,
            user.id
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to link tag to transaction");
    }

    tx.commit().await.expect("Failed to commit transaction");

    let category = fetch_category(&pool, &user, row.category_id).await;
    let tags = fetch_transaction_tags(&pool, &user, row.id).await;

    let updated = Transaction {
        id: row.id,
        description: row.description,
        amount: row.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(row.created_at),
        category,
        tags,
    };

    Ok(Json(updated))
}

async fn delete_transaction(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        DELETE FROM transactions
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
