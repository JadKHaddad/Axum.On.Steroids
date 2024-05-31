use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::error::{ApiError, ApiKeyError, ErrorVerbosityProvider};

pub trait ApiKeyProvider {
    /// Returns the API key header name.
    fn header_name(&self) -> &str;

    /// Validates the API key.
    fn validate(&self, key: &str) -> bool;
}

/// Extracts and validates the API key from the request headers.
pub struct ApiKey {
    pub key: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync + ApiKeyProvider + ErrorVerbosityProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "api_key_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let header_name = state.header_name();
        let headers = &parts.headers;

        let used_api_key = headers
            .get(header_name)
            .ok_or_else(|| {
                tracing::warn!("API key not found");

                ApiKeyError {
                    verbosity,
                    api_key_error_reason: "API key not found".to_string(),
                }
            })?
            .to_str()
            .map_err(|err| {
                tracing::warn!(%err, "API key header value is not a valid string");

                ApiKeyError {
                    verbosity,
                    api_key_error_reason: "API key header value is not a valid string".to_string(),
                }
            })?;

        if !state.validate(used_api_key) {
            tracing::warn!(used_api_key, "Invalid API key");

            return Err(ApiKeyError {
                verbosity,
                api_key_error_reason: "Invalid API key".to_string(),
            }
            .into());
        }

        Ok(ApiKey {
            key: used_api_key.to_string(),
        })
    }
}
