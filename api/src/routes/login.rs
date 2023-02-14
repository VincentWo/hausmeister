//! All login/logout etc. routes
//!
//! These routes are used to do everything related directly to logging in
//! or logging out

use std::sync::Arc;

use axum::{Extension, Json};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use sqlx::PgPool;

use crate::{
    database::{
        auth::{login_user, Credentials, LoginError, Session},
        remove_session, User,
    },
    error_handling::ApiError,
    middlewares::session::AuthenticatedSession,
};
use color_eyre::eyre::Context;

/// Returns 200 if the user is logged in, 401 otherwise
///
/// Note that [AuthenticatedSession] does all the actual work
#[tracing::instrument]
pub(crate) async fn test_login(_: AuthenticatedSession) {}

/// Logs the current user out
///
/// Deletes the session in the redis cache and postgres server.
#[tracing::instrument(skip(pool, redis_client))]
pub(crate) async fn logout(
    Extension(pool): Extension<PgPool>,
    Extension(redis_client): Extension<Arc<redis::Client>>,
    AuthenticatedSession(session_id): AuthenticatedSession,
) -> Result<(), ApiError> {
    let mut redis_connection = redis_client
        .get_async_connection()
        .await
        .wrap_err("Redis error")?;
    remove_session(&pool, &mut redis_connection, &session_id).await?;

    Ok(())
}

/// Tries to log the user in
///
/// Checks whether the credentials are valid (otherwise returns either 404
/// if the user cannot be found or 401 if the password is wrong) and if so
/// returns the [Session] containing the session id and user object.
// #[tracing::instrument(skip(pool))]
#[axum::debug_handler]
pub(crate) async fn login(
    Extension(pool): Extension<PgPool>,
    cookie_jar: CookieJar,
    Json(credentials): Json<Credentials>,
) -> Result<(CookieJar, Json<User>), ApiError> {
    let session = match login_user(&pool, credentials).await? {
        Ok(session) => session,
        Err(e) => {
            return Err(match e {
                LoginError::UserNotFound => ApiError::UserNotFound,
                LoginError::InvalidCredentials => ApiError::WrongCredentials,
            })
        }
    };
    let session_cookie = Cookie::build("id", session.session_id.to_string())
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .finish();

    let cookie_jar = cookie_jar.add(session_cookie);

    Ok((cookie_jar, Json(session.user)))
}
