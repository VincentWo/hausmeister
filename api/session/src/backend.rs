use std::future::Future;

use axum::response::IntoResponse;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

pub trait Backend: Send + Sync {
    type Error: Error;
    fn verify_session(
        &self,
        session_id: Uuid,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn create_session(&self) -> impl Future<Output = Result<Uuid, Self::Error>> + Send;
    fn remove_session(&self, id: Uuid) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn get_data<T: DeserializeOwned>(
        &self,
        session_id: Uuid,
        path: &'static str,
    ) -> impl Future<Output = Result<Option<T>, Self::Error>> + Send;

    fn set_data<'a, T: Serialize + Sync>(
        &self,
        session_id: Uuid,
        path: &'static str,
        data: &'a T,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a;
}

pub trait Error: Send + Sync + IntoResponse {
    fn no_session() -> Self;
    fn missing_data(path: &str) -> Self;
}
