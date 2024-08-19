use axum::{extract::State, routing::get, Router};

use crate::{
    error::ApiError,
    state::{ApiState, StateProvider},
};

pub fn app() -> Router<ApiState> {
    Router::<ApiState>::new().route("/internal_server_error", get(internal_server_error))
}

pub async fn internal_server_error(State(state): State<ApiState>) -> Result<(), ApiError> {
    tokio::fs::read_to_string("non_existent_file.txt")
        .await
        .map(|_| ())
        .map_err(|err| ApiError::from_generic_error(state.error_verbosity(), err))
}
