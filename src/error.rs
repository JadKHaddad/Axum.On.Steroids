use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use derive_more::From;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// TODO: use ErrorTypes for QueryError, BodyError and PathError

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum ErrorVerbosity {
    /// Server returns an empty response with [`StatusCode::NO_CONTENT`] for all errors.
    None,
    /// Server returns only the appropriate status code.
    StatusCode,
    /// Server returns only the message with the appropriate status code.
    Message,
    /// Server returns the message, the error type with cleared error content and the appropriate status code.
    Type,
    /// Server returns the message, the error type with the error content and the appropriate status code.
    Full,
}

#[derive(Debug, Serialize, ToSchema)]
struct ApiErrorResponse {
    error: ApiError,
    message: String,
}

#[derive(Debug, Serialize)]
struct ApiErrorMessage {
    message: String,
}

impl From<ApiErrorResponse> for ApiErrorMessage {
    fn from(response: ApiErrorResponse) -> Self {
        ApiErrorMessage {
            message: response.message,
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(mut self) -> Response {
        let headers = self.error.headers();

        match self.error.verbosity() {
            ErrorVerbosity::None => StatusCode::NO_CONTENT.into_response(),
            ErrorVerbosity::StatusCode => {
                let status_code = self.error.status_code();

                (status_code, headers).into_response()
            }
            ErrorVerbosity::Message => {
                let status_code = self.error.status_code();

                (status_code, headers, Json(ApiErrorMessage::from(self))).into_response()
            }
            ErrorVerbosity::Type => {
                self.error.clear();
                let status_code = self.error.status_code();

                (status_code, headers, Json(self)).into_response()
            }
            ErrorVerbosity::Full => {
                let status_code = self.error.status_code();

                (status_code, headers, Json(self)).into_response()
            }
        }
    }
}

#[derive(Debug, From, Serialize, ToSchema)]
#[serde(tag = "type", content = "error")]
/// API error
pub enum ApiError {
    /// Internal server error
    ///
    /// This error is returned when an internal server error occurs.
    InternalServerError(InternalServerError),
    /// Query error
    ///
    /// This error is returned when the query parameters are not as expected.
    Query(QueryError),
    /// Body error
    ///
    /// This error is returned when the body is not as expected.
    Body(BodyError),
    /// Path error
    ///
    /// This error is returned when the path is not as expected.
    Path(PathError),
    /// Method not allowed
    ///
    /// This error is returned when the method is not allowed.
    MethodNotAllowed(MethodNotAllowedError),
    /// API key error
    ///
    /// This error is returned when the API key is not as expected.
    ApiKey(ApiKeyError),
    /// Basic auth error
    ///
    /// This error is returned when the basic auth is not as expected.
    BasicAuth(BasicAuthError),
}

impl ApiError {
    fn verbosity(&self) -> ErrorVerbosity {
        match self {
            ApiError::InternalServerError(err) => err.verbosity,
            ApiError::Query(err) => err.verbosity,
            ApiError::Body(err) => err.verbosity,
            ApiError::Path(err) => err.verbosity,
            ApiError::MethodNotAllowed(err) => err.verbosity,
            ApiError::ApiKey(err) => err.verbosity,
            ApiError::BasicAuth(err) => err.verbosity,
        }
    }

    fn message(&self) -> String {
        match self {
            ApiError::InternalServerError(_) => {
                String::from("An internal server error has occurred")
            }
            ApiError::Query(_) => String::from("Failed to parse query parameters"),
            ApiError::Body(_) => String::from("Failed to parse body"),
            ApiError::Path(_) => String::from("Failed to parse path parameters"),
            ApiError::MethodNotAllowed(_) => String::from("Method not allowed"),
            ApiError::ApiKey(err) => err.message(),
            ApiError::BasicAuth(_) => String::from("Failed to perform basic auth"),
        }
    }

    fn clear(&mut self) {
        match self {
            ApiError::InternalServerError(err) => err.clear(),
            ApiError::Query(err) => err.clear(),
            ApiError::Body(err) => err.clear(),
            ApiError::Path(err) => err.clear(),
            ApiError::MethodNotAllowed(_) => {}
            ApiError::ApiKey(err) => err.clear(),
            ApiError::BasicAuth(err) => err.clear(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::InternalServerError(err) => err.status_code(),
            ApiError::Query(err) => err.status_code(),
            ApiError::Body(err) => err.status_code(),
            ApiError::Path(err) => err.status_code(),
            ApiError::MethodNotAllowed(err) => err.status_code(),
            ApiError::ApiKey(err) => err.status_code(),
            ApiError::BasicAuth(err) => err.status_code(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        match self {
            ApiError::BasicAuth(_) => {
                headers.insert("WWW-Authenticate", HeaderValue::from_static("Basic"));
            }
            _ => {}
        }
        headers
    }
}

impl From<ApiError> for ApiErrorResponse {
    fn from(error: ApiError) -> Self {
        let message = match error.verbosity() {
            ErrorVerbosity::None => String::new(),
            _ => error.message(),
        };

        ApiErrorResponse { error, message }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        ApiErrorResponse::from(self).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct InternalServerError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) internal_server_error: String,
}

impl InternalServerError {
    pub fn from_generic_error<E: Into<anyhow::Error>>(verbosity: ErrorVerbosity, err: E) -> Self {
        let err: anyhow::Error = err.into();
        let err = format!("{err:#}");
        tracing::error!(%err, "Internal server error");

        InternalServerError {
            verbosity,
            internal_server_error: err,
        }
    }

    fn clear(&mut self) {
        self.internal_server_error.clear();
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug, Serialize)]
pub struct QueryError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) query_error_reason: String,
    pub(crate) query_expected_schema: String,
}

impl QueryError {
    fn clear(&mut self) {
        self.query_error_reason.clear();
        self.query_expected_schema.clear();
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Serialize)]
pub struct BodyError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) body_error_reason: String,
    pub(crate) body_expected_schema: String,
}

impl BodyError {
    fn clear(&mut self) {
        self.body_error_reason.clear();
        self.body_expected_schema.clear();
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Serialize)]
pub struct PathError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) path_error_reason: String,
}

