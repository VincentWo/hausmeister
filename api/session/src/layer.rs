use std::{
    future::Future,
    iter,
    task::{Context, Poll},
};

use axum::{body::HttpBody, response::IntoResponse};
use headers::Header;
use hyper::{header::AUTHORIZATION, Request};
use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::{debug, trace};

use crate::{backend::Backend, header::UncheckedSessionId, BoxError, SessionId};

#[pin_project(project = SessionProject, project_replace = SessionRepl)]
pub enum SessionState<S, Req, F>
where
    S: Service<Req>,
    F: Future,
{
    VerifyingSession {
        #[pin]
        verifying_future: F,
        request: Option<Req>,
        inner: Option<S>,
    },
    InInner(#[pin] S::Future),
    Invalid,
}

impl<S, Req, F, E> Future for SessionState<S, Req, F>
where
    S: Service<Req, Response = axum::response::Response>,
    S::Error: Into<BoxError>,
    F: Future<Output = Result<(), E>>,
    E: IntoResponse,
{
    type Output = Result<S::Response, BoxError>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (mut inner, request) = match self.as_mut().project() {
            SessionProject::VerifyingSession {
                verifying_future,
                inner,
                request,
            } => match verifying_future.poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(_) => {
                        let inner = inner
                            .take()
                            .expect("We transform the variant directly after this");
                        let request = request
                            .take()
                            .expect("We transform the variant directly after this");

                        (inner, request)
                    }
                    Err(err) => return Poll::Ready(Ok(err.into_response())),
                },
                Poll::Pending => return Poll::Pending,
            },
            SessionProject::InInner(inner_future) => match inner_future.poll(cx) {
                Poll::Ready(result) => return Poll::Ready(result.map_err(|e| e.into())),
                Poll::Pending => return Poll::Pending,
            },
            SessionProject::Invalid => {
                unreachable!("Panic while calling the inner service, in an invalid state now :(")
            }
        };

        // To give an error if we panic
        self.set(SessionState::Invalid);
        let inner_future = inner.call(request);
        self.set(SessionState::InInner(inner_future));

        self.poll(cx)
    }
}

#[derive(Debug, Clone)]
pub struct SessionService<S, B> {
    inner: S,
    backend: B,
}

impl<S, B, ReqBody> Service<Request<ReqBody>> for SessionService<S, B>
where
    S: Service<Request<ReqBody>, Response = axum::response::Response> + Clone,
    S::Error: Into<BoxError>,
    ReqBody: HttpBody,
    B: Backend + Send + Sync + Clone + 'static,
    B::Error: IntoResponse,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future =
        SessionState<S, Request<ReqBody>, impl Future<Output = Result<(), impl IntoResponse>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Our middleware doesn't care about backpressure so its ready as long
        // as the inner service is ready.
        self.inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        // request.extensions_mut().insert();

        request.extensions_mut().insert(self.backend.clone());
        let header_map = request.headers();
        if let Some(auth_header) = header_map.get(AUTHORIZATION) {
            match UncheckedSessionId::decode(&mut iter::once(auth_header)) {
                Ok(UncheckedSessionId(session_id)) => {
                    trace!("extracted session_id: {:#?}", session_id);
                    request.extensions_mut().insert(SessionId(session_id));

                    SessionState::VerifyingSession {
                        verifying_future: self.backend.verify_session(session_id),
                        request: Some(request),
                        inner: Some(self.inner.clone()),
                    }
                }
                Err(_) => {
                    debug!("Got invalid bearer header {:#?}", auth_header);

                    todo!("Handle invalid bearer")
                }
            }
        } else {
            trace!("Request without auth");
            SessionState::InInner(self.inner.call(request))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionLayer<Backend>(Backend);

impl<B> SessionLayer<B> {
    pub fn new(backend: B) -> Self {
        Self(backend)
    }
}
impl<S, B> Layer<S> for SessionLayer<B>
where
    B: Clone,
{
    type Service = SessionService<S, B>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionService {
            inner,
            backend: Clone::clone(&self.0),
        }
    }
}
