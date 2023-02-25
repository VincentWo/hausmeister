//! All login/logout etc. routes
//!
//! These routes are used to do everything related directly to logging in
//! or logging out

use axum::{response::IntoResponse, Extension, Json};

use serde_json::json;
use sqlx::PgPool;

use crate::{
    database::{
        auth::{login_user, Credentials, LoginError, Session},
        User,
    },
    error_handling::ApiError,
    RedisBackend,
};

/// Returns 200 if the user is logged in, 401 otherwise
///
/// Note that [session::data::SessionData] does all the actual work
#[tracing::instrument]
pub(crate) async fn test_login(
    _login_checker: session::data::SessionData<RedisBackend, User>,
) -> impl IntoResponse {
    Json(json!({
        "msg": "You are logged in."
    }))
}

/// Logs the current user out
///
/// Deletes the session in the redis cache and postgres server.
#[tracing::instrument]
pub(crate) async fn logout(
    user_data: session::data::SessionData<RedisBackend, User>,
) -> Result<impl IntoResponse, ApiError> {
    user_data.remove_session().await?;
    Ok(Json(json!({
        "msg": "You logged out.",
    })))
}

/// Tries to log the user in
///
/// Checks whether the credentials are valid (otherwise returns either 404
/// if the user cannot be found or 401 if the password is wrong) and if so
/// returns the [Session] containing the session id and user object.
#[tracing::instrument(skip(pool))]
pub(crate) async fn login(
    Extension(pool): Extension<PgPool>,
    user_data: session::data::SessionData<RedisBackend, Option<User>>,
    Json(credentials): Json<Credentials>,
) -> Result<Json<Session>, ApiError> {
    match login_user(&pool, credentials).await? {
        Ok(user) => {
            let session_data = user_data.set(user).await?;
            let session_id = session_data.id();
            let user = session_data.take_data();
            // let session_id = create_session(&mut redis_con, &user).await?;
            // todo!()
            Ok(Json(Session { session_id, user }))
        }
        Err(e) => Err(match e {
            LoginError::UserNotFound => ApiError::UserNotFound,
            LoginError::InvalidCredentials => ApiError::WrongCredentials,
        }),
    }
}
