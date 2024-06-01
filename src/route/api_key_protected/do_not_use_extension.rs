use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DoNotUseExtensionResponse {
    message: String,
}

impl IntoResponse for DoNotUseExtensionResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub async fn do_not_use_extension() -> DoNotUseExtensionResponse {
    DoNotUseExtensionResponse {
        message: String::from("I was protected by an API key! but I have no use for it. I don't even know wich API key was used."),
    }
}
