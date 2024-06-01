use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Serialize;

use crate::types::used_api_key::UsedApiKey;

#[derive(Debug, Serialize)]
pub struct ApiKeyFromExtensionResponse {
    used_api_key: String,
}

impl IntoResponse for ApiKeyFromExtensionResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the API key that was provided from [`validate_api_key_and_put_as_extension`](crate::middleware::validate_api_key_and_put_as_extension::validate_api_key_and_put_as_extension) middleware.
pub async fn api_key_from_extension(
    used_api_key: Extension<UsedApiKey>,
) -> ApiKeyFromExtensionResponse {
    ApiKeyFromExtensionResponse {
        used_api_key: used_api_key.used_api_key.clone(),
    }
}
