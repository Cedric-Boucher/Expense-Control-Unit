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
    // Sorting by path length guarantees parents (shorter paths) are processed before children
    categories.sort_by_key(|c| c.path.len());

    let mut known_paths: HashSet<Vec<String>> = HashSet::new();

    for cat in &categories {
        if cat.path.is_empty() {
            return Err("Category path cannot be empty.");
        }

        let parent_path = &cat.path[..cat.path.len() - 1];
        
        // If it has a parent, the parent MUST have been processed already
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
    // Validation (Fails fast before touching the database)
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

    // Maps a full Materialized Path (e.g., ["Auto", "Gas"]) to its Postgres ID
    let mut category_map: HashMap<Vec<String>, i32> = HashMap::new();

    // Insert Categories in Top-Down Order
    for cat in sorted_categories {
        let name = match cat.path.last() {
            Some(n) => n.clone(),
            None => return StatusCode::BAD_REQUEST, // Caught by validation, but safe unwrap
        };

        let parent_path = &cat.path[..cat.path.len() - 1];
        
        let parent_id = if parent_path.is_empty() {
            None
        } else {
            category_map.get(parent_path).copied()
        };

        // Insert using the schema and scoped uniqueness expression
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

        // Map the full path to the newly generated Postgres ID
        category_map.insert(cat.path.clone(), rec.id);
    }

    // Insert Transactions
    for tx_item in payload.transactions {
        let category_id = match category_map.get(&tx_item.category_path) {
            Some(id) => *id,
            None => {
                eprintln!("Transaction references an unknown category path: {:?}", tx_item.category_path);
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
