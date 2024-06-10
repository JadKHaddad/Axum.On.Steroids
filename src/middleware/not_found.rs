use axum::extract::State;

use crate::{
    error::{ApiError, NotFoundError},
    state::{ApiState, StateProvider},
};

pub async fn not_found(State(state): State<ApiState>) -> ApiError {
    ApiError::NotFound(NotFoundError::new(state.error_verbosity()))
}
