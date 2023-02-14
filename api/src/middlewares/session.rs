//! Session extraction & Routeguarding

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use axum_extra::extract::CookieJar;
use color_eyre::{eyre::Context, Report};
use redis::AsyncCommands;
use tracing::debug;
use uuid::Uuid;

use crate::error_handling::ApiError;

/// Extractor requiring the client to be logged in.
///
/// This extracts the session id, **not** the user id!
/// (Probably a good idea to somehow type this better)
/// Currently to get any info about the user we need to
/// call [get_user_from_session](crate::database::get_user_from_session)
/// - note that this allows a race condition if the session get's deleted
/// between the two calls - so probably something we should fix on the
/// side of the middleware
#[derive(Clone, Debug)]
pub(crate) struct AuthenticatedSession(
    /// The session id
    pub(crate) Uuid,
);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedSession
where
    S: Sync + Send,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let cookies = CookieJar::from_headers(&parts.headers);
        let uuid = cookies.get("id").ok_or(ApiError::NotLoggedIn)?.value();

        let session_id = Uuid::try_parse(uuid).map_err(|e| ApiError::MisformedAuth(e.into()))?;

        let redis_client = parts
            .extensions
            .get::<Arc<redis::Client>>()
            .expect("Redis Client is missing from extensions");
        let mut redis_connection = redis_client
            .get_async_connection()
            .await
            .wrap_err("Could not get redis async connection")?;

        if let Ok(user_uuid) = redis_connection
            .get::<_, String>(session_id.to_string())
            .await
        {
            if Uuid::try_parse(&user_uuid).is_ok() {
                debug!("Restored session {} from cache", session_id);
                return Ok(AuthenticatedSession(session_id));
            }
        }

        let pool = parts
            .extensions
            .get::<sqlx::PgPool>()
            .expect("Missing PgPool from Extensions");

        match sqlx::query!("SELECT * FROM sessions WHERE id = $1", session_id)
            .fetch_optional(pool)
            .await
            .wrap_err("Retrieving session from DB")?
        {
            Some(record) => {
                let user_uuid = record.user_id;
                debug!("Caching session {}", session_id);
                redis_connection
                    .set::<_, _, ()>(session_id.to_string(), user_uuid.to_string())
                    .await
                    .wrap_err("Retrieving session from Redis")?;
                Ok(AuthenticatedSession(session_id))
            }
            None => Err(ApiError::NotLoggedIn),
        }
    }
}
