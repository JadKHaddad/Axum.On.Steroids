use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    error::{ApiError, MethodNotAllowedError},
    state::ApiState,
    traits::ErrorVerbosityProvider,
};

/// Middleware to map axum's `MethodNotAllowed` rejection to our [`ApiError`].
pub async fn method_not_allowed(
    State(state): State<ApiState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let resp = next.run(req).await;
    let status = resp.status();

    match status {
        StatusCode::METHOD_NOT_ALLOWED => Err(MethodNotAllowedError {
            verbosity: state.error_verbosity(),
        }
        .into()),
        _ => Ok(resp),
    }
}
