// in `src/main.rs`

use std::{error::Error, net::SocketAddr};

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    routing::{get, patch, post},
    Extension, Router, Server, ServiceExt,
};

use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::info;

use crate::{
    database::create_admin_if_no_user_exist,
    middlewares::session::SessionLayer,
    routes::{
        login::{logout, test_login, Credentials},
        reset::{request_reset, reset_password, test_reset_token},
        root,
        user::{get_user, patch_user},
    },
    types::{EMail, Password},
};

mod database;
mod error_handling;
mod middlewares;
mod request_id;
mod routes;
mod trace;
mod types;

use routes::login::login;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    trace::setup()?;
    run_server().await?;
    trace::teardown();
    Ok(())
}

async fn run_server() -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "0.0.0.0:3779".parse()?;
    info!("Listening on http://{}", addr);

    let pool = database::connect().await?;

    tokio::spawn({
        let pool = pool.clone();
        async move {
            let pool = pool;
            create_admin_if_no_user_exist(
                &pool,
                &Credentials {
                    email: EMail("admin@example.com".to_owned()),
                    password: Password("password".to_owned()),
                },
            )
            .await
            .unwrap();
        }
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/test_login", get(test_login))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/request-reset", post(request_reset))
        .route("/reset", post(reset_password))
        .route("/user", get(get_user))
        .route("/user", patch(patch_user))
        .route("/test_reset_token", post(test_reset_token));

    let svc = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .propagate_x_request_id()
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH])
                .allow_origin([
                    "http://localhost:5173".try_into().unwrap(),
                    "http://localhost:3001".try_into().unwrap(),
                ])
                .allow_headers([CONTENT_TYPE, AUTHORIZATION])
                .allow_credentials(true),
        )
        .layer(Extension(pool))
        .layer(SessionLayer)
        .service(app);

    Server::bind(&addr).serve(svc.into_make_service()).await?;

    Ok(())
}
