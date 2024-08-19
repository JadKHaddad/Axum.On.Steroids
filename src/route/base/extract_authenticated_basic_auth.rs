use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::extractor::authenticated_basic_auth::ApiAuthenticatedBasicAuth;

#[derive(Debug, Serialize)]
pub struct ExtractAuthenticatedBasicAuthResponse {
    used_username: String,
    used_password: Option<String>,
}

impl IntoResponse for ExtractAuthenticatedBasicAuthResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the basic auth from the request using the [`ApiAuthenticatedBasicAuth`] extractor.
pub async fn extract_authenticated_basic_auth_using_extractor(
    ApiAuthenticatedBasicAuth(basic_auth): ApiAuthenticatedBasicAuth,
) -> ExtractAuthenticatedBasicAuthResponse {
    ExtractAuthenticatedBasicAuthResponse {
        used_username: basic_auth.username,
        used_password: basic_auth.password,
    }
}
