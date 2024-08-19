use axum::{routing::post, Router};

use crate::state::ApiState;

pub fn app() -> Router<ApiState> {
    Router::<ApiState>::new().route(
        "/validate_a_person",
        post(super::validate_a_person::validate_a_person),
    )
}
