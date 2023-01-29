use axum::{response::IntoResponse, Extension, Json};
use sqlx::PgPool;

use crate::{database::list_user, error_handling::ApiError};

pub(crate) mod login;
pub(crate) mod reset;
pub(crate) mod user;

#[tracing::instrument(skip(pool))]
pub(crate) async fn root(
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, ApiError> {
    let res = list_user(&pool).await?;

    Ok(Json(res))
}
