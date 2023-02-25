//!
//!
//!

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

/// JSON for requesting a password reset
#[derive(Debug, Deserialize)]
pub(crate) struct ResetRequest {
    /// The email of the account which requested a password reset
    email: EMail,
}

/// Request a code for a password reset
///
/// This checks whether an account exists (otherwise return 404) and
/// if so create a new request to reset the password of the account
///
/// At the moment this just logs the reset id, email infrastructure is planned.
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

/// JSON for resetting a password
#[derive(Debug, Deserialize)]
pub(crate) struct PasswordReset {
    /// The token sent by mail
    reset_token: Uuid,
    /// The new, unhashed password
    new_password: Password,
}

/// Execute the password reset
///
/// Checks whether the `reset_token` exists, otherwise returns 404,
/// then deletes the token (invalidating it) and set's the new password.
///
/// This function works atomically so if an error is returned it is guarenteed
/// that the reset did not happen.
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

/// JSON for checking the validity of a `reset_token`
#[derive(Deserialize)]
pub struct TokenCheck {
    /// The maybe valid reset token
    reset_token: Uuid,
}
/// Check whether a `reset_token` is valid
///
/// Usefull for UX purposes
#[tracing::instrument(skip(pool))]
pub(crate) async fn test_reset_token(
    Extension(pool): Extension<PgPool>,
    Json(TokenCheck { reset_token }): Json<TokenCheck>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        database::reset_token_is_valid(&pool, &reset_token).await?,
    ))
}
