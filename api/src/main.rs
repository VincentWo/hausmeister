// in `src/main.rs`

use std::{error::Error, net::SocketAddr, sync::Arc};

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    routing::{get, patch, post},
    Extension, Router, Server, ServiceExt,
};

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
mod settings;
mod trace;
mod types;

use routes::login::login;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;

    color_eyre::install()?;
    trace::setup()?;

    let config = read_config()?;

    run_server(config).await?;
    trace::teardown();
    Ok(())
}

async fn run_server(config: Config) -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "[::1]:3779".parse()?;
    info!("Listening on http://{}", addr);

    let pool = database::connect(&config.database).await?;

    let redis_client = redis::Client::open("redis://localhost")?;

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
                    let config = request.extensions.get::<Arc<Config>>().unwrap();

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
        .layer(SessionLayer)
        .set_x_request_id(MakeRequestUuid)
        .propagate_x_request_id()
        .service(app);

    Server::bind(&addr).serve(svc.into_make_service()).await?;

    Ok(())
}
