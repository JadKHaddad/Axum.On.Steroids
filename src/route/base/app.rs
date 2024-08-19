use axum::{routing::get, Router};

use crate::state::ApiState;

pub fn app() -> Router<ApiState> {
    Router::<ApiState>::new()
        .route("/", get(|| async { "Index" }))
        .route(
            "/extract_valid_jwt_claims_using_extractor",
            get(super::extract_jwt_claims::extract_valid_jwt_claims_using_extractor),
        )
        .route(
            "/extract_bearer_token_using_extractor",
            get(super::extract_bearer_token::extract_bearer_token_using_extractor),
        )
        .route(
            "/extract_authenticated_basic_auth_using_extractor",
            get(super::extract_authenticated_basic_auth::extract_authenticated_basic_auth_using_extractor),
        )
        .route(
            "/extract_basic_auth_using_extractor",
            get(super::extract_basic_auth::extract_basic_auth_using_extractor),
        )
        .route(
            "/extract_api_key_using_extractor",
            get(super::extract_api_key::extract_api_key_using_extractor),
        )
        .route(
            "/extract_valid_api_key_using_optional_extractor",
            get(super::extract_valid_api_key_optional::extract_valid_api_key_using_optional_extractor),
        )
        .route(
            "/extract_valid_api_key_using_extractor",
            get(super::extract_valid_api_key::extract_valid_api_key_using_extractor),
        )
}
