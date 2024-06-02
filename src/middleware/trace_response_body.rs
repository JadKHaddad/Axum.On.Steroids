use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;

use crate::{
    error::{ApiError, InternalServerError},
    state::ApiState,
    traits::StateProvider,
};

/// Middlware to trace the response body.
///
/// This is a very expensive middleware, since it reads the entire response body and logs it.
pub async fn trace_response_body(
    State(state): State<ApiState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = body
        .collect()
        .await
        .map_err(|err| InternalServerError::from_generic_error(state.error_verbosity(), err))?
        .to_bytes();

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::trace!(%body, "Response body");
    }

    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}
