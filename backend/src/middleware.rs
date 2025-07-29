use axum::{extract::FromRequestParts, http::{StatusCode, request::Parts}, Extension};
use axum_extra::extract::cookie::CookieJar;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{models::user::User, time_conversion::convert_time_to_chrono};

pub struct AuthSession(pub User);

impl<S> FromRequestParts<S> for AuthSession
where
    PgPool: Send + Sync,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<PgPool>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "State unavailable"))?;

        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to extract cookies"))?;

        let Some(session_cookie) = jar.get("session") else {
            return Err((StatusCode::UNAUTHORIZED, "No session cookie"));
        };

        let user_id = session_cookie
            .value()
            .parse::<Uuid>()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid session token"))?;

        let user_record = sqlx::query!(
            "SELECT id, username, password_hash, created_at FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load user"))?;

        let user = match user_record {
            Some(user) => Some(User {
                id: user.id,
                username: user.username,
                password_hash: user.password_hash,
                created_at: convert_time_to_chrono(user.created_at),
            }),
            None => None,
        };

        match user {
            Some(user) => Ok(AuthSession(user)),
            None => Err((StatusCode::UNAUTHORIZED, "Invalid session")),
        }
    }
}
