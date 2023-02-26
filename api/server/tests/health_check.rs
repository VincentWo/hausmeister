use std::collections::HashSet;

use hausmeister::{
    create_app,
    settings::{AppConfig, Config, DbConfig},
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::{postgres::PgConnectOptions, Connection, Executor, PgConnection};
use tokio::spawn;
use uuid::Uuid;

async fn spawn_app() -> (String, reqwest::Client, PgConnection) {
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
            allowed_origins: HashSet::new(),
            allow_localhost: true,
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
    let password = "1234";
    let response = client
        .post(format!("{base_url}/register"))
        .json(&json!({
            "email": email,
            "name": name,
            "password": password,
        }))
        .send()
        .await
        .expect("Failed to send request");

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
                "password": "password"
            }),
            "missing the email",
        ),
        (
            json!({
                "email": "test@example.com",
                "password": "password",
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
