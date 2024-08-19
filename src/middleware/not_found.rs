use axum::extract::State;

use crate::{
    error::{ApiError, NotFoundError},
    state::StateProvider,
};

pub async fn not_found<S: StateProvider>(State(state): State<S>) -> ApiError {
    ApiError::NotFound(NotFoundError::new(state.error_verbosity()))
}
