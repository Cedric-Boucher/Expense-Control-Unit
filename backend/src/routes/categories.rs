use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use std::collections::HashMap;
use crate::{
    middleware::AuthSession, 
    models::{
        category::{Category, NewCategory}, 
        tag::Tag, 
        transaction::Transaction
    }, 
    time_conversion::convert_time_to_chrono
};
use bigdecimal::ToPrimitive;
use futures::future::join_all;

pub fn routes() -> Router {
    Router::new().route("/categories", get(list_categories).post(create_category))
        .route("/categories/{id}", get(get_category).put(update_category).delete(delete_category))
        .route("/categories/{id}/transactions", get(get_transactions))
}

async fn list_categories(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
) -> impl IntoResponse {
    let futures = sqlx::query!(
        r#"
        SELECT id, name, is_asset, parent_id, created_at
        FROM categories
        WHERE user_id = $1
        ORDER BY name ASC
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch categories")
    .into_iter()
    .map(async |row| Category {
        id: row.id,
        name: row.name,
        parent_id: row.parent_id,
        is_asset: row.is_asset,
        created_at: convert_time_to_chrono(row.created_at)
    });

    let rows: Vec<Category> = join_all(futures).await;

    Json(rows)
}

pub async fn create_category(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<NewCategory>,
) -> Result<Json<Category>, StatusCode> {
    let duplicate_check = sqlx::query!(
        r#"
        SELECT id FROM categories 
        WHERE user_id = $1 
          AND LOWER(name) = LOWER($2) 
          AND (parent_id IS NOT DISTINCT FROM $3)
        LIMIT 1
        "#,
        user.id,
        payload.name,
        payload.parent_id as Option<i32>
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to check for duplicate category");

    if duplicate_check.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let record = sqlx::query!(
        r#"
        INSERT INTO categories (user_id, name, is_asset, parent_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, is_asset, parent_id, created_at
        "#,
        user.id,
        payload.name,
        payload.is_asset,
        payload.parent_id as Option<i32>
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert category");

    let result = Category {
        id: record.id,
        name: record.name,
        parent_id: record.parent_id,
        is_asset: record.is_asset,
        created_at: convert_time_to_chrono(record.created_at)
    };

    Ok(Json(result))
}

async fn get_category(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession
) -> impl IntoResponse {
    let existing = sqlx::query!(
        r#"
        SELECT id, name, is_asset, parent_id, created_at
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
        parent_id: row.parent_id,
        is_asset: row.is_asset,
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
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    // 1. Check for duplicates (ignoring this exact category's ID)
    let duplicate_check = sqlx::query!(
        r#"
        SELECT id FROM categories 
        WHERE user_id = $1 
          AND LOWER(name) = LOWER($2) 
          AND (parent_id IS NOT DISTINCT FROM $3)
          AND id != $4
        LIMIT 1
        "#,
        user.id,
        payload.name,
        payload.parent_id as Option<i32>,
        id
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("Failed to check for duplicate category");

    if duplicate_check.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    // 2. Check for circular dependencies BEFORE we update anything
    if let Some(pid) = payload.parent_id {
        if pid == id {
            // Cannot parent to itself
            return Err(StatusCode::BAD_REQUEST);
        }

        let is_descendant = sqlx::query!(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id FROM categories WHERE parent_id = $1
                UNION ALL
                SELECT c.id FROM categories c
                INNER JOIN descendants d ON c.parent_id = d.id
            )
            SELECT id FROM descendants WHERE id = $2
            "#,
            id,
            pid
        )
        .fetch_optional(&mut *tx)
        .await
        .expect("Failed to check descendants");

        if is_descendant.is_some() {
            // Cannot parent to a descendant (circular loop)
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // 3. Proceed with update
    let row = sqlx::query!(
        r#"
        UPDATE categories 
        SET name = $1, is_asset = $2, parent_id = $3 
        WHERE id = $4 AND user_id = $5
        RETURNING id, name, is_asset, parent_id, created_at
        "#,
        payload.name,
        payload.is_asset,
        payload.parent_id as Option<i32>,
        id,
        user.id
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("Failed to update category");

    if row.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    let row = row.unwrap();

    tx.commit().await.expect("Failed to commit transaction");

    let updated = Category {
        id: row.id,
        name: row.name,
        parent_id: row.parent_id,
        is_asset: row.is_asset,
        created_at: convert_time_to_chrono(row.created_at),
    };

    Ok(Json(updated))
}

async fn delete_category(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession
) -> impl IntoResponse {
    // 1. Check if this category is a parent to any children
    let has_children = sqlx::query!(
        r#"
        SELECT 1 AS exists 
        FROM categories 
        WHERE parent_id = $1
        LIMIT 1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to check for child categories");

    if has_children.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    // 2. If no children exist, proceed with deletion
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

async fn get_transactions(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession
) -> impl IntoResponse {
    let tags_records = sqlx::query!(
        r#"
        WITH RECURSIVE category_tree AS (
            SELECT id FROM categories WHERE id = $1 AND user_id = $2
            UNION ALL
            SELECT c.id FROM categories c
            INNER JOIN category_tree ct ON c.parent_id = ct.id
        )
        SELECT tt.transaction_id, t.id, t.name, t.created_at
        FROM tags t
        JOIN transaction_tags tt ON t.id = tt.tag_id
        JOIN transactions txn ON tt.transaction_id = txn.id
        WHERE txn.category_id IN (SELECT id FROM category_tree)
        AND t.user_id = $2
        "#,
        id,
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
        });
    }

    let rows: Vec<Transaction> = sqlx::query!(
        r#"
        WITH RECURSIVE category_tree AS (
            -- Base case: the requested category
            SELECT id FROM categories WHERE id = $1 AND user_id = $2
            UNION ALL
            -- Recursive step: find all children of the categories in the tree
            SELECT c.id
            FROM categories c
            INNER JOIN category_tree ct ON c.parent_id = ct.id
        )
        SELECT
            t.id AS transaction_id, t.description, t.amount, t.created_at AS transaction_created_at,
            c.id AS category_id, c.name, c.is_asset AS category_is_asset, c.parent_id, c.created_at AS category_created_at
        FROM transactions t
        JOIN categories c ON t.category_id = c.id
        WHERE c.id IN (SELECT id FROM category_tree)
        AND t.user_id = $2
        ORDER BY t.created_at DESC
        "#,
        id,
        user.id
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch category transactions")
    .into_iter()
    .map(|row| Transaction {
        id: row.transaction_id,
        category: Category { 
            id: row.category_id,
            name: row.name,
            parent_id: row.parent_id, 
            is_asset: row.category_is_asset,
            created_at: convert_time_to_chrono(row.category_created_at),
        },
        tags: tags_by_transaction.remove(&row.transaction_id).unwrap_or_default(),
        description: row.description,
        amount: row.amount.to_f64().unwrap_or(0.0),
        created_at: convert_time_to_chrono(row.transaction_created_at)
    })
    .collect(); 

    Json(rows)
}
