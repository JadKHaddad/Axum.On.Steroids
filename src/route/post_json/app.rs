use axum::{routing::post, Router};

use crate::state::ApiState;

pub fn app() -> Router<ApiState> {
    Router::<ApiState>::new().route("/echo_a_person", post(super::echo_a_person::echo_a_person))
}
