use std::{collections::HashSet, net::Ipv6Addr};

use hausmeister::{
    create_app,
    settings::{AppConfig, Config, DbConfig, RedisConfig},
    trace,
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::{postgres::PgConnectOptions, Connection, Executor, PgConnection};
use tokio::spawn;
use uuid::Uuid;

async fn spawn_app() -> (String, reqwest::Client, PgConnection) {
    if std::env::var("TEST_LOG").is_ok() {
        trace::setup().expect("Setting up trace");
    }
    let db_name = Uuid::new_v4().to_string();

    let config = Config {
        database: DbConfig::Parameters {
            username: "hausmeister".to_owned(),
            password: "password".to_owned(),
            host: "localhost".to_owned(),
            port: None,
            // First connect without Database since we need to create it & run migrations
            db_name: Some(db_name.clone()),
        },
        app: AppConfig {
            port: 0,
            listen_on: Ipv6Addr::LOCALHOST.into(),
            allowed_origins: HashSet::new(),
            allow_localhost: true,
        },
        redis: RedisConfig {
            url: "redis://localhost".parse().unwrap(),
        },
    };

    let connect_options: PgConnectOptions =
        TryInto::try_into(&config.database).expect("Creating PgConnectOptions");

    let mut db_client =
        PgConnection::connect_with(&connect_options.clone().database("hausmeister"))
            .await
            .expect("Connecting to DB");

    db_client
        .execute(format!(r#"CREATE DATABASE  "{db_name}" "#).as_str())
        .await
        .expect("Failed creating database");

    let mut db_client = PgConnection::connect_with(&connect_options)
        .await
        .expect("Connecting to DB");

    sqlx::migrate!("./migrations")
        .run(&mut db_client)
        .await
        .expect("Failed to run migrations");

    let (addr, app) = create_app(config).await.expect("Failed to create app");

    spawn(async { app.await.expect("Running the server") });

    (format!("http://{addr}"), reqwest::Client::new(), db_client)
}

#[tokio::test]
async fn health_check_works() {
    let (base_url, client, _) = spawn_app().await;

    let response = client
        .get(format!("{base_url}/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn register_works() {
    let (base_url, client, mut db_client) = spawn_app().await;

    let email = "test@example.com";
    let name = "Test User";
    let password = "12345678910";
    let response = client
        .post(format!("{base_url}/register"))
        .json(&json!({
            "email": email,
            "name": name,
            "password": password,
        }))
        .send()
        .await
        .expect("Failed to send request")
        .error_for_status()
        .expect("Register failed with error status");

    let user = sqlx::query!("SELECT * FROM USERS WHERE email = $1", email)
        .fetch_one(&mut db_client)
        .await
        .expect("Failed to fetch created user");

    assert_eq!(user.email, email);
    assert_eq!(user.name, name);

    assert!(response.status().is_success());
}

#[tokio::test]
async fn register_returns_400_when_data_is_missing() {
    let (base_url, client, _) = spawn_app().await;

    let test_cases = [
        (
            json!({
                "name": "Test User",
                "password": "test_password"
            }),
            "missing the email",
        ),
        (
            json!({
                "email": "test@example.com",
                "password": "test_password",
            }),
            "missing the name",
        ),
        (
            json!({
                "email": "test@example.com",
                "name": "User",
            }),
            "missing the password",
        ),
    ];

    for (data, reason) in test_cases {
        let response = client
            .post(format!("{base_url}/register"))
            .json(&data)
            .send()
            .await
            .expect("Sending the request");

        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 422 Unprocessable Entity when the payload was {reason}"
        )
    }
}

#[tokio::test]
async fn register_returns_a_422_when_fields_are_present_but_invalid() {
    let (url, client, _) = spawn_app().await;

    // This only handles name + email, password validation is more complex
    // so we have a different test for that
    let test_cases = [
        (
            json! ({
                "email": "",
                "name": "Joshua",
                "password": "test_password"
            }),
            "Missing the e-mail",
        ),
        (
            json! ({
                "email": "test@example.com",
                "name": "",
                "password": "test_password"
            }),
            "Missing the name",
        ),
        (
            json! ({
                "email": "not-an-email",
                "name": "Hannah",
                "password": "test_password"
            }),
            "Invalid e-mail",
        ),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(format!("{url}/register"))
            .json(&body)
            .send()
            .await
            .expect("Failed to reach the app");
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not return a 422 when the payload was {description}",
        );
    }
}
