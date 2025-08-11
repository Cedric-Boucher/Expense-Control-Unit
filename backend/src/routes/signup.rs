use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use sqlx::PgPool;
use tower_cookies::cookie::SameSite;
use uuid::Uuid;

use crate::{models::user::NewUser, passwords::hash_password};

pub fn routes() -> Router {
    Router::new().route("/signup", post(signup))
}

pub async fn signup(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
    Json(payload): Json<NewUser>,
) -> impl IntoResponse {
    if payload.password.len() < 8 {
        return (StatusCode::BAD_REQUEST, "Password too short (minimum 8 characters)").into_response();
    }

    let existing_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE username = $1"
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .expect("Failed to check for existing username");

    if existing_count > 0 {
        return (StatusCode::CONFLICT, "Username already taken").into_response();
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response(),
    };

    let user_id = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)"
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(&password_hash)
    .execute(&pool)
    .await;

    if result.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response();
    }

    let cookie = Cookie::build(("session", user_id.to_string()))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax);

    (jar.add(cookie), StatusCode::CREATED).into_response()
}
