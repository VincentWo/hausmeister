//! Methods for handling, logging and returning errors
//!
//! This module most interesting type is [ApiError], it
//! can be returned as an error from any route since it
//! implements [IntoResponse](axum::response::IntoResponse).
//!
//! # General thoughts on error handling
//! Error handling is hard, so much is certain. We found
//! (hopefully) satisfying solutions for most problems,
//! but gaps remain:
//!
//! ## Internal error handling
//! For internal error handling we differentiate between
//! "server" and "client" errors. Client-errors
//! use enums (though we don't use something like this-error at the moment,
//! why?) and server-errors use [color_eyre] to produce pretty back- &
//! span-traces (usefull in async).
//!
//! We also don't refrain from mixing these, either with
//! `Result<Result<_, _>, _>` or with an enum with variants containing
//! [color_eyre::Report].
//!
//! We use this terminology since in the context of a web app
//! we usually have to return something to the user and in
//! general client-errors imply a 4XX error code and server-errors
//! imply a 5XX error code.

//! To get a better image of things, let's take a look at
//! [reset_password](crate::database::reset_password). This function
//! returns a `Result<Result<(), ResetError>, Report>`, where
//! [ResetError](crate::database::ResetError) is an Client-error
//! and thus an enum. The `Report` functions as a catch-all for
//! anything uncontrollable going wrong, in this case this means
//! DB Calls failing or the password hashing fails (even though
//! it should never). This is distinctly not the fault of the
//! client who tried to reset their password, but it's on us!
//! So the only reasonable thing to do is returning a 500 and logging
//! the error for developers or sysadmins, there is no need for our
//! code to differentiate between the various reasons a DB call can fail:
//! We can't fix them anyway (yet?).
//!
//! But if everything DB & password hashing wise succeeds there still
//! might be the case that the reset token is simply invalid: This
//! is the fault of the client and something with a clear fix on their
//! side, thus the handler can match on this enum and return the appropriate
//! 4XX status codes.

use axum::{http::StatusCode, response::IntoResponse, Json};
use color_eyre::Report;
use serde::Serialize;
use tracing::error;

impl From<Report> for ApiError {
    fn from(value: Report) -> Self {
        ApiError::UnknownError(value)
    }
}

/// The type for all possible Errors that can be returned by a handler.
pub(crate) enum ApiError {
    /// A request contains an email or user-id, that does not map to an user
    UserNotFound,
    /// A session id was specified that does'nt map to a session
    InvalidSession,
    /// Session was specified using a invalid syntax, details
    /// are in the inner Report.
    MisformedAuth(Report),
    /// A specified token (i.e. for password reset) was not found
    TokenNotFound,
    /// The wrong password was submitted
    WrongCredentials,
    /// A route required authentication, but none was provided
    NotLoggedIn,
    /// Something unexpected happened (i.e. database connection failed)
    UnknownError(Report),
}

/// Every [ApiError] except the `UnknownError` returns the JSON
/// serialized version of this struct.
#[derive(Serialize)]
struct ErrorReturn {
    /// A max. 2 sentence developer understandable reason for the error
    ///
    /// A good error message should always contain:
    ///  - Why did the error occur: As abstract as possible, as concrete as nescessary
    ///  - What can I do: A clear instruction how to prevent this error
    /// The concrete reasons are *not* considered part of the stable API
    /// and can always change, even within a patch release. At the moment every
    /// error is differentiable by the StatusCode (this is part of the stable API)
    ///
    /// Does not need to be understandable by users (so can use technical terms)
    /// but should not require knowledge about implementation details, that is
    /// (at the moment) the job of the frontend.
    reason: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, reason) = match self {
            ApiError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_owned()),
            ApiError::TokenNotFound => (StatusCode::NOT_FOUND, "Token not found".to_owned()),
            ApiError::WrongCredentials => {
                (StatusCode::UNAUTHORIZED, "Wrong Credentials".to_owned())
            }
            ApiError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                "Invalid/expired Session".to_owned(),
            ),
            ApiError::NotLoggedIn => (
                StatusCode::FORBIDDEN,
                "You have to be logged in to access this part of the api".to_owned(),
            ),
            ApiError::MisformedAuth(error) => (
                StatusCode::BAD_REQUEST,
                format!("Session id in 'id' cookie wasn't valid, getting error: {error}"),
            ),
            ApiError::UnknownError(r) => {
                let error = format!("{r:?}");

                error!("Error: {error}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal Server Error: {error}"),
                )
                    .into_response();
            }
        };

        (status, Json(ErrorReturn { reason })).into_response()
    }
}
