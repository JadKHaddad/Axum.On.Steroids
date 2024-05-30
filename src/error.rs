use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use derive_more::From;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy)]
pub enum ErrorVerbosity {
    /// Server returns an empty response with [`StatusCode::NO_CONTENT`] for all errors.
    None,
    /// Server returns only the message with the appropriate status code.
    Message,
    /// Server returns the message, the error type with cleared error content and the appropriate status code.
    Type,
    /// Server returns the message, the error type with the error content and the appropriate status code.
    Full,
}

pub trait ErrorVerbosityProvider {
    fn error_verbosity(&self) -> ErrorVerbosity;
}

#[derive(Debug, Serialize, ToSchema)]
struct ApiErrorResponse {
    error: ApiError,
    message: String,
}

#[derive(Debug, Serialize)]
struct ApiErrorMessage {
    message: String,
}

impl From<ApiErrorResponse> for ApiErrorMessage {
    fn from(response: ApiErrorResponse) -> Self {
        ApiErrorMessage {
            message: response.message,
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(mut self) -> Response {
        match self.error.verbosity() {
            ErrorVerbosity::None => StatusCode::NO_CONTENT.into_response(),
            ErrorVerbosity::Message => {
                let status_code = self.error.status_code();

                (status_code, Json(ApiErrorMessage::from(self))).into_response()
            }
            ErrorVerbosity::Type => {
                self.error.clear();
                let status_code = self.error.status_code();

                (status_code, Json(self)).into_response()
            }
            ErrorVerbosity::Full => {
                let status_code = self.error.status_code();

                (status_code, Json(self)).into_response()
            }
        }
    }
}

#[derive(Debug, From, Serialize, ToSchema)]
#[serde(tag = "type", content = "error")]
/// API error
pub enum ApiError {
    /// Internal server error
    ///
    /// This error is returned when an internal server error occurs.
    InternalServerError(InternalServerError),
    /// Query error
    ///
    /// This error is returned when the query parameters are not as expected.
    Query(QueryError),
    /// Body error
    ///
    /// This error is returned when the body is not as expected.
    Body(BodyError),
    /// Path error
    ///
    /// This error is returned when the path is not as expected.
    Path(PathError),
}

impl ApiError {
    fn verbosity(&self) -> ErrorVerbosity {
        match self {
            ApiError::InternalServerError(err) => err.verbosity,
            ApiError::Query(err) => err.verbosity,
            ApiError::Body(err) => err.verbosity,
            ApiError::Path(err) => err.verbosity,
        }
    }

    fn message(&self) -> String {
        match self {
            ApiError::InternalServerError(_) => {
                String::from("An internal server error has occurred")
            }
            ApiError::Query(_) => String::from("Failed to parse query parameters"),
            ApiError::Body(_) => String::from("Failed to parse body"),
            ApiError::Path(_) => String::from("Failed to parse path parameters"),
        }
    }

    fn clear(&mut self) {
        match self {
            ApiError::InternalServerError(err) => err.clear(),
            ApiError::Query(err) => err.clear(),
            ApiError::Body(err) => err.clear(),
            ApiError::Path(err) => err.clear(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::InternalServerError(err) => err.status_code(),
            ApiError::Query(err) => err.status_code(),
            ApiError::Body(err) => err.status_code(),
            ApiError::Path(err) => err.status_code(),
        }
    }
}

impl From<ApiError> for ApiErrorResponse {
    fn from(error: ApiError) -> Self {
        let message = match error.verbosity() {
            ErrorVerbosity::None => String::new(),
            _ => error.message(),
        };

        ApiErrorResponse { error, message }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        ApiErrorResponse::from(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct InternalServerError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) internal_server_error: String,
}

impl InternalServerError {
    pub fn from_generic_error<E: Into<anyhow::Error>>(verbosity: ErrorVerbosity, err: E) -> Self {
        let err: anyhow::Error = err.into();
        let err = format!("{err:#}");
        tracing::error!(%err, "Internal server error");

        InternalServerError {
            verbosity,
            internal_server_error: err,
        }
    }

    fn clear(&mut self) {
        self.internal_server_error.clear();
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug, Serialize)]
pub struct QueryError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) query_error_reason: String,
    pub(crate) query_expected_schema: String,
}

impl QueryError {
    fn clear(&mut self) {
        self.query_error_reason.clear();
        self.query_expected_schema.clear();
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Serialize)]
pub struct BodyError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) body_error_reason: String,
    pub(crate) body_expected_schema: String,
}

impl BodyError {
    fn clear(&mut self) {
        self.body_error_reason.clear();
        self.body_expected_schema.clear();
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Serialize)]
pub struct PathError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) path_error_reason: String,
}

impl PathError {
    fn clear(&mut self) {
        self.path_error_reason.clear();
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
