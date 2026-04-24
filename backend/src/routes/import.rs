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
        let category_id = match category_map.get(&tx_item.category_name) {
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


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // --- Test Helpers ---

    /// Quick constructor for ImportCategory to keep tests clean
    fn make_cat(name: &str, parent: Option<&str>) -> ImportCategory {
        ImportCategory {
            name: name.to_string(),
            created_at: Utc::now(),
            parent_name: parent.map(|s| s.to_string()),
        }
    }

    /// Verifies that a parent appears before its child in the sorted result
    fn assert_parent_before_child(sorted: &[ImportCategory], parent_name: &str, child_name: &str) {
        let parent_idx = sorted.iter().position(|c| c.name == parent_name).unwrap();
        let child_idx = sorted.iter().position(|c| c.name == child_name).unwrap();
        assert!(
            parent_idx < child_idx,
            "Expected parent '{}' to appear before child '{}'",
            parent_name, child_name
        );
    }

    // --- 🟢 Happy Paths ---

    #[test]
    fn test_empty_list() {
        let result = sort_categories_topologically(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_flat_categories() {
        let input = vec![make_cat("A", None), make_cat("B", None), make_cat("C", None)];
        let result = sort_categories_topologically(input).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_simple_tree_already_sorted() {
        let input = vec![make_cat("Parent", None), make_cat("Child", Some("Parent"))];
        let result = sort_categories_topologically(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_parent_before_child(&result, "Parent", "Child");
    }

    #[test]
    fn test_simple_tree_reversed() {
        let input = vec![make_cat("Child", Some("Parent")), make_cat("Parent", None)];
        let result = sort_categories_topologically(input).unwrap();
        assert_parent_before_child(&result, "Parent", "Child");
    }

    #[test]
    fn test_deep_hierarchy_shuffled() {
        let input = vec![
            make_cat("C", Some("B")),
            make_cat("A", None),
            make_cat("B", Some("A")),
            make_cat("D", Some("C")),
        ];
        let result = sort_categories_topologically(input).unwrap();
        assert_parent_before_child(&result, "A", "B");
        assert_parent_before_child(&result, "B", "C");
        assert_parent_before_child(&result, "C", "D");
    }

    #[test]
    fn test_multiple_disconnected_trees() {
        let input = vec![
            make_cat("ChildB", Some("RootB")),
            make_cat("RootA", None),
            make_cat("ChildA", Some("RootA")),
            make_cat("RootB", None),
        ];
        let result = sort_categories_topologically(input).unwrap();
        assert_parent_before_child(&result, "RootA", "ChildA");
        assert_parent_before_child(&result, "RootB", "ChildB");
    }

    // --- 🔴 Error / Validation Paths ---

    #[test]
    fn test_missing_parent() {
        let input = vec![make_cat("Child", Some("Ghost"))];
        let result = sort_categories_topologically(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_self_referencing_cycle() {
        let input = vec![make_cat("A", Some("A"))];
        let result = sort_categories_topologically(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_direct_circular_dependency() {
        let input = vec![make_cat("A", Some("B")), make_cat("B", Some("A"))];
        let result = sort_categories_topologically(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_deep_circular_dependency() {
        let input = vec![
            make_cat("A", Some("C")),
            make_cat("B", Some("A")),
            make_cat("C", Some("B")),
        ];
        let result = sort_categories_topologically(input);
        assert!(result.is_err());
    }

    // --- 🟡 Edge Cases ---

    #[test]
    fn test_duplicate_category_names() {
        let input = vec![
            make_cat("A", None),
            make_cat("A", Some("B")), // Duplicate overwrites in map
            make_cat("B", None),
        ];
        let result = sort_categories_topologically(input);
        // Because of the overwrite, the total count mapped won't match the input length
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_children_one_parent() {
        let input = vec![
            make_cat("Child1", Some("A")),
            make_cat("Child2", Some("A")),
            make_cat("A", None),
        ];
        let result = sort_categories_topologically(input).unwrap();
        assert_parent_before_child(&result, "A", "Child1");
        assert_parent_before_child(&result, "A", "Child2");
    }

    #[test]
    fn test_deep_tree_missing_leaf() {
        let input = vec![
            make_cat("A", None),
            make_cat("B", Some("A")),
            make_cat("C", Some("B")),
            make_cat("E", Some("D")), // D is missing
        ];
        let result = sort_categories_topologically(input);
        assert!(result.is_err());
    }
}
