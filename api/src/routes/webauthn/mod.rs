use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use sqlx::PgPool;
use tracing::{debug, info};
use webauthn_rs::Webauthn;

use crate::{
    database::{get_user_from_session, set_webauthn_registration},
    error_handling::ApiError,
    middlewares::session::AuthenticatedSession,
};

#[tracing::instrument]
pub(crate) async fn start_register(
    Extension(pool): Extension<PgPool>,
    AuthenticatedSession(session_id): AuthenticatedSession,
    Extension(webauthn): Extension<Arc<Webauthn>>,
) -> Result<impl IntoResponse, ApiError> {
    let user = get_user_from_session(&pool, &session_id)
        .await?
        .ok_or(ApiError::NotLoggedIn)?;

    let res = match webauthn.start_passkey_registration(user.id, &user.email.0, &user.email.0, None)
    {
        Ok((ccr, reg_state)) => {
            set_webauthn_registration(&pool, &session_id, reg_state).await?;
            Json(ccr)
        }
        Err(e) => {
            debug!("challenge_register -> {:?}", e);
            return Err(ApiError::UnknownError(e.into()));
        }
    };
    Ok(res)
}
