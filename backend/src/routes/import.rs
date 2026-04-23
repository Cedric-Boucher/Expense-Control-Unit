use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use crate::{
    middleware::AuthSession,
    models::import_payload::{ImportCategory, ImportPayload},
    time_conversion::convert_chrono_to_time
};
use sqlx::PgPool;
use bigdecimal::{BigDecimal, FromPrimitive};
use std::collections::{HashMap, VecDeque};

pub fn routes() -> Router {
    Router::new().route("/import", post(import_data))
}

/// Validates and sorts categories topologically (parents before children).
/// Returns an Error if there is a circular dependency or missing parent.
fn sort_categories_topologically(categories: Vec<ImportCategory>) -> Result<Vec<ImportCategory>, &'static str> {
    let mut cat_by_name = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let cat_count = categories.len();

    // Build the graph and count dependencies
    for cat in categories {
        let name = cat.name.clone();
        let parent_name = cat.parent_name.clone();

        cat_by_name.insert(name.clone(), cat);
        in_degree.entry(name.clone()).or_insert(0);

        if let Some(p_name) = parent_name {
            *in_degree.entry(name.clone()).or_insert(0) += 1;
            graph.entry(p_name).or_default().push(name);
        }
    }

    let mut queue = VecDeque::new();

    // Start with top-level categories (0 dependencies)
    for (name, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(name.clone());
        }
    }

    let mut sorted_categories = Vec::new();

    // Process the queue top-down
    while let Some(name) = queue.pop_front() {
        if let Some(cat) = cat_by_name.remove(&name) {
            sorted_categories.push(cat);
        }

        if let Some(children) = graph.get(&name) {
            for child_name in children {
                if let Some(deg) = in_degree.get_mut(child_name) {
                    *deg -= 1; // Parent processed, remove the dependency requirement
                    if *deg == 0 {
                        queue.push_back(child_name.clone());
                    }
                }
            }
        }
    }

    // Validation: If counts don't match, we mathematically failed to resolve the tree
    if sorted_categories.len() != cat_count {
        return Err("Circular dependency or dangling parent reference detected.");
    }

    Ok(sorted_categories)
}

pub async fn import_data(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<ImportPayload>,
) -> StatusCode {
    // Validation (Fails fast before we ever touch the database)
    let sorted_categories = match sort_categories_topologically(payload.categories) {
        Ok(categories) => categories,
        Err(e) => {
            eprintln!("Import Validation Failed: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let mut category_map: HashMap<String, i32> = HashMap::new();

    // Insert Categories Safely in Top-Down Order
    for cat in sorted_categories {
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

        let new_id = rec.id;

        // Map Category Name to the newly generated Postgres ID
        category_map.insert(cat.name.clone(), new_id);

        // Upsert the hierarchy link
        if let Some(ref p_name) = cat.parent_name {
            if let Some(&new_pid) = category_map.get(p_name) {
                sqlx::query!(
                    r#"
                    INSERT INTO category_hierarchy (category_id, parent_id, user_id)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (category_id) DO UPDATE SET parent_id = EXCLUDED.parent_id
                    "#,
                    new_id,
                    new_pid,
                    user.id
                )
                .execute(&mut *tx)
                .await
                .expect("Failed to insert hierarchy link");
            }
        } else {
            // Ensure no hierarchy link exists if the import explicitly sets it to null
            sqlx::query!(
                r#"DELETE FROM category_hierarchy WHERE category_id = $1"#,
                new_id
            )
            .execute(&mut *tx)
            .await
            .expect("Failed to delete hierarchy link");
        }
    }

    // Insert Transactions
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
