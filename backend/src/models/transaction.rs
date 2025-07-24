use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i32,
    pub description: String,
    pub amount: f64
}
