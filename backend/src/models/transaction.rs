use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::models::{category::Category, tag::Tag};

#[derive(Serialize)]
pub struct Transaction {
    pub id: i32,
    pub category: Category,
    pub tags: Vec<Tag>,
    pub description: String,
    pub amount: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct NewTransaction {
    pub category_id: i32,
    #[serde(default)]
    pub tag_ids: Vec<i32>,
    pub description: String,
    pub amount: f64,
    pub created_at: Option<DateTime<Utc>>,
}
