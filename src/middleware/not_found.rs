use axum::extract::State;

use crate::{
    error::{ApiError, NotFoundError},
    state::ApiState,
    traits::StateProvider,
};

pub async fn not_found(State(state): State<ApiState>) -> ApiError {
    let error_verbosity = state.error_verbosity();

    ApiError::NotFound(NotFoundError {
        verbosity: error_verbosity,
    })
}
