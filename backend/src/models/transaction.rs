use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::models::category::Category;

#[derive(Serialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i32,
    pub category: Category,
    pub description: String,
    pub amount: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct NewTransaction {
    pub category_id: i32,
    pub description: String,
    pub amount: f64,
    pub created_at: Option<DateTime<Utc>>,
}
