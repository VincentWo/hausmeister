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
#![feature(lint_reasons, rustdoc_missing_doc_code_examples)]
#![doc = include_str!("../README.md")]

use std::{net::SocketAddr, sync::Arc};

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    routing::{get, patch, post},
    Extension, Router, Server, ServiceExt,
};

use color_eyre::Report;
use settings::{read_config, Config};
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::info;

use crate::{
    database::{auth::Credentials, create_admin_if_no_user_exist},
    routes::{
        login::{logout, test_login},
        reset::{request_reset, reset_password, test_reset_token},
        user::{get_user, patch_user},
    },
    types::{EMail, Password},
};

mod database;
mod error_handling;
mod middlewares;
mod routes;
mod settings;
mod trace;
mod types;

use routes::login::login;

/// Run the complete application
///
/// Only public function at the moment - should be changed to accept the
/// settings to move setting loading into the application
pub async fn run() -> Result<(), Report> {
    dotenv::dotenv()?;

    color_eyre::install()?;
    trace::setup()?;

    let config = read_config()?;

    run_server(config).await?;

    Ok(())
}

/// Start the server with the given configuration
async fn run_server(config: Config) -> Result<(), Report> {
    let addr: SocketAddr = "[::1]:3779".parse()?;
    info!("Listening on http://{}", addr);

    let pool = database::connect(&config.database).await?;

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
        .route("/test_login", get(test_login))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/request-reset", post(request_reset))
        .route("/reset", post(reset_password))
        .route("/user", get(get_user))
        .route("/user", patch(patch_user))
        .route("/test_reset_token", post(test_reset_token));

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
                    let config = request
                        .extensions
                        .get::<Arc<Config>>()
                        .expect("Config is missing from extensions");

                    if config.app.allowed_origins.contains(origin) {
                        true
                    } else {
                        config.app.allow_localhost
                            && (origin.starts_with("http://localhost")
                                || origin.starts_with("https://localhost"))
                    }
                }))
                .allow_headers([CONTENT_TYPE, AUTHORIZATION])
                .allow_credentials(true),
        )
        .layer(Extension(pool))
        .layer(Extension(Arc::new(redis_client)))
        .set_x_request_id(MakeRequestUuid)
        .propagate_x_request_id()
        .service(app);

    Server::bind(&addr).serve(svc.into_make_service()).await?;

    Ok(())
}
