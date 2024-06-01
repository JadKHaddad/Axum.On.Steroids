use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{
    error::{ApiError, ApiKeyError, ApiKeyErrorType},
    traits::{ApiKeyProvider, ErrorVerbosityProvider},
    types::used_api_key::UsedApiKey,
};

/// Extracts the API key from the request headers.
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

        let used_api_key = headers
            .get(header_name)
            .ok_or_else(|| {
                tracing::warn!("Rejection. API key not found");

                ApiKeyError::new(verbosity, ApiKeyErrorType::Missing)
            })?
            .to_str()
            .map_err(|err| {
                tracing::warn!(%err, "Rejection. API key header value is not valid ASCII string");

                ApiKeyError::new(verbosity, ApiKeyErrorType::InvalidFromat)
            })?;

        tracing::trace!(%used_api_key, "Extracted");

        let used_api_key = used_api_key.to_string();

        Ok(ApiKey(UsedApiKey { used_api_key }))
    }
}
