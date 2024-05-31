use axum::{extract::Request, http::Response, middleware::Next, response::IntoResponse};

/// Middlware to trace headers.
pub async fn trace_headers(req: Request, next: Next) -> impl IntoResponse {
    let incoming_headers = req.headers();
    tracing::trace!(?incoming_headers, "Headers");

    let response = next.run(req).await;
    let (parts, body) = response.into_parts();

    let outgoing_headers = &parts.headers;
    tracing::trace!(?outgoing_headers, "Headers");

    Response::from_parts(parts, body)
}
