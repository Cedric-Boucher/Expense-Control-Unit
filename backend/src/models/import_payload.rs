use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImportCategory {
    pub path: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub is_asset: bool,
}

#[derive(Deserialize)]
pub struct ImportTransaction {
    pub category_path: Vec<String>,
    pub description: String,
    pub amount: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ImportPayload {
    pub categories: Vec<ImportCategory>,
    pub transactions: Vec<ImportTransaction>,
}
