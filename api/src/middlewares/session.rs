use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};

use tower::{Layer, Service};
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
        let pool = parts.extensions.get::<sqlx::PgPool>().unwrap();
        let uuid = Uuid::try_parse_ascii(&uuid).map_err(|_| StatusCode::BAD_REQUEST)?;

        match sqlx::query!("SELECT * FROM sessions WHERE id = $1", uuid)
            .fetch_optional(pool)
            .await
            .unwrap()
        {
            Some(_) => Ok(AuthenticatedSession(uuid)),
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
