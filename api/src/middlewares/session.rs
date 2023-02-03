use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use color_eyre::{eyre::Context, Report};
use redis::AsyncCommands;
use tower::{Layer, Service};
use tracing::debug;
use uuid::Uuid;

use crate::error_handling::ApiError;

pub(crate) struct SessionLayer;

impl<S> Layer<S> for SessionLayer {
    type Service = Session<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Session { inner }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AuthenticatedSession(pub(crate) Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedSession
where
    S: Sync + Send,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(ApiError::NotLoggedIn)?;
        let uuid =
            auth_header
                .as_bytes()
                .strip_prefix(b"Bearer ")
                .ok_or(ApiError::MisformedAuth(Report::msg(
                    "Missing Bearer Prefix",
                )))?;

        let session_id =
            Uuid::try_parse_ascii(uuid).map_err(|e| ApiError::MisformedAuth(e.into()))?;

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

#[derive(Clone, Debug)]
pub(crate) struct Session<S> {
    inner: S,
}

impl<S, Request> Service<Request> for Session<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.inner.call(req)
    }
}
