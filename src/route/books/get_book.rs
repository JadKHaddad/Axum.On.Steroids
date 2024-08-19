use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::StateProvider;

use crate::{
    error::{ResourceError, ResourceErrorProvider},
    extractor::query::ApiQuery,
    state::ApiState,
};

use super::Book;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBookQuery {
    pub id: i64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetBookResponse {
    pub book: Book,
}

impl IntoResponse for GetBookResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "error_type")]
pub enum GetBookErrorType {
    NotFound {
        #[serde(skip)]
        id: i64,
    },
    IdTooBig {
        #[serde(skip)]
        id: i64,
    },
}

#[derive(Debug, Serialize)]
pub struct GetBookErrorContext {
    pub reason: String,
}

impl ResourceErrorProvider for GetBookErrorType {
    type Context = GetBookErrorContext;

    fn headers(&self) -> Option<axum::http::HeaderMap> {
        None
    }

    fn status_code(&self) -> StatusCode {
        match self {
            GetBookErrorType::NotFound { .. } => StatusCode::NOT_FOUND,
            GetBookErrorType::IdTooBig { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            GetBookErrorType::NotFound { .. } => "Book not found",
            GetBookErrorType::IdTooBig { .. } => "Id too big",
        }
    }

    fn context(&self) -> Self::Context {
        match self {
            GetBookErrorType::NotFound { id } => GetBookErrorContext {
                reason: format!("Book with id {} not found", id),
            },
            GetBookErrorType::IdTooBig { id } => GetBookErrorContext {
                reason: format!("Id {} is too big", id),
            },
        }
    }
}

pub async fn get_book(
    ApiQuery(query): ApiQuery<GetBookQuery>,
    State(_state): State<ApiState>,
) -> Result<GetBookResponse, ResourceError<GetBookErrorType, GetBookErrorContext>> {
    let id = query.id;

    Ok(GetBookResponse {
        book: Book {
            title: "The Catcher in the Rye".to_string(),
            author: "J.D. Salinger".to_string(),
            isbn: "978-0-316-76948-0".to_string(),
            year: 1951,
            id,
        },
    })
}

pub async fn get_book_not_found(
    ApiQuery(query): ApiQuery<GetBookQuery>,
    State(state): State<ApiState>,
) -> Result<GetBookResponse, ResourceError<GetBookErrorType, GetBookErrorContext>> {
    let id = query.id;

    Err(ResourceError::new(
        state.error_verbosity(),
        GetBookErrorType::NotFound { id },
    ))
}

pub async fn get_book_id_too_big(
    ApiQuery(query): ApiQuery<GetBookQuery>,
    State(state): State<ApiState>,
) -> Result<GetBookResponse, ResourceError<GetBookErrorType, GetBookErrorContext>> {
    let id = query.id;

    Err(ResourceError::new(
        state.error_verbosity(),
        GetBookErrorType::IdTooBig { id },
    ))
}
