use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::extractor::{json::ApiJson, validated::ValidatedFromRequest};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Person {
    #[validate(length(min = 5, message = "Must be at least 5 characters long"))]
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

pub async fn validate_a_person(
    ValidatedFromRequest(ApiJson(person)): ValidatedFromRequest<ApiJson<Person>>,
) -> Person {
    person
}
