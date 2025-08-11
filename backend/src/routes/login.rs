use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use sqlx::PgPool;

use crate::{models::user::NewUser, passwords::verify_password};

pub fn routes() -> Router {
    Router::new().route("/login", post(login))
}

pub async fn login(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
    Json(payload): Json<NewUser>,
) -> impl IntoResponse {
    let record = sqlx::query!(
        r#"
        SELECT id, username, password_hash, created_at
        FROM users
        WHERE username = $1
        "#,
        payload.username
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch user");

    let user = match record {
        Some(user) => user,
        _ => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    };

    let password_is_valid = match verify_password(&user.password_hash, &payload.password) {
        Ok(true) => true,
        _ => false,
    };

    if !password_is_valid {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
    }

    let cookie = Cookie::build(("session", user.id.to_string()))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax);

    (jar.add(cookie), StatusCode::OK).into_response()
}
