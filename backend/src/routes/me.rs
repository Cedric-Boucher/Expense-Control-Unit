use axum::Json;
use axum::response::IntoResponse;

use axum::{Router, routing::get};

use crate::middleware::AuthSession;
use crate::models::user::User;

pub fn routes() -> Router {
    Router::new().route("/api/me", get(login_check))
}

async fn login_check(AuthSession(user): AuthSession) -> impl IntoResponse {
    let user = User {
        id: user.id,
        username: user.username,
        password_hash: user.password_hash,
        created_at: user.created_at,
    };

    Json(user)
}
