use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    database::{get_user_from_session, update_current_user, PatchUser, User},
    error_handling::ApiError,
    middlewares::session::AuthenticatedSession,
};

#[tracing::instrument]
pub(crate) async fn get_user(
    Extension(pool): Extension<PgPool>,
    AuthenticatedSession(session_id): AuthenticatedSession,
) -> Result<Json<User>, ApiError> {
    let user = get_user_from_session(&pool, &session_id)
        .await?
        .ok_or(ApiError::InvalidSession)?;
    Ok(Json(user))
}

#[tracing::instrument]
pub(crate) async fn patch_user(
    Extension(pool): Extension<PgPool>,
    AuthenticatedSession(session_id): AuthenticatedSession,
    Json(user_patch): Json<PatchUser>,
) -> Result<Json<User>, ApiError> {
    let user = update_current_user(&pool, &session_id, user_patch)
        .await?
        .ok_or(ApiError::InvalidSession)?;

    Ok(Json(user))
}
