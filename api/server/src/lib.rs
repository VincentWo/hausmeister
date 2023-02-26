#![forbid(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::as_conversions,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cognitive_complexity,
    clippy::unwrap_used,
    clippy::branches_sharing_code,
    clippy::fallible_impl_from,
    clippy::filetype_is_file,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::fn_params_excessive_bools,
    clippy::format_push_string,
    clippy::from_iter_instead_of_collect,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::items_after_statements,
    clippy::iter_not_returning_iterator,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_with_drain,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::manual_assert,
    clippy::manual_clamp,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::map_err_ignore
)]
#![allow(incomplete_features)]
#![feature(
    lint_reasons,
    rustdoc_missing_doc_code_examples,
    return_position_impl_trait_in_trait
)]
#![doc = include_str!("../README.md")]

use std::{
    any::type_name,
    future::Future,
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpListener},
    sync::Arc,
};

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    routing::{get, patch, post},
    Extension, Router, Server, ServiceExt,
};

use color_eyre::{eyre::Context, Report};
use error_handling::ApiError;
use futures::FutureExt;
use redis::{AsyncCommands, JsonAsyncCommands};

use serde::{de::DeserializeOwned, Serialize};

use settings::Config;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{debug, debug_span, info, trace, Instrument};
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
    database::{auth::Credentials, create_admin_if_no_user_exist},
    routes::{
        healthcheck::health_check,
        login::{logout, test_login},
        register::register,
        reset::{request_reset, reset_password, test_reset_token},
        user::{get_user, patch_user},
        webauthn::{finish_authentication, finish_register, start_authentication, start_register},
    },
    types::{EMail, Password},
};

mod database;
mod error_handling;
mod routes;
pub mod settings;
pub mod trace;
mod types;
mod webauthn;

use routes::login::login;

/// Run the complete application
///
/// Only public function at the moment - should be changed to accept the
/// settings to move setting loading into the application
pub async fn create_app(
    config: Config,
) -> Result<(SocketAddr, impl Future<Output = Result<(), Report>> + Send), Report> {
    run_server(config).await
}

/// Start the server with the given configuration
async fn run_server(
    config: Config,
) -> Result<(SocketAddr, impl Future<Output = Result<(), Report>> + Send), Report> {
    let addr = SocketAddrV6::new("::1".parse()?, config.app.port, 0, 0);
    let listener = TcpListener::bind(addr)?;
    let addr = listener.local_addr()?;

    info!("Listening on http://{}", addr);

    let pool = database::connect(&config.database).await?;
    let webauthn = webauthn::setup(&config.app)?;

    let redis_client = redis::Client::open("redis://localhost")?;

    create_admin_if_no_user_exist(
        &pool,
        &Credentials {
            email: EMail("admin@example.com".to_owned()),
            password: Password("password".to_owned()),
        },
    )
    .await?;

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/test_login", get(test_login))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/register", post(register))
        .route("/request-reset", post(request_reset))
        .route("/reset", post(reset_password))
        .route("/user", get(get_user))
        .route("/user", patch(patch_user))
        .route("/test_reset_token", post(test_reset_token))
        .route("/webauthn/start_register", post(start_register))
        .route("/webauthn/finish_register", post(finish_register))
        .route("/webauthn/start_authentication", post(start_authentication))
        .route(
            "/webauthn/finish_authentication",
            post(finish_authentication),
        );

    let redis_client = Arc::new(redis_client);
    let svc = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .layer(Extension(Arc::new(config)))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH])
                .allow_origin(AllowOrigin::predicate(|header, request| {
                    let Ok(origin) = header.to_str() else {
                        // We don't allow non utf-origins at the moment
                        return false;
                    };
                    let Ok(origin) = Url::parse(origin) else {
                        return false;
                    };
                    let config = request
                        .extensions
                        .get::<Arc<Config>>()
                        .expect("Config is missing from extensions");

                    if config.app.allowed_origins.contains(&origin) {
                        true
                    } else {
                        config.app.allow_localhost && origin.host_str() == Some("localhost")
                    }
                }))
                .allow_headers([CONTENT_TYPE, AUTHORIZATION])
                .allow_credentials(true),
        )
        .layer(Extension(pool))
        .layer(Extension(Arc::new(webauthn)))
        .layer(Extension(Arc::clone(&redis_client)))
        .set_x_request_id(MakeRequestUuid)
        .layer(session::SessionLayer::new(RedisBackend {
            client: redis_client,
        }))
        .propagate_x_request_id()
        .service(app);

    Ok((
        addr,
        Server::from_tcp(listener)?
            .serve(svc.into_make_service())
            .map(|r| r.map_err(|e| e.into())),
    ))
}

