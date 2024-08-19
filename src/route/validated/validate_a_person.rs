use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::extractor::{json::ApiJson, validated::Validated};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Person {
    #[validate(length(min = 5, message = "Must be at least 5 characters long"))]
    pub name: String,
    #[validate(range(min = 25, max = 150, message = "Must be between 25 and 150"))]
    pub age: u8,
    pub is_alive: bool,
    pub city: String,
}

impl IntoResponse for Person {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub async fn validate_a_person(Validated(ApiJson(person)): Validated<ApiJson<Person>>) -> Person {
    person
}