impl PathError {
    fn clear(&mut self) {
        self.path_error_reason.clear();
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Serialize)]
pub struct MethodNotAllowedError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
}

impl MethodNotAllowedError {
    fn status_code(&self) -> StatusCode {
        StatusCode::METHOD_NOT_ALLOWED
    }
}

#[derive(Debug, Serialize)]
pub enum ApiKeyErrorType {
    Missing,
    InvalidFromat,
    Invalid,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    api_key_error_type: ApiKeyErrorType,
    api_key_error_reason: String,
}

impl ApiKeyError {
    pub fn new(verbosity: ErrorVerbosity, api_key_error_type: ApiKeyErrorType) -> Self {
        let api_key_error_reason = Self::reason(&api_key_error_type);

        ApiKeyError {
            verbosity,
            api_key_error_type,
            api_key_error_reason,
        }
    }

    fn reason(api_key_error_type: &ApiKeyErrorType) -> String {
        match api_key_error_type {
            ApiKeyErrorType::Missing => String::from("API key not found"),
            ApiKeyErrorType::InvalidFromat => {
                String::from("API key header value is not valid ASCII string")
            }
            ApiKeyErrorType::Invalid => String::from("Invalid API key"),
        }
    }

    fn message(&self) -> String {
        match self.verbosity {
            ErrorVerbosity::None => String::new(),
            _ => self.api_key_error_reason.clone(),
        }
    }

    fn clear(&mut self) {
        self.api_key_error_reason.clear();
    }

    fn status_code(&self) -> StatusCode {
        match self.api_key_error_type {
            ApiKeyErrorType::Missing => StatusCode::UNAUTHORIZED,
            ApiKeyErrorType::InvalidFromat => StatusCode::UNAUTHORIZED,
            ApiKeyErrorType::Invalid => StatusCode::FORBIDDEN,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BasicAuthError {
    #[serde(skip)]
    pub(crate) verbosity: ErrorVerbosity,
    pub(crate) basic_auth_error_reason: String,
}

impl BasicAuthError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn clear(&mut self) {
        self.basic_auth_error_reason.clear();
    }
}
