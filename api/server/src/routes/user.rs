//! Routes modifying or requesting user data

use axum::{Extension, Json};
use session::data::SessionData;
use sqlx::PgPool;

use crate::{
    database::{update_user, User, UserUpdate},
    error_handling::ApiError,
    RedisBackend,
};

#[tracing::instrument]
pub(crate) async fn get_user(
    session_data: SessionData<RedisBackend, User>,
) -> Result<Json<User>, ApiError> {
    Ok(Json(session_data.take_data()))
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn patch_user(
    Extension(pool): Extension<PgPool>,
    mut session_data: SessionData<RedisBackend, User>,
    Json(user_patch): Json<UserUpdate>,
) -> Result<Json<User>, ApiError> {
    let user = update_user(&pool, &session_data.get().id, user_patch)
        .await?
        .ok_or(ApiError::InvalidSession)?;

    session_data.set(user).await?;

    Ok(Json(session_data.take_data()))
}
