use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::{claims::Claims, extractor::jwt::ApiJwt};

#[derive(Debug, Serialize)]
pub struct ExtractClaimsResponse {
    claims: Claims,
}

impl IntoResponse for ExtractClaimsResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Extracts the JWT claims from the request using the [`ApiJwt`] extractor.
///
/// The JWT claims are validated by [`ApiJwt`].
/// This function will reject if [`ApiJwt`] rejects.
pub async fn extract_valid_jwt_claims_using_extractor(
    ApiJwt(claims): ApiJwt<Claims>,
) -> ExtractClaimsResponse {
    ExtractClaimsResponse { claims }
}
