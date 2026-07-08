use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, sqlx::FromRow, Clone, Debug)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub closing_date: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
pub struct NewTag {
    pub name: String,
    pub closing_date: Option<DateTime<Utc>>,
}
