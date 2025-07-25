use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i32,
    pub description: String,
    pub amount: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct NewTransaction {
    pub description: String,
    pub amount: f64,
    pub created_at: Option<DateTime<Utc>>,
}
