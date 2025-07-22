use axum::{Router, routing::get, Json};
use crate::models::transaction::Transaction;

pub fn routes() -> Router {
    Router::new().route("/api/transactions", get(list_transactions))
}

async fn list_transactions() -> Json<Vec<Transaction>> {
    let sample_data = vec![
        Transaction { id: 1, description: "Sample".into(), amount: -156 },
        Transaction { id: 2, description: "Test".into(), amount: 843 },
    ];

    Json(sample_data)
}
