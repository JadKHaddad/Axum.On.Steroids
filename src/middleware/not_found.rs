use axum::extract::State;

use crate::error::{ApiError, ErrorVerbosityProvider, NotFoundError};

pub async fn not_found<S: ErrorVerbosityProvider>(State(state): State<S>) -> ApiError {
    ApiError::NotFound(NotFoundError::new(state.error_verbosity()))
}
