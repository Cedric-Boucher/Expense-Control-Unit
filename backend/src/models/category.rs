use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct NewCategory {
    pub name: String,
}
