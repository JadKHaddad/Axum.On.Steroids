use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use derive_more::From;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, From, Serialize, ToSchema)]
#[serde(tag = "error_type", content = "error")]
/// API error
pub enum ApiError {
    /// Internal server error
    InternalServerError(InternalServerError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InternalServerError;

/// Convert every error that implements [`core::convert::Into<>`] [`anyhow::Error`] into an [`InternalServerError`].
impl<E> From<E> for InternalServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let err: anyhow::Error = err.into();
        let err = format!("{err:#}");
        tracing::error!(%err, "Internal server error");

        Self
    }
}
