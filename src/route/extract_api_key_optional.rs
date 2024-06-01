use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::extractor::{optional::Optional, valid_api_key::ValidApiKey};

#[derive(Debug, Serialize)]
pub struct OptionalExtractApiKeyResponse {
    used_api_key: Option<String>,
}

impl IntoResponse for OptionalExtractApiKeyResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the API key from the request using the [`Optional`] extractor.
///
/// The API key is optional, so this function will not reject if the API key is not provided.
pub async fn extract_api_key_using_optional_extractor(
    Optional(opt_api_key): Optional<ValidApiKey>,
) -> OptionalExtractApiKeyResponse {
    OptionalExtractApiKeyResponse {
        used_api_key: opt_api_key.map(|key| key.0.used_api_key),
    }
}
