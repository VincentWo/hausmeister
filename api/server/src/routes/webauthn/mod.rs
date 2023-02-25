use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, Extension, Json};

use serde::{Deserialize, Serialize};
use serde_json::json;
use session::data::{Extractable, SessionData};
use sqlx::PgPool;
use tracing::{debug, error};
use uuid::Uuid;
use webauthn_rs::{
    prelude::{
        PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
        RegisterPublicKeyCredential,
    },
    Webauthn,
};

use crate::{
    database::{
        add_passkey_for_user, get_passkeys_for_user, get_user_by_email, get_user_by_id, User,
    },
    error_handling::ApiError,
    types::EMail,
    RedisBackend,
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Registration(PasskeyRegistration);
impl Extractable for Registration {
    type Rejection = ApiError;

    const PATH: &'static str = "$.webauthn_registration";
}

#[tracing::instrument(skip(webauthn_data, webauthn))]
pub(crate) async fn start_register(
    user_data: SessionData<RedisBackend, User>,
    webauthn_data: SessionData<RedisBackend, Option<Registration>>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
) -> Result<impl IntoResponse, ApiError> {
    let user = user_data.take_data();
    let res = match webauthn.start_passkey_registration(user.id, &user.email.0, &user.email.0, None)
    {
        Ok((ccr, reg_state)) => {
            webauthn_data.set(Registration(reg_state)).await?;
            Json(ccr)
        }
        Err(e) => {
            debug!(error=?e);
            return Err(ApiError::UnknownError(e.into()));
        }
    };
    Ok(res)
}

#[tracing::instrument(skip(pool, webauthn, webauthn_data, reg))]
pub(crate) async fn finish_register(
    Extension(pool): Extension<PgPool>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    webauthn_data: SessionData<RedisBackend, Registration>,
    user_data: SessionData<RedisBackend, User>,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, ApiError> {
    let reg_state = webauthn_data.take_data().0;
    let user = user_data.take_data();

    let passkey = webauthn
        .finish_passkey_registration(&reg, &reg_state)
        .map_err(|e| {
            error!("Unknown failure to finish registration: \n {e:#?}");
            ApiError::UnknownWebauthnError
        })?;
    add_passkey_for_user(&pool, &user.id, passkey).await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub(crate) struct AuthenticationStart {
    email: EMail,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct AuthenticationState {
    user_id: Uuid,
    authentication: PasskeyAuthentication,
}

impl Extractable for AuthenticationState {
    type Rejection = ApiError;

    const PATH: &'static str = "$.webauthn_auth";
}

#[tracing::instrument(skip(pool, webauthn, auth_data))]
pub(crate) async fn start_authentication(
    Extension(pool): Extension<PgPool>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    auth_data: SessionData<RedisBackend, Option<AuthenticationState>>,
    Json(AuthenticationStart { email }): Json<AuthenticationStart>,
) -> Result<impl IntoResponse, ApiError> {
    let Some(user) = get_user_by_email(&pool, &email).await? else {
        return Err(ApiError::UserNotFound);
    };

    let passkeys = get_passkeys_for_user(&pool, &user.id).await?;

    if passkeys.is_empty() {
        return Err(ApiError::NoPasskeysRegistered);
    }

    let res = match webauthn.start_passkey_authentication(&passkeys) {
        Ok((rcr, auth_state)) => {
            let session_id = auth_data
                .set(AuthenticationState {
                    user_id: user.id,
                    authentication: auth_state,
                })
                .await?
                .id();
            Json(json!({
                "session_id": session_id,
                "key_data": rcr,
            }))
        }
        Err(e) => {
            debug!("challenge_authenticate -> {:?}", e);
            return Err(ApiError::UnknownWebauthnError);
        }
    };
    Ok(res)
}

#[derive(Deserialize)]
pub(crate) struct AuthenticationEnd {
    credential: PublicKeyCredential,
}
#[tracing::instrument(skip(pool, webauthn, auth_data))]
pub(crate) async fn finish_authentication(
    Extension(pool): Extension<PgPool>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    auth_data: SessionData<RedisBackend, AuthenticationState>,
    user_data: SessionData<RedisBackend, Option<User>>,
    Json(AuthenticationEnd { credential }): Json<AuthenticationEnd>,
) -> Result<impl IntoResponse, ApiError> {
    let auth_state = auth_data.take_data();
    match webauthn.finish_passkey_authentication(&credential, &auth_state.authentication) {
        Ok(_auth_result) => {
            // TODO: Update counter
            let user_id = auth_state.user_id;
            let Some(user) = get_user_by_id(&pool, &user_id).await? else {
                return Err(ApiError::UserNotFound);
            };
            user_data.set(user.clone()).await?;

            Ok(Json(user))
        }
        Err(e) => {
            debug!(error=?e);
            Err(ApiError::UnknownWebauthnError)
        }
    }
}
