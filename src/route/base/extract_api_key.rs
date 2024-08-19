use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::extractor::api_key::ApiKey;

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
/// The API key is not validated by [`ApiKey`].
/// This function will reject if [`ApiKey`] rejects.
pub async fn extract_api_key_using_extractor(ApiKey(key): ApiKey) -> ExtractApiKeyResponse {
    ExtractApiKeyResponse {
        used_api_key: key.value,
    }
}
