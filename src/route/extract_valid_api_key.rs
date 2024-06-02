use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::extractor::valid_api_key::ValidApiKey;

#[derive(Debug, Serialize)]
pub struct ExtractValidApiKeyResponse {
    used_valid_api_key: String,
}

impl IntoResponse for ExtractValidApiKeyResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the valid API key from the request using the [`ValidApiKey`] extractor.
///
/// This function will reject if [`ValidApiKey`] rejects.
pub async fn extract_valid_api_key_using_extractor(
    ValidApiKey(key): ValidApiKey,
) -> ExtractValidApiKeyResponse {
    ExtractValidApiKeyResponse {
        used_valid_api_key: key.api_key,
    }
}
