use serde::Serialize;

#[derive(Serialize)]
pub struct Transaction {
    pub id: u32,
    pub description: String,
    pub amount: i32
}
