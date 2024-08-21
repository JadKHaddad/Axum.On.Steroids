use std::{borrow::Cow, string::FromUtf8Error};

use axum::{
    extract::{
        path::ErrorKind as PathErrorKind,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::DecodeError;
use derive_more::From;
use reqwest::header::ToStrError;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::ValidationErrors;

use crate::jwt::JwtValidationError;

// FIXME: Must not be public to all routes, to prevent defining arbitrary error verbosity.
// Create PrivateErrorVerbosity in state.rs. and use it as input here.
// TODO: add a RandomStatus code that returns only a random status code.
#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub enum ErrorVerbosity {
    /// Server returns an empty response with [`StatusCode::NO_CONTENT`] for all errors.
    None,
    /// Server returns only the appropriate status code.
    #[default]
    StatusCode,
    /// Server returns only the message with the appropriate status code.
    Message,
    /// Server returns the message, the error type with cleared error content and the appropriate status code.
    Type,
    /// Server returns the message, the error type with the error content and the appropriate status code.
    Full,
}

impl ErrorVerbosity {
    fn should_generate_error_context(&self) -> bool {
        matches!(self, ErrorVerbosity::Full)
    }
}

#[derive(Debug, Serialize, ToSchema)]
struct ApiErrorResponse {
    #[serde(flatten)]
    error: ApiError,
    message: &'static str,
}

/// Holds only the message of the error.
///
/// Used if the error verbosity is set to [`ErrorVerbosity::Message`].
#[derive(Debug, Serialize)]
struct ErrorMessage {
    message: &'static str,
}

impl From<ApiErrorResponse> for ErrorMessage {
    fn from(response: ApiErrorResponse) -> Self {
        ErrorMessage {
            message: response.message,
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let headers = self.error.headers().unwrap_or_default();

        match self.error.verbosity() {
            ErrorVerbosity::None => StatusCode::NO_CONTENT.into_response(),
            ErrorVerbosity::StatusCode => (self.error.status_code(), headers).into_response(),
            ErrorVerbosity::Message => (
                self.error.status_code(),
                headers,
                Json(ErrorMessage::from(self)),
            )
                .into_response(),
            // error content is (cleared/not cleared) on error creation
            ErrorVerbosity::Type | ErrorVerbosity::Full => {
                (self.error.status_code(), headers, Json(self)).into_response()
            }
        }
    }
}

#[derive(Debug, From, Serialize, ToSchema)]
#[serde(tag = "error_type", content = "error")]
/// API error.
pub enum ApiError {
    /// Internal server error.
    ///
    /// This error is returned when an internal server error occurs.
    InternalServerError(InternalServerError),
    /// Query error.
    ///
    /// This error is returned when the query parameters are not as expected.
    Query(QueryError),
    /// Json Body error.
    ///
    /// This error is returned when the body is not as expected.
    JsonBody(JsonBodyError),
    /// Path error.
    ///
    /// This error is returned when the path is not as expected.
    Path(PathError),
    /// Method not allowed.
    ///
    /// This error is returned when the method is not allowed.
    MethodNotAllowed(MethodNotAllowedError),
    /// Not found error.
    ///
    /// This error is returned when the requested resource is not found.
    NotFound(NotFoundError),
    /// API key error.
    ///
    /// This error is returned when the API key is not as expected.
    ApiKey(ApiKeyError),
    /// Basic auth error.
    ///
    /// This error is returned when the basic auth is not as expected.
    BasicAuth(BasicAuthError),
    /// Bearer extract error.
    ///
    /// This error is returned when the bearer token is not as expected.
    Bearer(BearerError),
    /// JWT error.
    ///
    /// This error is returned when the JWT is not as expected.
    Jwt(JwtError),
    /// Validation error.
    ///
    /// This error is returned when the validation of the extracted data fails.
    Validation(ValidationError),
}

impl Default for ApiError {
    fn default() -> Self {
        Self::InternalServerError(Default::default())
    }
}

impl ApiError {
    fn verbosity(&self) -> ErrorVerbosity {
        match self {
            ApiError::InternalServerError(err) => err.verbosity,
            ApiError::Query(err) => err.verbosity,
            ApiError::JsonBody(err) => err.verbosity,
            ApiError::Path(err) => err.verbosity,
            ApiError::MethodNotAllowed(err) => err.verbosity,
            ApiError::NotFound(err) => err.verbosity,
            ApiError::ApiKey(err) => err.verbosity,
            ApiError::BasicAuth(err) => err.verbosity,
            ApiError::Bearer(err) => err.verbosity,
            ApiError::Jwt(err) => err.verbosity,
            ApiError::Validation(err) => err.verbosity,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            ApiError::InternalServerError(_) => "An internal server error has occurred",
            ApiError::Query(_) => "Failed to parse query parameters",
            ApiError::JsonBody(_) => "Failed to parse request body",
            ApiError::Path(_) => "Failed to parse path parameters",
            ApiError::MethodNotAllowed(_) => "Method not allowed",
            ApiError::NotFound(_) => "The requested resource was not found",
            ApiError::ApiKey(_) => "API key error",
            ApiError::BasicAuth(_) => "Basic auth error",
            ApiError::Bearer(_) => "Bearer auth error",
            ApiError::Jwt(_) => "JWT error",
            ApiError::Validation(_) => "Validation error",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::InternalServerError(err) => err.status_code(),
            ApiError::Query(err) => err.status_code(),
            ApiError::JsonBody(err) => err.status_code(),
            ApiError::Path(err) => err.status_code(),
            ApiError::MethodNotAllowed(err) => err.status_code(),
            ApiError::NotFound(err) => err.status_code(),
            ApiError::ApiKey(err) => err.status_code(),
            ApiError::BasicAuth(err) => err.status_code(),
            ApiError::Bearer(err) => err.status_code(),
            ApiError::Jwt(err) => err.status_code(),
            ApiError::Validation(err) => err.status_code(),
        }
    }

    fn headers(&self) -> Option<HeaderMap> {
        match self {
            ApiError::BasicAuth(_) => {
                let mut headers = HeaderMap::new();
                headers.insert("WWW-Authenticate", HeaderValue::from_static("Basic"));

                Some(headers)
            }
            ApiError::Bearer(_) | ApiError::Jwt(_) => {
                let mut headers = HeaderMap::new();
                headers.insert("WWW-Authenticate", HeaderValue::from_static("Bearer"));

                Some(headers)
            }
            _ => None,
        }
    }

    pub fn from_generic_error<E: Into<anyhow::Error>>(verbosity: ErrorVerbosity, err: E) -> Self {
        InternalServerError::from_generic_error(verbosity, err).into()
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
    error: Option<String>,
}

impl InternalServerError {
    pub fn from_generic_error<E: Into<anyhow::Error>>(verbosity: ErrorVerbosity, err: E) -> Self {
        let err: anyhow::Error = err.into();
        let err = format!("{err:#}");
        tracing::error!(%err, "Internal server error");

        let error = verbosity.should_generate_error_context().then_some(err);

        Self { verbosity, error }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl Default for InternalServerError {
    fn default() -> Self {
        tracing::error!("Internal server error");

        Self {
            verbosity: Default::default(),
            error: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum QueryErrorType {
    /// Query parameters deserialization failed.
    DeserializeError,
}

#[derive(Debug, Serialize)]
pub struct QueryError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    r#type: QueryErrorType,
    reason: Option<String>,
    expected_schema: Option<String>,
}

impl QueryError {
    pub fn from_query_rejection<T: JsonSchema>(
        verbosity: ErrorVerbosity,
        query_rejection: QueryRejection,
    ) -> ApiError {
        let r#type = match query_rejection {
            QueryRejection::FailedToDeserializeQueryString(_) => QueryErrorType::DeserializeError,
            _ => return ApiError::from_generic_error(verbosity, query_rejection),
        };

        let (reason, expected_schema) = match verbosity.should_generate_error_context() {
            true => {
                let reason = query_rejection.body_text();
                let expected_schema = match serde_yaml::to_string(&schema_for!(T)) {
                    Ok(schema) => schema,
                    Err(err) => return ApiError::from_generic_error(verbosity, err),
                };

                (Some(reason), Some(expected_schema))
            }
            false => (None, None),
        };

        QueryError {
            verbosity,
            r#type,
            reason,
            expected_schema,
        }
        .into()
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Serialize)]
pub enum JsonBodyErrorType {
    /// JSON data could not be deserialized to the target type.
    DataError,
    /// JSON syntax error. Invalid JSON.
    SyntaxError,
    /// Missing JSON content type.
    MissingJsonContentType,
}

#[derive(Debug, Serialize)]
pub struct JsonBodyError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    r#type: JsonBodyErrorType,
    reason: Option<String>,
    expected_schema: Option<String>,
}

impl JsonBodyError {
    pub fn from_json_rejection<T: JsonSchema>(
        verbosity: ErrorVerbosity,
        json_rejection: JsonRejection,
    ) -> ApiError {
        let r#type = match json_rejection {
            JsonRejection::JsonDataError(_) => JsonBodyErrorType::DataError,
            JsonRejection::JsonSyntaxError(_) => JsonBodyErrorType::SyntaxError,
            JsonRejection::MissingJsonContentType(_) => JsonBodyErrorType::MissingJsonContentType,
            _ => return ApiError::from_generic_error(verbosity, json_rejection),
        };

        let (reason, expected_schema) = match verbosity.should_generate_error_context() {
            true => {
                let reason = json_rejection.body_text();
                let expected_schema = match serde_yaml::to_string(&schema_for!(T)) {
                    Ok(schema) => schema,
                    Err(err) => return ApiError::from_generic_error(verbosity, err),
                };

                (Some(reason), Some(expected_schema))
            }
            false => (None, None),
        };

        JsonBodyError {
            verbosity,
            r#type,
            reason,
            expected_schema,
        }
        .into()
    }

    fn status_code(&self) -> StatusCode {
        match self.r#type {
            JsonBodyErrorType::DataError => StatusCode::UNPROCESSABLE_ENTITY,
            JsonBodyErrorType::SyntaxError => StatusCode::BAD_REQUEST,
            JsonBodyErrorType::MissingJsonContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum PathErrorType {
    /// Path parameters deserialization failed.
    DeserializeError,
}

#[derive(Debug, Serialize)]
pub struct PathError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    r#type: PathErrorType,
    reason: Option<String>,
}

impl PathError {
    pub fn from_path_rejection(
        verbosity: ErrorVerbosity,
        path_rejection: PathRejection,
    ) -> ApiError {
        let r#type = match path_rejection {
            PathRejection::FailedToDeserializePathParams(ref err) => match err.kind() {
                PathErrorKind::Message(_)
                | PathErrorKind::InvalidUtf8InPathParam { .. }
                | PathErrorKind::ParseError { .. }
                | PathErrorKind::ParseErrorAtIndex { .. }
                | PathErrorKind::ParseErrorAtKey { .. } => PathErrorType::DeserializeError,
                _ => return ApiError::from_generic_error(verbosity, path_rejection),
            },
            _ => return ApiError::from_generic_error(verbosity, path_rejection),
        };

        let reason = verbosity
            .should_generate_error_context()
            .then_some(path_rejection.body_text());

        PathError {
            verbosity,
            r#type,
            reason,
        }
        .into()
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
    /// API key is missing.
    Missing,
    /// API key contains invalid characters.
    InvalidChars {
        #[serde(skip)]
        err: ToStrError,
    },
    /// API key is invalid.
    Invalid,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    r#type: ApiKeyErrorType,
    reason: Option<Cow<'static, str>>,
}

impl ApiKeyError {
    pub fn new(verbosity: ErrorVerbosity, r#type: ApiKeyErrorType) -> Self {
        let reason = verbosity
            .should_generate_error_context()
            .then(|| Self::reason(&r#type));

        ApiKeyError {
            verbosity,
            r#type,
            reason,
        }
    }

    fn reason(r#type: &ApiKeyErrorType) -> Cow<'static, str> {
        match r#type {
            ApiKeyErrorType::Missing => Cow::Borrowed("API key is missing"),
            ApiKeyErrorType::InvalidChars { err } => {
                Cow::Owned(format!("API key contains invalid characters: {err}"))
            }
            ApiKeyErrorType::Invalid => Cow::Borrowed("API key invalid"),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self.r#type {
            ApiKeyErrorType::Missing => StatusCode::UNAUTHORIZED,
            ApiKeyErrorType::InvalidChars { .. } => StatusCode::UNAUTHORIZED,
            ApiKeyErrorType::Invalid => StatusCode::FORBIDDEN,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum BasicAuthErrorType {
    /// Authorization header is missing.
    AuthMissing,
    /// Authorization header contains invalid characters.
    AuthInvalidChars {
        #[serde(skip)]
        err: ToStrError,
    },
    /// Authorization header could not be decoded.
    Decode {
        #[serde(skip)]
        err: DecodeError,
    },
    /// Decoded authorization header contains invalid characters.
    AuthInvalidUTF8 {
        #[serde(skip)]
        err: FromUtf8Error,
    },
    /// Authorization header is invalid Basic.
    InvalidBasic,
    /// Authentication failed.
    Invalid,
}

#[derive(Debug, Serialize)]
pub struct BasicAuthError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    r#type: BasicAuthErrorType,
    reason: Option<Cow<'static, str>>,
}

impl BasicAuthError {
    pub fn new(verbosity: ErrorVerbosity, r#type: BasicAuthErrorType) -> Self {
        let reason = verbosity
            .should_generate_error_context()
            .then(|| Self::reason(&r#type));

        BasicAuthError {
            verbosity,
            r#type,
            reason,
        }
    }

    fn reason(r#type: &BasicAuthErrorType) -> Cow<'static, str> {
        match r#type {
            BasicAuthErrorType::AuthMissing => Cow::Borrowed("Authorization header is missing"),
            BasicAuthErrorType::AuthInvalidChars { err } => Cow::Owned(format!(
                "Authorization header contains invalid characters: {err}"
            )),
            BasicAuthErrorType::Decode { err } => {
                Cow::Owned(format!("Authorization header could not be decoded: {err}"))
            }
            BasicAuthErrorType::InvalidBasic => {
                Cow::Borrowed("Authorization header is invalid Basic")
            }
            BasicAuthErrorType::AuthInvalidUTF8 { err } => Cow::Owned(format!(
                "Decoded authorization header contains invalid characters: {err}"
            )),
            BasicAuthErrorType::Invalid => Cow::Borrowed("Basic auth is invalid"),
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

#[derive(Debug, Serialize)]
pub enum BearerErrorType {
    /// Authorization header is missing.
    AuthMissing,
    /// Authorization header contains invalid characters.
    AuthInvalidChars {
        #[serde(skip)]
        err: ToStrError,
    },
    /// Authorization header is invalid Bearer.
    InvalidBearer,
}

#[derive(Debug, Serialize)]
pub struct BearerError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    r#type: BearerErrorType,
    reason: Option<Cow<'static, str>>,
}

impl BearerError {
    pub fn new(verbosity: ErrorVerbosity, r#type: BearerErrorType) -> Self {
        let reason = verbosity
            .should_generate_error_context()
            .then(|| Self::reason(&r#type));

        BearerError {
            verbosity,
            r#type,
            reason,
        }
    }

    fn reason(r#type: &BearerErrorType) -> Cow<'static, str> {
        match r#type {
            BearerErrorType::AuthMissing => Cow::Borrowed("Authorization header is missing"),
            BearerErrorType::AuthInvalidChars { err } => Cow::Owned(format!(
                "Authorization header contains invalid characters: {err}"
            )),
            BearerErrorType::InvalidBearer => {
                Cow::Borrowed("Authorization header is invalid Bearer")
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

#[derive(Debug, Serialize)]
pub enum JwtErrorType {
    /// JWT validation failed.
    Invalid {
        #[serde(skip)]
        err: JwtValidationError,
    },
    /// ExpiredSignature is a special case of Invalid.
    ///
    /// Intentionally extracted from the Invalid variant to provide a more specific error message.
    ExpiredSignature,
    /// User does not have a valid role.
    Forbidden,
}

#[derive(Debug, Serialize)]
pub struct JwtError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    r#type: JwtErrorType,
    reason: Option<Cow<'static, str>>,
}

impl JwtError {
    pub fn new(verbosity: ErrorVerbosity, r#type: JwtErrorType) -> Self {
        let reason = verbosity
            .should_generate_error_context()
            .then(|| Self::reason(&r#type));

        JwtError {
            verbosity,
            r#type,
            reason,
        }
    }

    fn reason(r#type: &JwtErrorType) -> Cow<'static, str> {
        match r#type {
            JwtErrorType::Invalid { err } => Cow::Owned(format!("JWT is invalid: {err}")),
            JwtErrorType::ExpiredSignature => Cow::Borrowed("JWT has expired"),
            JwtErrorType::Forbidden => Cow::Borrowed("User does not have a valid role"),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self.r#type {
            JwtErrorType::Invalid { .. } | JwtErrorType::ExpiredSignature => {
                StatusCode::UNAUTHORIZED
            }
            JwtErrorType::Forbidden => StatusCode::FORBIDDEN,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ValidationError {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    reason: Option<String>,
}

impl ValidationError {
    pub fn from_validation_errors(
        verbosity: ErrorVerbosity,
        validation_errors: ValidationErrors,
    ) -> Self {
        let reason = verbosity
            .should_generate_error_context()
            .then_some(validation_errors.to_string());

        ValidationError { verbosity, reason }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

#[derive(Debug, Serialize, ToSchema)]
struct ResourceErrorResponse<ET, C> {
    #[serde(flatten)]
    error: ResourceError<ET, C>,
    message: &'static str,
}

impl<ET, C> From<ResourceErrorResponse<ET, C>> for ErrorMessage {
    fn from(response: ResourceErrorResponse<ET, C>) -> Self {
        ErrorMessage {
            message: response.message,
        }
    }
}

/// Defined for a specific route.
///
/// ET: Error type. Must implement [`ResourceErrorProvider`].
/// C: Context wich contains additional information about the error
#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceError<ET, C> {
    #[serde(skip)]
    verbosity: ErrorVerbosity,
    #[serde(flatten)]
    error_type: ET,
    #[serde(rename = "error")]
    context: Option<C>,
}

/// Must be implemented for a specific error type to be used in [`ResourceError`].
pub trait ResourceErrorProvider {
    /// Resource specific context.
    ///
    /// Provides additional information about the error.
    /// Set to () if not needed.
    type Context;

    /// Headers to be returned with the error.
    fn headers(&self) -> Option<HeaderMap>;

    /// Status code to be returned with the error.
    fn status_code(&self) -> StatusCode;

    /// Message to be returned with the error.
    fn message(&self) -> &'static str;

    /// Context to be returned with the error.
    fn context(&self) -> Self::Context;
}

impl<ET, C> ResourceError<ET, C>
where
    ET: ResourceErrorProvider<Context = C>,
{
    pub fn new(verbosity: ErrorVerbosity, error_type: ET) -> Self {
        let context = verbosity
            .should_generate_error_context()
            .then_some(error_type.context());

        ResourceError {
            verbosity,
            error_type,
            context,
        }
    }
}

impl<ET, C> From<ResourceError<ET, C>> for ResourceErrorResponse<ET, C>
where
    ET: ResourceErrorProvider<Context = C>,
{
    fn from(error: ResourceError<ET, C>) -> Self {
        let message = error.error_type.message();

        ResourceErrorResponse { error, message }
    }
}

impl<ET, C> IntoResponse for ResourceError<ET, C>
where
    ET: ResourceErrorProvider<Context = C> + Serialize,
    C: Serialize,
{
    fn into_response(self) -> Response {
        ResourceErrorResponse::from(self).into_response()
    }
}

impl<ET, C> IntoResponse for ResourceErrorResponse<ET, C>
where
    ET: ResourceErrorProvider<Context = C> + Serialize,
    C: Serialize,
{
    fn into_response(self) -> Response {
        let headers = self.error.error_type.headers().unwrap_or_default();

        match self.error.verbosity {
            ErrorVerbosity::None => StatusCode::NO_CONTENT.into_response(),
            ErrorVerbosity::StatusCode => {
                (self.error.error_type.status_code(), headers).into_response()
            }
            ErrorVerbosity::Message => (
                self.error.error_type.status_code(),
                headers,
                Json(ErrorMessage::from(self)),
            )
                .into_response(),
            ErrorVerbosity::Type | ErrorVerbosity::Full => {
                (self.error.error_type.status_code(), headers, Json(self)).into_response()
            }
        }
    }
}
