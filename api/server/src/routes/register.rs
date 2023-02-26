use axum::{response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    database::{create_user, UserCreation},
    error_handling::ApiError,
};

#[tracing::instrument]
pub(crate) async fn register(
    Extension(pool): Extension<PgPool>,
    Json(new_user): Json<UserCreation>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(create_user(&pool, new_user).await?))
}
