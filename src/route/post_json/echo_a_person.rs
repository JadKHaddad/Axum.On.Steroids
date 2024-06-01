use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::extractor::json::ApiJson;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Person {
    pub name: String,
    pub age: u8,
    pub is_alive: bool,
    pub city: String,
}

impl IntoResponse for Person {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub async fn echo_a_person(ApiJson(person): ApiJson<Person>) -> Person {
    person
}
