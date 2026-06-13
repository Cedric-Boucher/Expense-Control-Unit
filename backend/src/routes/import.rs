use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use crate::{
    middleware::AuthSession,
    models::import_payload::{ImportCategory, ImportPayload},
    time_conversion::convert_chrono_to_time
};
use sqlx::PgPool;
use bigdecimal::{BigDecimal, FromPrimitive};
use std::collections::{HashMap, HashSet};

pub fn routes() -> Router {
    Router::new().route("/import", post(import_data))
}

/// Validates and sorts categories by path length.
/// Returns an Error if a path is empty or a parent path is missing.
fn validate_and_sort_categories(mut categories: Vec<ImportCategory>) -> Result<Vec<ImportCategory>, &'static str> {
    categories.sort_by_key(|c| c.path.len());

    let mut known_paths: HashSet<Vec<String>> = HashSet::new();

    for cat in &categories {
        if cat.path.is_empty() {
            return Err("Category path cannot be empty.");
        }

        let parent_path = &cat.path[..cat.path.len() - 1];
        
        if !parent_path.is_empty() && !known_paths.contains(parent_path) {
            return Err("Missing parent category in import payload.");
        }

        known_paths.insert(cat.path.clone());
    }

    Ok(categories)
}

pub async fn import_data(
    Extension(pool): Extension<PgPool>,
    AuthSession(user): AuthSession,
    Json(payload): Json<ImportPayload>,
) -> StatusCode {
    let sorted_categories = match validate_and_sort_categories(payload.categories) {
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

    let mut category_map: HashMap<Vec<String>, i32> = HashMap::new();

    // 1. Insert Categories
    for cat in sorted_categories {
        let name = match cat.path.last() {
            Some(n) => n.clone(),
            None => return StatusCode::BAD_REQUEST,
        };

        let parent_path = &cat.path[..cat.path.len() - 1];
        
        let parent_id = if parent_path.is_empty() {
            None
        } else {
            category_map.get(parent_path).copied()
        };

        let rec = sqlx::query!(
            r#"
            INSERT INTO categories (user_id, name, created_at, is_asset, parent_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, name, (COALESCE(parent_id, -1))) 
            DO UPDATE SET is_asset = EXCLUDED.is_asset
            RETURNING id
            "#,
            user.id,
            name,
            convert_chrono_to_time(cat.created_at),
            cat.is_asset,
            parent_id as Option<i32>
        )
        .fetch_one(&mut *tx)
        .await
        .expect("Failed to insert category");

        category_map.insert(cat.path.clone(), rec.id);
    }

    let mut tag_map: HashMap<String, i32> = HashMap::new();

    // 2. Insert Tags
    for tag in payload.tags {
        let rec = sqlx::query!(
            r#"
            INSERT INTO tags (user_id, name, created_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, name) 
            DO UPDATE SET name = EXCLUDED.name -- Dummy update to force RETURNING id on conflict
            RETURNING id
            "#,
            user.id,
            tag.name,
            convert_chrono_to_time(tag.created_at)
        )
        .fetch_one(&mut *tx)
        .await
        .expect("Failed to insert tag");

        tag_map.insert(tag.name.clone(), rec.id);
    }

    // 3. Insert Transactions
    for tx_item in payload.transactions {
        let category_id = match category_map.get(&tx_item.category_path) {
            Some(id) => *id,
            None => {
                eprintln!("Transaction references an unknown category path: {:?}", tx_item.category_path);
                return StatusCode::BAD_REQUEST;
            }
        };

        let amount = BigDecimal::from_f64(tx_item.amount).unwrap_or(BigDecimal::from(0));

        let tx_record = sqlx::query!(
            r#"
            INSERT INTO transactions (user_id, category_id, description, amount, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            user.id,
            category_id,
            tx_item.description,
            amount,
            convert_chrono_to_time(tx_item.created_at),
        )
        .fetch_one(&mut *tx)
        .await
        .expect("Failed to insert transaction");

        // Process and link tags for this transaction
        for tag_name in tx_item.tags {
            let tag_id = match tag_map.get(&tag_name) {
                Some(&id) => id,
                None => {
                    // Fallback: Check if the tag exists in the DB but wasn't in the payload's top-level array
                    let existing_tag = sqlx::query!(
                        "SELECT id FROM tags WHERE user_id = $1 AND name = $2 LIMIT 1",
                        user.id,
                        tag_name
                    )
                    .fetch_optional(&mut *tx)
                    .await
                    .expect("Failed to query tag");

                    if let Some(row) = existing_tag {
                        tag_map.insert(tag_name.clone(), row.id);
                        row.id
                    } else {
                        eprintln!("Transaction references an unknown tag: {}", tag_name);
                        return StatusCode::BAD_REQUEST;
                    }
                }
            };

            sqlx::query!(
                r#"
                INSERT INTO transaction_tags (transaction_id, tag_id, user_id) 
                VALUES ($1, $2, $3)
                ON CONFLICT (transaction_id, tag_id) DO NOTHING
                "#,
                tx_record.id,
                tag_id,
                user.id
            )
            .execute(&mut *tx)
            .await
            .expect("Failed to link tag");
        }
    }

    if let Err(_) = tx.commit().await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // --- Test Helpers ---

    /// Quick constructor for ImportCategory to keep tests clean
    fn make_cat(path: &[&str]) -> ImportCategory {
        ImportCategory {
            path: path.iter().map(|s| s.to_string()).collect(),
            created_at: Utc::now(),
            is_asset: false,
        }
    }

    /// Verifies that a parent appears before its child in the sorted result
    fn assert_parent_before_child(sorted: &[ImportCategory], parent_path: &[&str], child_path: &[&str]) {
        let p_vec: Vec<String> = parent_path.iter().map(|s| s.to_string()).collect();
        let c_vec: Vec<String> = child_path.iter().map(|s| s.to_string()).collect();

        let parent_idx = sorted.iter().position(|c| c.path == p_vec).unwrap();
        let child_idx = sorted.iter().position(|c| c.path == c_vec).unwrap();
        
        assert!(
            parent_idx < child_idx,
            "Expected parent '{:?}' to appear before child '{:?}'",
            parent_path, child_path
        );
    }

    // --- 🟢 Happy Paths ---

    #[test]
    fn test_empty_list() {
        let result = validate_and_sort_categories(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_flat_categories() {
        let input = vec![make_cat(&["A"]), make_cat(&["B"]), make_cat(&["C"])];
        let result = validate_and_sort_categories(input).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_simple_tree_already_sorted() {
        let input = vec![make_cat(&["Parent"]), make_cat(&["Parent", "Child"])];
        let result = validate_and_sort_categories(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_parent_before_child(&result, &["Parent"], &["Parent", "Child"]);
    }

    #[test]
    fn test_simple_tree_reversed() {
        let input = vec![make_cat(&["Parent", "Child"]), make_cat(&["Parent"])];
        let result = validate_and_sort_categories(input).unwrap();
        assert_parent_before_child(&result, &["Parent"], &["Parent", "Child"]);
    }

    #[test]
    fn test_deep_hierarchy_shuffled() {
        let input = vec![
            make_cat(&["A", "B", "C"]),
            make_cat(&["A"]),
            make_cat(&["A", "B"]),
            make_cat(&["A", "B", "C", "D"]),
        ];
        let result = validate_and_sort_categories(input).unwrap();
        assert_parent_before_child(&result, &["A"], &["A", "B"]);
        assert_parent_before_child(&result, &["A", "B"], &["A", "B", "C"]);
        assert_parent_before_child(&result, &["A", "B", "C"], &["A", "B", "C", "D"]);
    }

    #[test]
    fn test_duplicate_names_different_paths() {
        let input = vec![
            make_cat(&["Auto"]),
            make_cat(&["Auto", "Gas"]),
            make_cat(&["Home"]),
            make_cat(&["Home", "Gas"]),
        ];
        let result = validate_and_sort_categories(input);
        assert!(result.is_ok(), "Should allow identical child names if paths differ");
    }

    // --- 🔴 Error / Validation Paths ---

    #[test]
    fn test_missing_parent() {
        let input = vec![make_cat(&["Ghost", "Child"])];
        let result = validate_and_sort_categories(input);
        assert!(result.is_err(), "Should fail if parent path is not in the list");
    }

    #[test]
    fn test_deep_tree_missing_middle_node() {
        let input = vec![
            make_cat(&["A"]),
            make_cat(&["A", "B"]),
            make_cat(&["A", "B", "C", "D"]), // ["A", "B", "C"] is missing
        ];
        let result = validate_and_sort_categories(input);
        assert!(result.is_err(), "Should fail if a middle node in the hierarchy is missing");
    }

    #[test]
    fn test_empty_path() {
        let input = vec![make_cat(&[])];
        let result = validate_and_sort_categories(input);
        assert!(result.is_err(), "Should fail if a path is entirely empty");
    }
}
