//! Checking credentials
//!
//! This is the only file that is supposed to see and
//! interact with the passwords saved in the database.
//! Limiting this to this file allows easier changes
//! to hashing algorithms, security updates and helps
//! hiding passwords from attackers

use argon2::{password_hash, Argon2, PasswordHash, PasswordVerifier};
use color_eyre::Report;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{EMail, Password};

use super::User;

/// Create a new session
///
/// Does not check any credentials, use [check_credentials_and_get_user]
/// for that.
///
/// Note that this currently deletes an old session, which probably does
/// not make sense in a multi-device scenario, but we will think about
/// this once we are ready for multi-device.
#[tracing::instrument(skip(pool))]
async fn create_new_session(pool: &PgPool, user_id: &Uuid) -> Result<Uuid, Report> {
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

/// Expected errors during login
pub(crate) enum LoginError {
    /// User has not been found
    ///
    /// Currently user enumeration is trivially possible, should it?
    /// (Return "UserNotFound" allows for better UX), maybe configurable
    UserNotFound,
    /// Currently equal to wrong password.
    InvalidCredentials,
}

/// Unhashed Login Credentials
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Credentials {
    // Self-explanatory, doc would just be noise
    #[allow(clippy::missing_docs_in_private_items)]
    pub(crate) email: EMail,
    /// Unhashed password
    pub(crate) password: Password,
}

/// Checks credentials and returns user
///
/// The double result is used as always:
/// The outside result contains unexpected errors, the inner the expected ones
/// This function properly differentiates between a user not existing and
/// a credentials being wrong, see [LoginError] for details.
#[tracing::instrument]
async fn check_credentials_and_get_user(
    pool: &PgPool,
    credentials: Credentials,
) -> Result<Result<User, LoginError>, Report> {
    let Some(saved_user) = sqlx::query!("SELECT * FROM users WHERE email=$1", credentials.email.0)
        .fetch_optional(pool)
        .await? else {
        // Expected error, so outer Ok
        return Ok(Err(LoginError::UserNotFound));
    };

    let hash = PasswordHash::new(&saved_user.password)?;

    // All this double result stuff can be confusing, but the basic idea is
    // that we only return an outer error if something unexpected goes wrong
    // So InvalidCredentials are wrapped in Ok (since DB etc. did not have a problem),
    // but are still an Err
    let user_or_error = Argon2::default()
        .verify_password(credentials.password.0.as_bytes(), &hash)
        .map(|_| {
            Ok(User {
                id: saved_user.id,
                name: saved_user.name,
                email: EMail(saved_user.email),
            })
        })
        .or_else(|e| match e {
            password_hash::Error::Password => Ok(Err(LoginError::InvalidCredentials)),
            e => Err(e),
        })?;

    Ok(user_or_error)
}

/// A successfully created session
///
/// Contains the user data at the time of the creation
/// of the session - note that this data may become stale
/// at anytime if the user is deleted, so consider it "trusted", but
/// not nescessarily up-to-date.
#[derive(Serialize, Debug)]
pub(crate) struct Session {
    /// The session id/token
    pub(crate) session_id: Uuid,
    /// User data at session creation
    pub(crate) user: User,
}

/// Check credentials & create session
///
#[tracing::instrument]
pub(crate) async fn login_user(
    pool: &PgPool,
    credentials: Credentials,
) -> Result<Result<Session, LoginError>, Report> {
    let user = match check_credentials_and_get_user(pool, credentials).await? {
        Ok(user) => user,
        Err(err) => return Ok(Err(err)),
    };

    let session_id = create_new_session(pool, &user.id).await?;

    Ok(Ok(Session { user, session_id }))
}