#[derive(Clone, Debug)]
struct RedisBackend {
    client: Arc<redis::Client>,
}

impl session::backend::Backend for RedisBackend {
    type Error = ApiError;

    fn verify_session(&self, session_id: Uuid) -> impl Future<Output = Result<(), Self::Error>> {
        let redis_client = Arc::clone(&self.client);

        async move {
            let mut connection = redis_client
                .get_async_connection()
                .await
                .wrap_err("Getting redis connection")?;

            let exists: bool = connection
                .exists(session_id.to_string())
                .await
                .wrap_err("Trying to get session")?;

            debug!(is_valid = exists);

            exists.then_some(()).ok_or(ApiError::InvalidSession)
        }
        .instrument(debug_span!("Verifying session", ?session_id))
    }
    fn create_session(&self) -> impl Future<Output = Result<Uuid, Self::Error>> + Send {
        let redis_client = Arc::clone(&self.client);
        async move {
            let mut connection = redis_client
                .get_async_connection()
                .await
                .wrap_err("Getting redis connection")?;

            let session_id = Uuid::new_v4();

            connection
                .json_set(session_id.to_string(), ".", &serde_json::Map::new())
                .await
                .wrap_err("Setting root object")?;

            debug!(?session_id);

            Ok(session_id)
        }
        .instrument(debug_span!("Creating session"))
    }
    fn set_data<'a, T: Serialize + Sync>(
        &self,
        session_id: Uuid,
        path: &'static str,
        data: &'a T,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        let redis_client = Arc::clone(&self.client);
        async move {
            let redis_client = redis_client;
            debug!("Getting redis connection to set_data");
            let mut connection = redis_client
                .get_async_connection()
                .await
                .wrap_err("Getting redis connection")?;
            connection
                .json_set(session_id.to_string(), path, &data)
                .await
                .wrap_err("Setting redis data")?;

            Ok(())
        }
        .instrument(tracing::debug_span!(
            "Setting Session Data",
            ?session_id,
            path,
        ))
    }

    fn get_data<T: DeserializeOwned>(
        &self,
        session_id: Uuid,
        path: &'static str,
    ) -> impl Future<Output = Result<Option<T>, Self::Error>> + Send {
        let redis_client = Arc::clone(&self.client);
        async move {
            let redis_client = redis_client;
            let mut connection = redis_client
                .get_async_connection()
                .await
                .wrap_err("Getting redis connection")?;

            let json: String = connection
                .json_get(session_id.to_string(), path)
                .await
                .wrap_err("Getting value from path")?;
            trace!(json = json, target_type = type_name::<T>());

            // TODO: What to do about vec with multiple entries
            let mut value =
                serde_json::from_str::<Vec<T>>(&json).wrap_err("Deserializing value from Redis")?;

            Ok(value.pop())
        }
        .instrument(tracing::info_span!(
            "Getting Session Data",
            ?session_id,
            path
        ))
    }

    fn remove_session(
        &self,
        session_id: Uuid,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        let redis_client = Arc::clone(&self.client);
        async move {
            let mut connection = redis_client
                .get_async_connection()
                .await
                .wrap_err("Getting redis connection")?;

            connection
                .del(session_id.to_string())
                .await
                .wrap_err("Deleting session")?;

            Ok(())
        }
        .instrument(tracing::debug_span!("Removing session", ?session_id))
    }
}
