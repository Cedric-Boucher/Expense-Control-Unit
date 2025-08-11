use axum::{http::StatusCode, response::IntoResponse, routing::post, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use tower_cookies::cookie::time;

pub fn routes() -> Router {
    Router::new().route("/logout", post(logout))
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cleared = Cookie::build(("session", ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .http_only(true)
        .secure(false);

    (jar.remove(cleared), StatusCode::OK).into_response()
}
