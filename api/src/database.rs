//! All methods to talk to the database reside here.
//!
//! This makes any changes to tables, relations etc. easier.
//! Currently this module is also responsible for password hashing,
//! but this should propably be moved to a different module.

pub(crate) mod auth;

use std::time::Duration;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

use color_eyre::{
    eyre::{eyre, Context},
    Report,
};

use redis::{aio::Connection, AsyncCommands};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use tracing::{debug, debug_span, info, Instrument};
use uuid::Uuid;

use crate::{
    settings::DbConfig,
    types::{EMail, Password},
};

use self::auth::Credentials;

/// This directly mirrors the `users` table, expect for the password
/// column, since we don't want to return a password on accident
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug, Serialize)]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) email: EMail,
}

/// This is an subset of [User], containing all the updatable properties
/// as [Options](::std::option), usefull for allowing partial updates.
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug, Deserialize)]
pub(crate) struct UserUpdate {
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
}

/// The same as [User], just including a password where it is
/// required
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug, Serialize)]
pub(crate) struct UserWithPassword {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) email: EMail,
    pub(crate) password: Password,
}

/// Connects to the database given by `config`, setting the application
/// name to "hausmeister"
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

/// Returns the number of registered users
#[tracing::instrument(skip(pool))]
pub(crate) async fn count_user(pool: &PgPool) -> Result<i64, Report> {
    sqlx::query!("select Count(*) from users")
        .fetch_one(pool)
        .await?
        .count
        .ok_or_else(|| eyre!("Count was None (should not happen)"))
}

/// If no user exists, this tries to create a new admin user
/// with the given credentials.
///
/// Not creating the admin is not considered a failure
/// since it is assumed that this is only desirable on
/// new installations.
///
/// # Note:
/// This method can include a (safe) race condition if running
/// multiple instances of hausmeister connecting to
/// the same database and specifying different credentials,
/// it is not specified nor predictable how many admins
/// will be created and which ones, only that it is
/// at least one. To prevent that make sure that
/// the same admin + password is chosen by all instances
/// (and this method probably needs to be removed for
/// security reasons anyway)
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

#[tracing::instrument(skip(pool))]
pub(crate) async fn get_user_by_email(
    pool: &PgPool,
    email: &EMail,
) -> Result<Option<User>, Report> {
    let db_user = sqlx::query!("SELECT * FROM users WHERE email=$1", email.0)
        .fetch_optional(pool)
        .await?;

    Ok(db_user.map(|db_user| User {
        id: db_user.id,
        name: db_user.name,
        email: EMail(db_user.email),
    }))
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

/// The known errors which can occur when calling [reset_password]
pub(crate) enum ResetError {
    /// The reset token does not exist so no password was reset
    TokenNotFound,
}

/// Resets the password of the user associated with the given reset token.
///
/// This returns a result in a result: The outer result is for any "unknown"
/// errors and a failure here should result in an 500, the inner result is
/// for errors that have a concrete reason and can be fixed by the caller.
///
/// See [ResetError] for the possible failures.
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

#[tracing::instrument(skip(pool, redis_connection))]
pub(crate) async fn remove_session(
    pool: &PgPool,
    redis_connection: &mut Connection,
    id: &Uuid,
) -> Result<(), Report> {
    sqlx::query!("DELETE FROM sessions WHERE id = $1", id)
        .execute(pool)
        .await?;
    redis_connection.del::<_, ()>(id.to_string()).await?;

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

#[tracing::instrument(skip(pool))]
pub(crate) async fn update_current_user(
    pool: &PgPool,
    session_id: &Uuid,
    update: UserUpdate,
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
