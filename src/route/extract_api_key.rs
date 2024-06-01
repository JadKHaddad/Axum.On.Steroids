use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::extractor::valid_api_key::ValidApiKey;

#[derive(Debug, Serialize)]
pub struct ExtractApiKeyResponse {
    used_api_key: String,
}

impl IntoResponse for ExtractApiKeyResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the API key from the request using the [`ApiKey`] extractor.
///
/// This function will reject if [`ApiKey`] rejects.
pub async fn extract_api_key_using_extractor(
    ValidApiKey(key): ValidApiKey,
) -> ExtractApiKeyResponse {
    ExtractApiKeyResponse {
        used_api_key: key.used_api_key,
    }
}
