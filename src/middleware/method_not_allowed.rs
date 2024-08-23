use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};

use crate::error::{ApiError, ErrorVerbosityProvider, MethodNotAllowedError};

/// Middleware to map axum's `MethodNotAllowed` rejection to our [`ApiError`].
pub async fn method_not_allowed<S: ErrorVerbosityProvider>(
    State(state): State<S>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let resp = next.run(req).await;
    let status = resp.status();

    match status {
        StatusCode::METHOD_NOT_ALLOWED => {
            Err(MethodNotAllowedError::new(state.error_verbosity()).into())
        }
        _ => Ok(resp),
    }
}
