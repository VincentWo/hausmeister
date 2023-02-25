use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, HeaderValue},
};
use color_eyre::{eyre::Context, Report};
use redis::{AsyncCommands, JsonAsyncCommands};
use redis_macros::Json;
use uuid::Uuid;

use crate::{database::User, error_handling::ApiError};

pub(crate) async fn remove_session(
    connection: &mut redis::aio::Connection,
    session_id: Uuid,
) -> Result<(), Report> {
    connection.del(session_id.to_string()).await?;

    Ok(())
}

pub(crate) async fn create_session(
    connection: &mut redis::aio::Connection,
    user: &User,
) -> Result<Uuid, Report> {
    let session_id = Uuid::new_v4();
    connection
        .hset(session_id.to_string(), "user", serde_json::to_string(user)?)
        .await?;

    Ok(session_id)
}

pub(crate) struct SessionStorage(Uuid);
pub(crate) struct SessionId(Uuid);

async fn get_redis_connection(parts: &mut Parts) -> Result<redis::aio::Connection, Report> {
    // if let Some(conn) = parts.extensions.get_mut::<redis::aio::Connection>() {
    // return Ok(conn);
    // }
    let client = parts
        .extensions
        .get::<Arc<redis::Client>>()
        .expect("Redis client is missing");
    let connection = client.get_async_connection().await?;

    // parts.extensions.insert(connection);

    Ok(connection)
}
// #[async_trait]
// impl<S> FromRequestParts<S> for SessionStorage
// where
//     S: Sync + Send,
// {
//     type Rejection = ApiError;

//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         let session_id =
//         let redis_con = get_redis_connection(parts).await?;
//     }
// }

// #[async_trait]
// impl<S> FromRequestParts<S> for SessionId
// where
//     S: Sync + Send,
// {
//     type Rejection = ApiError;

//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         match headers.get(AUTHORIZATION).map(extract_bearer)
//         let auth_header = parts
//             .headers
//             .get(AUTHORIZATION)
//             .ok_or(ApiError::NotLoggedIn)?;
//         let uuid =
//             auth_header
//                 .as_bytes()
//                 .strip_prefix(b"Bearer ")
//                 .ok_or(ApiError::MisformedAuth(Report::msg(
//                     "Missing Bearer Prefix",
//                 )))?;
//         let session_id =
//             Uuid::try_parse_ascii(uuid).map_err(|e| ApiError::MisformedAuth(e.into()))?;

//         let redis_client = parts
//             .extensions
//             .get::<Arc<redis::Client>>()
//             .expect("Redis Client is missing from extensions");
//         let mut redis_connection = redis_client
//             .get_async_connection()
//             .await
//             .wrap_err("Could not get redis async connection")?;
//         let Json(user): Json<Option<User>> = redis_connection
//             .json_get(session_id.to_string(), ".user")
//             .await
//             .wrap_err("Could not get user session")?;

//         Ok(SessionWithUser {
//             session_id,
//             user: user.unwrap(),
//         })
//     }
// }

mod middleware {
    use axum::{
        headers::{self, authorization::Bearer, Authorization},
        response::IntoResponse,
        TypedHeader,
    };

    use crate::error_handling::ApiError;

    // #[tracing::instrument]
    pub async fn extract_session_id(
        header: Result<TypedHeader<Authorization<Bearer>>, headers::Error>,
    ) -> Result<impl IntoResponse, ApiError> {
        header.
    }
}
