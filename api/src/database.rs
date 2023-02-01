use std::time::Duration;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

use color_eyre::{
    eyre::{eyre, Context},
    Report,
};

use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use tracing::{debug, debug_span, info, Instrument};
use uuid::Uuid;

use crate::{
    routes::login::Credentials,
    settings::DbConfig,
    types::{EMail, Password},
};

#[derive(Debug, Serialize)]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) email: EMail,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserWithPassword {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) email: EMail,
    pub(crate) password: Password,
}

#[tracing::instrument(skip(config))]
pub(crate) async fn connect(config: &DbConfig) -> color_eyre::Result<PgPool> {
    let options = config
        .url
        .parse::<PgConnectOptions>()
        .wrap_err("Failed parsing database URL")?
        .application_name("hausmeister");

    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .instrument(debug_span!("Connecting to DB"))
        .await
        .wrap_err("Connecting to database")
}
#[tracing::instrument(skip(pool))]
pub(crate) async fn count_user(pool: &PgPool) -> Result<i64, Report> {
    sqlx::query!("select Count(*) from users")
        .fetch_one(pool)
        .await?
        .count
        .ok_or_else(|| eyre!("Count was None (should not happen)"))
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn create_admin_if_no_user_exist(
    pool: &PgPool,
    Credentials { password, email }: &Credentials,
) -> Result<(), Report> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(password.0.as_bytes(), &salt)?
        .to_string();

    if count_user(pool).await? == 0 {
        debug!("No user exist: Creating some.");
        let query_result = sqlx::query!(
            r#"
    INSERT INTO users (id, email, password, name) VALUES ($1, $2, $3, 'Admin')
        ON CONFLICT DO NOTHING"#,
            Uuid::new_v4(),
            email.0,
            hash,
        )
        .execute(pool)
        .await?;

        if query_result.rows_affected() == 0 {
            debug!("Other instance already created an admin");
        } else {
            info!("Successfully created admin user");
        }
    } else {
        debug!("User already exist - doing nothing");
    }

    Ok(())
}
#[tracing::instrument(skip(_pool))]
pub(crate) async fn list_user(_pool: &PgPool) -> Result<Vec<UserWithPassword>, Report> {
    // let user: Vec<_> = sqlx::query!("SELECT * FROM users")
    // .fetch(pool)
    // .try_collect()
    // .await?;

    todo!();
    // Ok(user)
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn get_user_by_email(
    pool: &PgPool,
    email: &EMail,
) -> Result<Option<UserWithPassword>, Report> {
    let db_user = sqlx::query!("SELECT * FROM users WHERE email=$1", email.0)
        .fetch_optional(pool)
        .await?;

    Ok(db_user.map(|db_user| UserWithPassword {
        id: db_user.id,
        name: db_user.name,
        password: Password(db_user.password),
        email: EMail(db_user.email),
    }))
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn create_new_session(pool: &PgPool, user_id: &Uuid) -> Result<Uuid, Report> {
    let session_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO
            sessions (id, user_id)
        VALUES
            ($1, $2)
        ON CONFLICT(user_id) DO
            UPDATE SET
                id = EXCLUDED.id,
                user_id = EXCLUDED.user_id,
                created_at = EXCLUDED.created_at",
        session_id,
        user_id,
    )
    .execute(pool)
    .await?;

    Ok(session_id)
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn new_reset_request(pool: &PgPool, user_id: &Uuid) -> Result<Uuid, Report> {
    let reset_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO
            password_reset_requests (id, user_id)
        VALUES
            ($1, $2)
        ON CONFLICT(user_id) DO
            UPDATE SET
                id = EXCLUDED.id,
                user_id = EXCLUDED.user_id,
                created_at = EXCLUDED.created_at",
        reset_id,
        user_id,
    )
    .execute(pool)
    .await?;

    Ok(reset_id)
}

pub(crate) enum ResetError {
    TokenNotFound,
}
#[tracing::instrument(skip(pool))]
pub(crate) async fn reset_password(
    pool: &PgPool,
    reset_token: &Uuid,
    new_password: &Password,
) -> Result<Result<(), ResetError>, Report> {
    let mut transaction = pool.begin().await?;

    let reset_request = sqlx::query!(
        "DELETE FROM password_reset_requests WHERE id = $1 RETURNING user_id",
        reset_token
    )
    .fetch_optional(&mut transaction)
    .await?;

    let Some(reset_request) = reset_request else {
        return Ok(Err(ResetError::TokenNotFound));
    };

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(new_password.0.as_bytes(), &salt)?
        .to_string();

    sqlx::query!(
        "UPDATE users
            SET password = $1
            WHERE id = $2",
        hash,
        reset_request.user_id,
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(Ok(()))
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn reset_token_is_valid(
    pool: &PgPool,
    reset_token: &Uuid,
) -> Result<bool, Report> {
    Ok(sqlx::query!(
        "SELECT * FROM password_reset_requests WHERE id = $1",
        reset_token,
    )
    .fetch_optional(pool)
    .await?
    .is_some())
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn remove_session(pool: &PgPool, id: &Uuid) -> Result<(), Report> {
    sqlx::query!("DELETE FROM sessions WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn get_user_from_session(
    pool: &PgPool,
    id: &Uuid,
) -> Result<Option<User>, Report> {
    Ok(sqlx::query!(
        "SELECT users.id, name, email FROM sessions INNER JOIN users ON (user_id=users.id) WHERE sessions.id = $1",
        id
    ).fetch_optional(pool)
    .await?
    .map(|db_user| {
            User {
                id: db_user.id,
                name: db_user.name,
                email: EMail(db_user.email),
            }
        })
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct PatchUser {
    name: Option<String>,
    email: Option<String>,
}
#[tracing::instrument(skip(pool))]
pub(crate) async fn update_current_user(
    pool: &PgPool,
    session_id: &Uuid,
    update: PatchUser,
) -> Result<Option<User>, Report> {
    let user = sqlx::query!(
        "UPDATE
            users
        SET
            name = coalesce($2, name),
            email = coalesce($3, email)
        FROM
            sessions
        WHERE
            sessions.id = $1
        RETURNING
            users.id, name, email",
        session_id,
        update.name,
        update.email,
    )
    .fetch_optional(pool)
    .await?
    .map(|db_user| User {
        id: db_user.id,
        name: db_user.name,
        email: EMail(db_user.email),
    });

    Ok(user)
}
