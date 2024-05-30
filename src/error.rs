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
    /// Server returns the error type as a JSON response with the appropriate status code.
    Type,
    // TODO: Add more verbosity levels
}

#[derive(Debug, From, Serialize, ToSchema)]
#[serde(tag = "error_type", content = "error")]
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
    pub fn verbosity(&self) -> &ErrorVerbosity {
        match self {
            ApiError::InternalServerError(err) => &err.verbosity,
            ApiError::Query(err) => &err.verbosity,
            ApiError::Body(err) => &err.verbosity,
            ApiError::Path(err) => &err.verbosity,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self.verbosity() {
            _ => {
                // TODO: do something with verbosity
            }
        }
        match self {
            ApiError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
            }
            ApiError::Query(_) => (StatusCode::BAD_REQUEST, Json(self)).into_response(),
            ApiError::Body(_) => (StatusCode::BAD_REQUEST, Json(self)).into_response(),
            ApiError::Path(_) => (StatusCode::BAD_REQUEST, Json(self)).into_response(),
        }
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
}

#[derive(Debug, Serialize)]
pub struct QueryError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) query_error_reason: String,
    pub(crate) query_expected_schema: String,
}

#[derive(Debug, Serialize)]
pub struct BodyError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) body_error_reason: String,
    pub(crate) body_expected_schema: String,
}

#[derive(Debug, Serialize)]
pub struct PathError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) path_error_reason: String,
}
