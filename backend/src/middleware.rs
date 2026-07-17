use axum::{
    body::{to_bytes, Body},
    extract::{FromRequestParts, Request},
    http::{header, request::Parts, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use axum_extra::extract::cookie::CookieJar;
use sqlx::PgPool;
use std::hash::{DefaultHasher, Hash, Hasher};
use uuid::Uuid;

use crate::{models::user::User, time_conversion::convert_time_to_chrono};

pub async fn etag_middleware(req: Request, next: Next) -> Response {
    if req.method() != Method::GET {
        return next.run(req).await;
    }

    let if_none_match = req.headers().get(header::IF_NONE_MATCH).cloned();

    let res = next.run(req).await;

    if res.status() != StatusCode::OK {
        return res;
    }

    let (mut parts, body) = res.into_parts();

    let bytes = match to_bytes(body, 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let etag = format!("\"{:x}\"", hasher.finish());

    parts.headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-cache"),
    );

    if let Some(inm) = if_none_match {
        if let Ok(inm_str) = inm.to_str() {
            if inm_str.contains(&etag) {
                let mut not_modified = Response::builder()
                    .status(StatusCode::NOT_MODIFIED)
                    .body(Body::empty())
                    .unwrap();

                not_modified
                    .headers_mut()
                    .insert(header::ETAG, etag.parse().unwrap());
                not_modified.headers_mut().insert(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("no-cache"),
                );

                return not_modified;
            }
        }
    }

    parts.headers.insert(header::ETAG, etag.parse().unwrap());
    Response::from_parts(parts, Body::from(bytes))
}


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
