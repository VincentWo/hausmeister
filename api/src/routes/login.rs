use argon2::{password_hash, Argon2, PasswordHash, PasswordVerifier};
use axum::{response::IntoResponse, Extension, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    database::{create_new_session, get_user_by_email, remove_session},
    error_handling::ApiError,
    middlewares::session::AuthenticatedSession,
    types::{EMail, Password},
};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Credentials {
    pub(crate) email: EMail,
    pub(crate) password: Password,
}

#[derive(Serialize)]
struct User {
    email: EMail,
    name: String,
}

#[derive(Serialize)]
struct LoginResponse {
    session_id: Uuid,
    user: User,
}

#[tracing::instrument]
pub(crate) async fn test_login(_: AuthenticatedSession) {
    ()
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn logout(
    Extension(pool): Extension<PgPool>,
    AuthenticatedSession(session_id): AuthenticatedSession,
) -> Result<(), ApiError> {
    remove_session(&pool, &session_id).await?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn login(
    Extension(pool): Extension<PgPool>,
    cookies: CookieJar,
    Json(credentials): Json<Credentials>,
) -> Result<impl IntoResponse, ApiError> {
    let user = get_user_by_email(&pool, &credentials.email).await?;

    let Some(user) = user else {
        return Err(ApiError::UserNotFound);
    };

    let parsed_hash =
        PasswordHash::new(&user.password.0).map_err(|e| ApiError::UnknownError(e.into()))?;

    Argon2::default()
        .verify_password(credentials.password.0.as_bytes(), &parsed_hash)
        .or_else(|e| match e {
            password_hash::Error::Password => Err(ApiError::WrongCredentials),
            e => Err(ApiError::UnknownError(e.into())),
        })?;

    // The password is verified, otherwise verify_password would have returned an Err

    let session = create_new_session(&pool, &user.id).await?;

    Ok((
        cookies,
        Json(LoginResponse {
            session_id: session,
            user: User {
                email: user.email,
                name: user.name,
            },
        }),
    ))
}