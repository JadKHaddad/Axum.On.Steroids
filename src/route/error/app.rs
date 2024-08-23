use crate::{
    error::{ApiError, ErrorVerbosityProvider},
    server_error,
    state::ApiState,
};
use axum::{extract::State, routing::get, Router};

pub fn app() -> Router<ApiState> {
    Router::<ApiState>::new()
        .route("/internal_server_error", get(internal_server_error))
        .route("/default_api_error", get(default_api_error))
}

pub async fn internal_server_error(State(state): State<ApiState>) -> Result<(), ApiError> {
    tokio::fs::read_to_string("non_existent_file.txt")
        .await
        .map(|_| ())
        .map_err(server_error!(state))
}

pub async fn default_api_error() -> ApiError {
    ApiError::default()
}
