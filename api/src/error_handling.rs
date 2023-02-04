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
                format!("The Auth header did not follow 'Bearer [session_uuid]', getting error: {error}")
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
