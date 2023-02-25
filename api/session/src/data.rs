use core::fmt;
use std::any::type_name;

use axum::{async_trait, extract::FromRequestParts, response::IntoResponse};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::{
    backend::{self, Backend, Error as _},
    SessionId,
};

pub struct SessionData<B, S> {
    backend: B,
    data: S,
    session_id: Option<SessionId>,
}

impl<B, D> fmt::Debug for SessionData<B, D>
where
    D: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionData")
            .field("data", &self.data)
            .field("session_id", &self.session_id)
            .field("backend", &format_args!("{}", type_name::<B>()))
            .finish()
    }
}

impl<B, D> SessionData<B, D>
where
    B: Backend,
    D: Extractable,
    D: Serialize + Sync,
{
    #[tracing::instrument(skip(self))]
    pub async fn remove_session(self) -> Result<(), B::Error> {
        self.backend.remove_session(self.id()).await
    }
    pub fn take_data(self) -> D {
        self.data
    }
    #[tracing::instrument(skip(self, new_data))]
    pub async fn set(&mut self, new_data: D) -> Result<D, B::Error> {
        self.backend.set_data(self.id(), D::PATH, &new_data).await?;
        Ok(std::mem::replace(&mut self.data, new_data))
    }
    pub fn id(&self) -> Uuid {
        self.session_id
            .as_ref()
            .expect("Has to be set if data is not Option")
            .0
    }

    pub fn get(&self) -> &D {
        &self.data
    }
    pub fn get_mut(&mut self) -> &mut D {
        &mut self.data
    }
}

impl<B, D> SessionData<B, Option<D>>
where
    D: Extractable + Serialize + Sync,
    B: Backend + Clone,
{
    pub async fn set(self, data: D) -> Result<SessionData<B, D>, B::Error> {
        let session_id = match self.session_id {
            Some(session_id) => session_id,
            None => SessionId(self.backend.create_session().await?),
        };

        self.backend.set_data(session_id.0, D::PATH, &data).await?;
        Ok(SessionData {
            backend: self.backend,
            data,
            session_id: Some(session_id),
        })
    }
}

pub trait Extractable: DeserializeOwned {
    type Rejection: IntoResponse;
    const PATH: &'static str;
}

#[async_trait]
impl<B, S, D> FromRequestParts<S> for SessionData<B, D>
where
    S: Send + Sync,
    B: Clone + Send + Sync + Backend + 'static,
    B::Error: backend::Error,
    D: Extractable,
{
    type Rejection = B::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let backend = parts.extensions.get::<B>().unwrap().clone();
        let Some(session_id) = parts.extensions.get::<SessionId>() else {
            return Err(B::Error::no_session())
        };

        let Some(data) = backend.get_data::<D>(session_id.0, D::PATH).await? else {
            return Err(B::Error::missing_data(D::PATH))
        };

        Ok(Self {
            backend,
            data,
            session_id: Some(*session_id),
        })
    }
}

#[async_trait]
impl<B, S, D> FromRequestParts<S> for SessionData<B, Option<D>>
where
    S: Send + Sync,
    B: Clone + Send + Sync + Backend + 'static,
    B::Error: backend::Error,
    D: Extractable,
{
    type Rejection = B::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let backend = parts.extensions.get::<B>().unwrap().clone();
        let Some(session_id) = parts.extensions.get::<SessionId>() else {
            return Ok(Self {
                backend,
                data: None,
                session_id: None,
            });
        };

        let data = backend.get_data::<D>(session_id.0, D::PATH).await?;

        Ok(Self {
            backend,
            data,
            session_id: Some(*session_id),
        })
    }
}
