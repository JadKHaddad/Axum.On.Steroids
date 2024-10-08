use std::future::Future;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{
    error::{ApiError, ApiKeyError, ApiKeyErrorType, ErrorVerbosityProvider},
    types::used_api_key::UsedApiKey,
};

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyProviderError<E> {
    #[error("Invalid")]
    Invalid,
    #[error(transparent)]
    InternalServerError(#[from] E),
}

pub trait ApiKeyProvider {
    type Error;

    /// Returns the API key header name.
    fn header_name(&self) -> &str;

    /// Validates the API key.
    fn validate(
        &self,
        key: &str,
    ) -> impl Future<Output = Result<(), ApiKeyProviderError<Self::Error>>> + Send;
}

/// Extracts the API key from the request headers.
#[derive(Debug, Clone)]
pub struct ApiKey(pub UsedApiKey);

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

        let api_key = headers
            .get(header_name)
            .ok_or_else(|| {
                tracing::warn!("Rejection. API key not found");

                ApiKeyError::new(verbosity, ApiKeyErrorType::Missing)
            })?
            .to_str()
            .map_err(|err| {
                tracing::warn!(%err, "Rejection. API key contains invalid characters");

                ApiKeyError::new(verbosity, ApiKeyErrorType::InvalidChars { err })
            })?;

        tracing::trace!(%api_key, "Extracted");

        let api_key = api_key.to_string();

        Ok(ApiKey(UsedApiKey { value: api_key }))
    }
}
