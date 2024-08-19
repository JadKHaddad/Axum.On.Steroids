use axum::{middleware::from_fn_with_state, routing::get, Router};

use crate::{middleware::validate_api_key_and_put_as_extension, state::ApiState};

pub fn app(state: ApiState) -> Router<ApiState> {
    Router::<ApiState>::new()
        .route("/", get(|| async { "API Key Protected" }))
        .route(
            "/do_not_use_extension",
            get(super::do_not_use_extension::do_not_use_extension),
        )
        .route(
            "/valid_api_key_from_extension",
            get(super::valid_api_key_from_extension::valid_api_key_from_extension),
        )
        .layer(from_fn_with_state(
            state,
            validate_api_key_and_put_as_extension::validate_api_key_and_put_as_extension,
        ))
}
