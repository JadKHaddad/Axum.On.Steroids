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

use crate::state::JwtValidationError;

// FIXME: Must not be public to all routes, to prevent defining arbitrary error verbosity.
// Create PrivateErrorVerbosity in state.rs. and use it as input here.
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
    internal_server_error: Option<String>,
}

impl InternalServerError {
    pub fn from_generic_error<E: Into<anyhow::Error>>(verbosity: ErrorVerbosity, err: E) -> Self {
        let err: anyhow::Error = err.into();
        let err = format!("{err:#}");
        tracing::error!(%err, "Internal server error");

        let internal_server_error = verbosity.should_generate_error_context().then_some(err);

        InternalServerError {
            verbosity,
            internal_server_error,
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
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
    query_error_type: QueryErrorType,
    query_error_reason: Option<String>,
    query_expected_schema: Option<String>,
}

impl QueryError {
    pub fn from_query_rejection<T: JsonSchema>(
        verbosity: ErrorVerbosity,
        query_rejection: QueryRejection,
    ) -> ApiError {
        let query_error_type = match query_rejection {
            QueryRejection::FailedToDeserializeQueryString(_) => QueryErrorType::DeserializeError,
            _ => return InternalServerError::from_generic_error(verbosity, query_rejection).into(),
        };

        let (query_error_reason, query_expected_schema) =
            match verbosity.should_generate_error_context() {
                true => {
                    let query_error_reason = query_rejection.body_text();
                    let query_expected_schema = match serde_yaml::to_string(&schema_for!(T)) {
                        Ok(schema) => schema,
                        Err(err) => {
                            return InternalServerError::from_generic_error(verbosity, err).into()
                        }
                    };

                    (Some(query_error_reason), Some(query_expected_schema))
                }
                false => (None, None),
            };

        QueryError {
            verbosity,
            query_error_type,
            query_error_reason,
            query_expected_schema,
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
    json_body_error_type: JsonBodyErrorType,
    json_body_error_reason: Option<String>,
    json_body_expected_schema: Option<String>,
}

impl JsonBodyError {
    pub fn from_json_rejection<T: JsonSchema>(
        verbosity: ErrorVerbosity,
        json_rejection: JsonRejection,
    ) -> ApiError {
        let json_body_error_type = match json_rejection {
            JsonRejection::JsonDataError(_) => JsonBodyErrorType::DataError,
            JsonRejection::JsonSyntaxError(_) => JsonBodyErrorType::SyntaxError,
            JsonRejection::MissingJsonContentType(_) => JsonBodyErrorType::MissingJsonContentType,
            _ => return InternalServerError::from_generic_error(verbosity, json_rejection).into(),
        };

        let (json_body_error_reason, json_body_expected_schema) =
            match verbosity.should_generate_error_context() {
                true => {
                    let json_body_error_reason = json_rejection.body_text();
                    let json_body_expected_schema = match serde_yaml::to_string(&schema_for!(T)) {
                        Ok(schema) => schema,
                        Err(err) => {
                            return InternalServerError::from_generic_error(verbosity, err).into()
                        }
                    };

                    (
                        Some(json_body_error_reason),
                        Some(json_body_expected_schema),
                    )
                }
                false => (None, None),
            };

        JsonBodyError {
            verbosity,
            json_body_error_type,
            json_body_error_reason,
            json_body_expected_schema,
        }
        .into()
    }

    fn status_code(&self) -> StatusCode {
        match self.json_body_error_type {
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
    path_error_type: PathErrorType,
    path_error_reason: Option<String>,
}

impl PathError {
    pub fn from_path_rejection(
        verbosity: ErrorVerbosity,
        path_rejection: PathRejection,
    ) -> ApiError {
        let path_error_type = match path_rejection {
            PathRejection::FailedToDeserializePathParams(ref err) => match err.kind() {
                PathErrorKind::Message(_)
                | PathErrorKind::InvalidUtf8InPathParam { .. }
                | PathErrorKind::ParseError { .. }
                | PathErrorKind::ParseErrorAtIndex { .. }
                | PathErrorKind::ParseErrorAtKey { .. } => PathErrorType::DeserializeError,
                _ => {
                    return InternalServerError::from_generic_error(verbosity, path_rejection)
                        .into()
                }
            },
            _ => return InternalServerError::from_generic_error(verbosity, path_rejection).into(),
        };

        let path_error_reason = verbosity
            .should_generate_error_context()
            .then_some(path_rejection.body_text());

        PathError {
            verbosity,
            path_error_type,
            path_error_reason,
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
    api_key_error_type: ApiKeyErrorType,
    api_key_error_reason: Option<Cow<'static, str>>,
}

impl ApiKeyError {
    pub fn new(verbosity: ErrorVerbosity, api_key_error_type: ApiKeyErrorType) -> Self {
        let api_key_error_reason = verbosity
            .should_generate_error_context()
            .then(|| Self::reason(&api_key_error_type));

        ApiKeyError {
            verbosity,
            api_key_error_type,
            api_key_error_reason,
        }
    }

    fn reason(api_key_error_type: &ApiKeyErrorType) -> Cow<'static, str> {
        match api_key_error_type {
            ApiKeyErrorType::Missing => Cow::Borrowed("API key is missing"),
            ApiKeyErrorType::InvalidChars { err } => {
                Cow::Owned(format!("API key contains invalid characters: {err}"))
            }
            ApiKeyErrorType::Invalid => Cow::Borrowed("API key invalid"),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self.api_key_error_type {
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
    basic_auth_error_type: BasicAuthErrorType,
    basic_auth_error_reason: Option<Cow<'static, str>>,
}

impl BasicAuthError {
    pub fn new(verbosity: ErrorVerbosity, basic_auth_error_type: BasicAuthErrorType) -> Self {
        let basic_auth_error_reason = verbosity
            .should_generate_error_context()
            .then(|| Self::reason(&basic_auth_error_type));

        BasicAuthError {
            verbosity,
            basic_auth_error_type,
            basic_auth_error_reason,
        }
    }

    fn reason(basic_auth_error_type: &BasicAuthErrorType) -> Cow<'static, str> {
        match basic_auth_error_type {
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
    bearer_error_type: BearerErrorType,
    bearer_error_reason: Option<Cow<'static, str>>,
}

impl BearerError {
    pub fn new(verbosity: ErrorVerbosity, bearer_error_type: BearerErrorType) -> Self {
        let bearer_error_reason = verbosity
            .should_generate_error_context()
            .then(|| Self::reason(&bearer_error_type));

        BearerError {
            verbosity,
            bearer_error_type,
            bearer_error_reason,
        }
    }

    fn reason(bearer_error_type: &BearerErrorType) -> Cow<'static, str> {
        match bearer_error_type {
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
    jwt_error_type: JwtErrorType,
    jwt_error_reason: Option<Cow<'static, str>>,
}

impl JwtError {
    pub fn new(verbosity: ErrorVerbosity, jwt_error_type: JwtErrorType) -> Self {
        let jwt_error_reason = verbosity
            .should_generate_error_context()
            .then(|| Self::reason(&jwt_error_type));

        JwtError {
            verbosity,
            jwt_error_type,
            jwt_error_reason,
        }
    }

    fn reason(jwt_error_type: &JwtErrorType) -> Cow<'static, str> {
        match jwt_error_type {
            JwtErrorType::Invalid { err } => Cow::Owned(format!("JWT is invalid: {err}")),
            JwtErrorType::ExpiredSignature => Cow::Borrowed("JWT has expired"),
            JwtErrorType::Forbidden => Cow::Borrowed("User does not have a valid role"),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self.jwt_error_type {
            JwtErrorType::Invalid { .. } | JwtErrorType::ExpiredSignature => {
                StatusCode::UNAUTHORIZED
            }
            JwtErrorType::Forbidden => StatusCode::FORBIDDEN,
        }
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
    error_type: ET,
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
        let message = match error.verbosity {
            ErrorVerbosity::None => "",
            _ => error.error_type.message(),
        };

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
