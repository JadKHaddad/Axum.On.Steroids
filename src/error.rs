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
    message: &'static str,
}

#[derive(Debug, Serialize)]
struct ApiErrorMessage {
    message: &'static str,
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
    /// Not found error
    ///
    /// This error is returned when the requested resource is not found.
    NotFound(NotFoundError),
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
            ApiError::NotFound(err) => err.verbosity,
            ApiError::ApiKey(err) => err.verbosity,
            ApiError::BasicAuth(err) => err.verbosity,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            ApiError::InternalServerError(_) => "An internal server error has occurred",
            ApiError::Query(_) => "Failed to parse query parameters",
            ApiError::Body(_) => "Failed to parse body",
            ApiError::Path(_) => "Failed to parse path parameters",
            ApiError::MethodNotAllowed(_) => "Method not allowed",
            ApiError::NotFound(_) => "The requested resource was not found",
            ApiError::ApiKey(_) => "API key error",
            ApiError::BasicAuth(_) => "Basic auth error",
        }
    }

    fn clear(&mut self) {
        match self {
            ApiError::InternalServerError(err) => err.clear(),
            ApiError::Query(err) => err.clear(),
            ApiError::Body(err) => err.clear(),
            ApiError::Path(err) => err.clear(),
            ApiError::MethodNotAllowed(_) => {}
            ApiError::NotFound(_) => {}
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
            ApiError::NotFound(err) => err.status_code(),
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
            ErrorVerbosity::None => "",
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
    verbosity: ErrorVerbosity,
    internal_server_error: String,
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
    verbosity: ErrorVerbosity,
    query_error_reason: String,
    query_expected_schema: String,
}

impl QueryError {
    pub fn new(
        verbosity: ErrorVerbosity,
        query_error_reason: String,
        query_expected_schema: String,
    ) -> Self {
        QueryError {
            verbosity,
            query_error_reason,
            query_expected_schema,
        }
    }

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
    verbosity: ErrorVerbosity,
    body_error_reason: String,
    body_expected_schema: String,
}

impl BodyError {
    pub fn new(
        verbosity: ErrorVerbosity,
        body_error_reason: String,
        body_expected_schema: String,
    ) -> Self {
        BodyError {
            verbosity,
            body_error_reason,
            body_expected_schema,
        }
    }

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
    verbosity: ErrorVerbosity,
    path_error_reason: String,
}

impl PathError {
    pub fn new(verbosity: ErrorVerbosity, path_error_reason: String) -> Self {
        PathError {
            verbosity,
            path_error_reason,
        }
    }

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
    verbosity: ErrorVerbosity,
}

impl MethodNotAllowedError {
    pub fn new(verbosity: ErrorVerbosity) -> Self {
        MethodNotAllowedError { verbosity }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::METHOD_NOT_ALLOWED
    }
}

#[derive(Debug, Serialize)]
pub struct NotFoundError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
}

impl NotFoundError {
    pub fn new(verbosity: ErrorVerbosity) -> Self {
        NotFoundError { verbosity }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

#[derive(Debug, Serialize)]
pub enum ApiKeyErrorType {
    Missing,
    InvalidChars,
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
            ApiKeyErrorType::Missing => String::from("API key is missing"),
            ApiKeyErrorType::InvalidChars => String::from("API key contains invalid characters"),
            ApiKeyErrorType::Invalid => String::from("API key invalid"),
        }
    }

    fn clear(&mut self) {
        self.api_key_error_reason.clear();
    }

    fn status_code(&self) -> StatusCode {
        match self.api_key_error_type {
            ApiKeyErrorType::Missing => StatusCode::UNAUTHORIZED,
            ApiKeyErrorType::InvalidChars => StatusCode::UNAUTHORIZED,
            ApiKeyErrorType::Invalid => StatusCode::FORBIDDEN,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum BasicAuthErrorType {
    Missing,
    InvalidChars,
    Decode {
        #[serde(skip)]
        reason: String,
    },
    NotBasic,
    Invalid,
}

#[derive(Debug, Serialize)]
pub struct BasicAuthError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    basic_auth_error_type: BasicAuthErrorType,
    basic_auth_error_reason: String,
}

impl BasicAuthError {
    pub fn new(verbosity: ErrorVerbosity, basic_auth_error_type: BasicAuthErrorType) -> Self {
        let basic_auth_error_reason = Self::reason(&basic_auth_error_type);

        BasicAuthError {
            verbosity,
            basic_auth_error_type,
            basic_auth_error_reason,
        }
    }

    fn reason(basic_auth_error_type: &BasicAuthErrorType) -> String {
        match basic_auth_error_type {
            BasicAuthErrorType::Missing => String::from("`Authorization` header is missing"),
            BasicAuthErrorType::InvalidChars => {
                String::from("`Authorization` header contains invalid characters")
            }
            BasicAuthErrorType::Decode { reason } => {
                format!("`Authorization` header could not be decoded: {reason}")
            }
            BasicAuthErrorType::NotBasic => String::from("`Authorization` header must be `Basic`"),
            BasicAuthErrorType::Invalid => String::from("`Authorization` header is invalid"),
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn clear(&mut self) {
        self.basic_auth_error_reason.clear();
    }
}
