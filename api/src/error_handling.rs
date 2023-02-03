use axum::{http::StatusCode, response::IntoResponse, Json};
use color_eyre::Report;
use serde::Serialize;
use tracing::error;

impl From<Report> for ApiError {
    fn from(value: Report) -> Self {
        ApiError::UnknownError(value)
    }
}

pub(crate) enum ApiError {
    UserNotFound,
    InvalidSession,
    TokenNotFound,
    WrongCredentials,
    UnknownError(Report),
}

#[derive(Serialize)]
struct ErrorReturn {
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
