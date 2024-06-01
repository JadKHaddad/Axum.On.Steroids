use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Serialize;

use crate::extractor::valid_api_key::ValidApiKey;

#[derive(Debug, Serialize)]
pub struct ApiKeyFromExtensionResponse {
    used_api_key: String,
}

impl IntoResponse for ApiKeyFromExtensionResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the API key from the [`Extension`] that was provided from [`validate_api_key_and_put_as_extension`](crate::middleware::validate_api_key_and_put_as_extension::validate_api_key_and_put_as_extension) middleware.
pub async fn valid_api_key_from_extension(
    Extension(valid_api_key): Extension<ValidApiKey>,
) -> ApiKeyFromExtensionResponse {
    ApiKeyFromExtensionResponse {
        used_api_key: valid_api_key.0.used_api_key,
    }
}
