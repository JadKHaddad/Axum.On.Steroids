use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::{extractor::bearer_token::ApiBearerToken, types::used_bearer_token::UsedBearerToken};

#[derive(Debug, Serialize)]
pub struct ExtractBearerTokenResponse {
    used_token: String,
}

impl IntoResponse for ExtractBearerTokenResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the bearer token from the request using the [`ApiBearerToken`] extractor.
///
/// The bearer token is not validated by [`ApiBearerToken`].
/// This function will reject if [`ApiBearerToken`] rejects.
pub async fn extract_bearer_token_using_extractor(
    ApiBearerToken(UsedBearerToken{value: used_token}): ApiBearerToken,
) -> ExtractBearerTokenResponse {
    ExtractBearerTokenResponse {
        used_token
    }
}
