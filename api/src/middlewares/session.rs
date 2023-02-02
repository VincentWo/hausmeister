use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};

use redis::AsyncCommands;
use tower::{Layer, Service};
use tracing::debug;
use uuid::Uuid;

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
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let uuid = auth_header
            .as_bytes()
            .strip_prefix(b"Bearer ")
            .ok_or(StatusCode::BAD_REQUEST)?;

        let session_id = Uuid::try_parse_ascii(&uuid).map_err(|_| StatusCode::BAD_REQUEST)?;

        let redis_client = parts.extensions.get::<Arc<redis::Client>>().unwrap();
        let mut redis_connection = redis_client.get_async_connection().await.unwrap();

        if let Ok(user_uuid) = redis_connection
            .get::<_, String>(session_id.to_string())
            .await
        {
            if let Ok(_) = Uuid::try_parse(&user_uuid) {
                debug!("Restored session {} from cache", session_id);
                return Ok(AuthenticatedSession(session_id));
            }
        }

        let pool = parts.extensions.get::<sqlx::PgPool>().unwrap();

        match sqlx::query!("SELECT * FROM sessions WHERE id = $1", session_id)
            .fetch_optional(pool)
            .await
            .unwrap()
        {
            Some(record) => {
                let user_uuid = record.user_id;
                debug!("Caching session {}", session_id);
                redis_connection
                    .set::<_, _, ()>(session_id.to_string(), user_uuid.to_string())
                    .await
                    .unwrap();
                Ok(AuthenticatedSession(session_id))
            }
            None => Err(StatusCode::FORBIDDEN),
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
