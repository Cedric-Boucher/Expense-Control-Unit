use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImportCategory {
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ImportTransaction {
    pub category: ImportCategory,
    pub description: String,
    pub amount: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ImportPayload {
    pub categories: Vec<ImportCategory>,
    pub transactions: Vec<ImportTransaction>,
}
