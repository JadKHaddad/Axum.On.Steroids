use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::extractor::basic_auth::ApiBasicAuth;

#[derive(Debug, Serialize)]
pub struct ExtractBasicAuthResponse {
    used_username: String,
    used_password: Option<String>,
}

impl IntoResponse for ExtractBasicAuthResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the basic auth from the request using the [`ApiBasicAuth`] extractor.
///
/// The basic auth is not validated by [`ApiBasicAuth`].
/// This function will reject if [`ApiBasicAuth`] rejects.
pub async fn extract_basic_auth_using_extractor(
    ApiBasicAuth(basic_auth): ApiBasicAuth,
) -> ExtractBasicAuthResponse {
    ExtractBasicAuthResponse {
        used_username: basic_auth.username,
        used_password: basic_auth.password,
    }
}
