use axum::{response::IntoResponse, Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::{
    database::{self, get_user_by_email, new_reset_request, ResetError},
    error_handling::ApiError,
    types::{EMail, Password},
};

#[derive(Debug, Deserialize)]
pub(crate) struct ResetRequest {
    email: EMail,
}
#[tracing::instrument(skip(pool))]
pub(crate) async fn request_reset(
    Extension(pool): Extension<PgPool>,
    Json(ResetRequest { email }): Json<ResetRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = get_user_by_email(&pool, &email)
        .await?
        .ok_or(ApiError::UserNotFound)?;

    let reset_id = new_reset_request(&pool, &user.id).await?;

    debug!("Reset ID for {email:#?}: {reset_id}");

    Ok(())
}

#[derive(Debug, Deserialize)]
pub(crate) struct PasswordReset {
    reset_token: Uuid,
    new_password: Password,
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn reset_password(
    Extension(pool): Extension<PgPool>,
    Json(PasswordReset {
        reset_token,
        new_password,
    }): Json<PasswordReset>,
) -> Result<impl IntoResponse, ApiError> {
    match database::reset_password(&pool, &reset_token, &new_password).await? {
        Err(ResetError::TokenNotFound) => Err(ApiError::TokenNotFound),
        Ok(()) => Ok("Password was reset"),
    }
}

#[derive(Deserialize)]
pub struct TokenCheck {
    reset_token: Uuid,
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn test_reset_token(
    Extension(pool): Extension<PgPool>,
    Json(TokenCheck { reset_token }): Json<TokenCheck>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(
        database::reset_token_is_valid(&pool, &reset_token).await?,
    ))
}
